#!/usr/bin/env python3
"""Inspect CC13 candidates in the archived MIUI Chromatix binaries.

Read-only exploratory decoder for the observed 1.3 container layout. This
does not select a sensor module, resolve tuning branches, generate a libcamera
profile, or install calibration. The decoded layout is inferred from these
archives and must be checked against the vendor schema before production use.
"""
import argparse
import hashlib
import json
import math
from pathlib import Path
import struct


def inspect(path):
    data = path.read_bytes()

    def block(offset, size):
        if offset < 0 or size < 0 or offset + size > len(data):
            raise ValueError("Container section exceeds file bounds")
        return data[offset:offset + size]

    def integers(offset, count):
        return struct.unpack("<" + "I" * count, block(offset, count * 4))

    if block(0, 20) != b"QTI Chromatix Header":
        raise ValueError("Not the expected Chromatix container")
    length, major, minor = integers(28, 3)
    if length != len(data) or (major, minor) != (1, 3):
        raise ValueError("Only the observed complete version 1.3 container is supported")
    table, count = integers(152, 2)
    sections = {}
    for index in range(count):
        kind, offset, size = integers(table + 12 * index, 3)
        block(offset, size)
        if kind in sections:
            raise ValueError("Duplicate section kind")
        sections[kind] = (offset, size)
    symbols, symbol_size = sections[0]
    payload, payload_size = sections[1]
    if symbol_size % 64:
        raise ValueError("Unexpected symbol-table entry size")
    candidates = []
    for offset in range(symbols, symbols + symbol_size, 64):
        identifier = integers(offset, 1)[0]
        name = block(offset + 4, 48).split(b"\0", 1)[0]
        flags, relative, size = integers(offset + 52, 3)
        if name != b"mod_cc13_cct_data":
            continue
        if size % 56 or relative + size > payload_size:
            raise ValueError("Unexpected CC13 record layout")
        records = []
        for position in range(payload + relative, payload + relative + size, 56):
            values = struct.unpack("<14f", block(position, 56))
            if not all(math.isfinite(value) for value in values):
                raise ValueError("Nonfinite CC13 candidate")
            matrix = [list(values[2 + row * 3:5 + row * 3]) for row in range(3)]
            records.append({
                "file_offset": position,
                "candidate_temperature_bounds": list(values[:2]),
                "candidate_matrix": matrix,
                "row_sums": [sum(row) for row in matrix],
                "candidate_offsets": list(values[11:]),
            })
        candidates.append({"symbol_id": identifier, "symbol_offset": offset,
                           "flags": flags, "records": records})
    return {
        "source_name": path.name,
        "source_sha256": hashlib.sha256(data).hexdigest(),
        "status": "Exploratory candidates only; schema, branch and module selection unverified",
        "candidates": candidates,
    }


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    args = parser.parse_args()
    print(json.dumps(inspect(args.binary), indent=2))
