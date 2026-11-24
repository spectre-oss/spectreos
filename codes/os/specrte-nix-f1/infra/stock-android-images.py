#!/usr/bin/env python3
"""Reconstruct full Android block-OTA images locally. Never accesses a device."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import zipfile


def reconstruct(rom, name, output, partition_size):
    with zipfile.ZipFile(rom) as archive:
        lines = archive.read(name + '.transfer.list').decode().splitlines()
        if lines[0] != '4' or lines[2:4] != ['0', '0']:
            raise ValueError('Only full version4 transfers without stashes are supported')
        operations = []
        for line in lines[4:]:
            command, ranges = line.split()
            if command not in ('new', 'zero', 'erase'):
                raise ValueError('Not a full OTA: ' + command)
            values = [int(x) for x in ranges.split(',')]
            if values[0] != len(values) - 1 or values[0] % 2:
                raise ValueError('Malformed block ranges')
            for begin, end in zip(values[1::2], values[2::2]):
                if not 0 <= begin < end <= partition_size // 4096:
                    raise ValueError('Range exceeds target partition')
                operations.append((command, begin * 4096, (end - begin) * 4096))
        compressed = output.with_suffix('.new.dat.br')
        with archive.open(name + '.new.dat.br') as source, compressed.open('wb') as dest:
            while chunk := source.read(1024 * 1024):
                dest.write(chunk)
    process = subprocess.Popen(['brotli', '--decompress', '--stdout', str(compressed)], stdout=subprocess.PIPE)
    try:
        with output.open('wb') as image:
            image.truncate(partition_size)
            for command, offset, size in operations:
                if command != 'new':
                    continue  # New sparse file already represents zero/erased ranges.
                image.seek(offset)
                while size:
                    data = process.stdout.read(min(size, 1024 * 1024))
                    if not data:
                        raise ValueError('Truncated decompressed OTA data')
                    image.write(data)
                    size -= len(data)
            if process.stdout.read(1):
                raise ValueError('Unused decompressed OTA data')
        if process.wait():
            raise ValueError('Brotli decompression failed')
    finally:
        process.stdout.close()
        if process.poll() is None:
            process.kill()
            process.wait()
    digest = hashlib.sha256()
    with output.open('rb') as image:
        while chunk := image.read(8 * 1024 * 1024):
            digest.update(chunk)
    print(json.dumps({'image': str(output), 'bytes': partition_size, 'sha256': digest.hexdigest()}))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('rom', type=Path)
    parser.add_argument('name', choices=['system', 'vendor'])
    parser.add_argument('output', type=Path)
    parser.add_argument('partition_size', type=int)
    args = parser.parse_args()
    if args.output.exists():
        parser.error('Refusing to overwrite an existing image')
    if args.partition_size <= 0 or args.partition_size % 4096:
        parser.error('Partition size must be a positive multiple of4096')
    reconstruct(args.rom, args.name, args.output, args.partition_size)
