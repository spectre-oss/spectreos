#!/usr/bin/env python3
"""Explicit Poco integration check. Requires an SSH-forwarded socket; changes sleep/wake state.
Does not unlock, enter credentials, type text, or alter device settings.
"""
import argparse
import json
from pathlib import Path
import socket
import struct
import time
import zlib
from client import Client


def decode_png(data):
    assert data[:8] == b"\x89PNG\r\n\x1a\n"
    pos, compressed = 8, bytearray()
    while pos < len(data):
        size = struct.unpack_from(">I", data, pos)[0]
        kind, chunk = data[pos+4:pos+8], data[pos+8:pos+8+size]
        if kind == b"IHDR":
            w, h, depth, color, _, _, interlace = struct.unpack(">IIBBBBB", chunk)
            assert (depth, color, interlace) == (8, 6, 0)
        if kind == b"IDAT":
            compressed.extend(chunk)
        pos += size + 12
    raw, stride = zlib.decompress(compressed), w * 4
    pixels, previous = bytearray(), bytearray(stride)
    for y in range(h):
        kind = raw[y*(stride+1)]
        row = bytearray(raw[y*(stride+1)+1:(y+1)*(stride+1)])
        for x in range(stride):
            a, b, c = (row[x-4] if x >= 4 else 0), previous[x], (previous[x-4] if x >= 4 else 0)
            if kind == 1:
                predictor = a
            elif kind == 2:
                predictor = b
            elif kind == 3:
                predictor = (a+b)//2
            elif kind == 4:
                p = a+b-c
                pa, pb, pc = abs(p-a), abs(p-b), abs(p-c)
                predictor = a if pa <= pb and pa <= pc else b if pb <= pc else c
            elif kind == 0:
                predictor = 0
            else:
                raise AssertionError("unknown PNG filter")
            row[x] = (row[x]+predictor) & 255
        pixels.extend(row)
        previous = row
    return w, h, pixels


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--port", type=int, default=8765)
    p.add_argument("--output", type=Path, required=True)
    args = p.parse_args()
    results = {}
    with Client(port=args.port) as client:
        results["status"] = client.request("status")[0]
        assert results["status"]["contacts"] == results["status"]["keys"] == 0
        # Exclusive session: a second hello must be refused without taking over.
        second = socket.create_connection(("127.0.0.1", args.port), timeout=2)
        second.sendall(struct.pack(">II", 35, 0) + b'{"id":1,"type":"hello","version":1}')
        try:
            assert not second.recv(8)
        except ConnectionResetError:
            pass
        finally:
            second.close()
        results["exclusive_session"] = True
        client.request("sleep")
        try:
            client.request("screenshot")
            raise AssertionError("screen-off capture unexpectedly succeeded")
        except RuntimeError as error:
            assert "screen is off" in str(error)
        client.request("wake")
        time.sleep(.2)
        assert client.request("status")[0]["locked"]
        results["wake_preserves_lock"] = True
        # Invalid contact transitions and bounds must be rejected.
        for command, fields in [("touch_up", dict(slot=0)),
                                ("touch_down", dict(slot=10, x=1, y=1)),
                                ("touch_down", dict(slot=0, x=-1, y=1)),
                                ("key", dict(code=0, down=True))]:
            try:
                client.request(command, **fields)
                raise AssertionError("invalid event accepted")
            except RuntimeError:
                pass
        results["invalid_input_rejected"] = True
        # Empty lock-screen region; cancel rather than tapping or entering digits.
        client.request("touch_down", slot=0, x=10, y=250)
        client.request("touch_move", slot=0, x=12, y=250)
        client.request("touch_down", slot=1, x=20, y=250)
        assert client.request("status")[0]["contacts"] == 2
        client.request("touch_cancel")
        client.request("key", code=42, down=True)
        assert client.request("status")[0]["keys"] == 1
        client.request("key", code=42, down=False)
        results["multitouch_and_keys"] = True
        response, png = client.request("screenshot")
        w, h, pixels = decode_png(png)
        assert (w, h) == (response["width"], response["height"])
        def pixel(x, y):
            return tuple(pixels[(y*w+x)*4:(y*w+x+1)*4])
        assert response["indicator"] == "device_only"
        assert pixel(200, 120) != (180, 20, 30, 255), pixel(200, 120)
        assert pixel(200, h-120) != (180, 20, 30, 255)
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.with_suffix(".png").write_bytes(png)
        results["upright_capture_without_device_indicator"] = {"width": w, "height": h, "bytes": len(png)}
        client.request("stream", fps=5, codec="png")
        started, count, total = time.monotonic(), 0, 0
        while time.monotonic() - started < 3:
            meta, binary = client.receive()
            if meta.get("event") == "frame":
                assert meta["ok"] and binary.startswith(b"\x89PNG")
                count += 1
                total += len(binary)
                # Responses and live frames share the same connection.
                client.request("heartbeat")
        client.request("stream", fps=0)
        results["stream"] = {"frames": count, "elapsed": round(time.monotonic()-started, 3), "bytes": total}
        assert count >= 3
        # Deliberately close without releasing input; next session must start clean.
        client.request("touch_down", slot=0, x=10, y=250)
        client.request("key", code=42, down=True)
        client.socket.close()
    time.sleep(.2)
    with Client(port=args.port) as client:
        status = client.request("status")[0]
        assert status["contacts"] == status["keys"] == 0
        results["disconnect_releases_input"] = True
        # No heartbeat: connection must be revoked, including held input.
        client.request("key", code=42, down=True)
        time.sleep(10.5)
        try:
            client.request("status")
            raise AssertionError("expired session remained active")
        except (OSError, ConnectionError):
            pass
    with Client(port=args.port) as client:
        status = client.request("status")[0]
        assert status["contacts"] == status["keys"] == 0
        results["timeout_releases_input"] = True
    args.output.write_text(json.dumps(results, indent=2)+"\n")
    print(json.dumps(results, indent=2))


if __name__ == "__main__":
    main()
