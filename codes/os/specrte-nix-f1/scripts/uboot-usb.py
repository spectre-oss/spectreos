#!/usr/bin/env python3
"""Control Spectre U-Boot over a directly connected USB Ethernet link."""
import argparse
from pathlib import Path
import select
import socket
import struct
import sys
import threading
import time


class SingleFileTFTP:
    """Read-only TFTP: one explicit FIT, one peer, no filesystem browsing."""
    def __init__(self, path, host, phone, port=1069, timeout=3):
        self.path, self.host, self.phone = Path(path), host, phone
        self.port, self.timeout = port, timeout
        self.listener = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        self.listener.bind((host, port))
        self.listener.settimeout(0.2)
        self.stopped = threading.Event()
        self.completed = threading.Event()
        self.error = None

    def close(self):
        self.stopped.set()

    def serve(self):
        try:
            while not self.stopped.is_set():
                try:
                    packet, peer = self.listener.recvfrom(2048)
                except socket.timeout:
                    continue
                if peer[0] != self.phone:
                    continue
                fields = packet[2:].split(b'\0')
                if packet[:2] != b'\0\1' or len(fields) < 3 or fields[:2] != [b'spectre.itb', b'octet']:
                    self.listener.sendto(b'\0\5\0\1Only spectre.itb in octet mode is available\0', peer)
                    continue
                self.transfer(peer)
                self.completed.set()
                return
        except (OSError, RuntimeError) as error:
            self.error = error
        finally:
            self.listener.close()

    def transfer(self, peer):
        # Sending DATA directly declines optional block-size negotiation (RFC 2347).
        with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as connection, self.path.open('rb') as stream:
            connection.bind((self.host, 0))
            connection.settimeout(self.timeout)
            block = 1
            while not self.stopped.is_set():
                data = stream.read(512)
                packet = struct.pack('!HH', 3, block) + data
                acknowledged = False
                for attempt in range(6):
                    connection.sendto(packet, peer)
                    deadline = time.monotonic() + self.timeout
                    while time.monotonic() < deadline and not self.stopped.is_set():
                        connection.settimeout(max(0.01, deadline - time.monotonic()))
                        try:
                            ack, sender = connection.recvfrom(2048)
                        except socket.timeout:
                            break
                        if sender == peer and ack == struct.pack('!HH', 4, block):
                            acknowledged = True
                            break
                    if acknowledged:
                        break
                if not acknowledged:
                    raise RuntimeError(f'TFTP block {block} was not acknowledged')
                if len(data) < 512:
                    return
                block = (block + 1) & 0xffff
            raise RuntimeError('TFTP transfer cancelled')


def connect(host, phone):
    connection = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    connection.bind((host, 6666))
    connection.settimeout(0.5)
    marker = b'SPECTRE_HOST_READY'
    connection.sendto(b'echo ' + marker + b'\n', (phone, 6666))
    deadline = time.monotonic() + 10
    received = b''
    while time.monotonic() < deadline:
        try:
            data, peer = connection.recvfrom(65535)
        except socket.timeout:
            continue
        if peer[0] != phone:
            continue
        sys.stdout.write(data.decode(errors='replace'))
        sys.stdout.flush()
        received = (received + data)[-8192:]
        # A returned prompt confirms completion; echoed input alone is insufficient.
        if marker in received and b'=> ' in received.split(marker)[-1]:
            return connection
    connection.close()
    raise RuntimeError('No U-Boot prompt received. Check USB networking and the host IP; no boot command was sent.')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action', choices=['console', 'continue', 'replace'])
    parser.add_argument('--host', default='172.16.42.2')
    parser.add_argument('--phone', default='172.16.42.1')
    parser.add_argument('--fit', type=Path, help='FIT to offer as spectre.itb for RAM-only boot')
    args = parser.parse_args()
    if args.action == 'replace':
        if not args.fit or not args.fit.is_file():
            parser.error('replace requires --fit PATH')
        with args.fit.open('rb') as stream:
            if stream.read(4) != b'\xd0\x0d\xfe\xed':
                parser.error('Expected a U-Boot FIT file')
        if args.fit.stat().st_size > 120 * 1024 * 1024:
            parser.error('FIT exceeds the reserved download buffer limit')
    server = None
    connection = connect(args.host, args.phone)
    try:
        if args.action == 'replace':
            server = SingleFileTFTP(args.fit, args.host, args.phone)
            threading.Thread(target=server.serve, daemon=True).start()
            command = b'setenv tftpdstp 1069; run spectre_replace\n'
        elif args.action == 'continue':
            command = b'run spectre_continue\n'
        else:
            command = b'run spectre_help\n'
        connection.sendto(command, (args.phone, 6666))
        print('\nUSB console active. Ctrl-C exits this client. Linux will disconnect this console when it boots.', flush=True)
        while True:
            if server and server.error:
                raise RuntimeError(str(server.error))
            ready, _, _ = select.select([connection, sys.stdin], [], [], 0.5)
            if connection in ready:
                data, peer = connection.recvfrom(65535)
                if peer[0] == args.phone:
                    sys.stdout.write(data.decode(errors='replace'))
                    sys.stdout.flush()
            if sys.stdin in ready:
                line = sys.stdin.readline()
                if not line:
                    return
                if len(line.encode()) > 400:
                    print('Command too long (400-byte limit).', file=sys.stderr)
                    continue
                connection.sendto(line.encode(), (args.phone, 6666))
    finally:
        connection.close()
        if server:
            server.close()


if __name__ == '__main__':
    try:
        main()
    except KeyboardInterrupt:
        pass
    except (OSError, RuntimeError) as error:
        sys.exit(str(error))
