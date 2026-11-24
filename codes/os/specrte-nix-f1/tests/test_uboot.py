"""Exercise boot packaging, USB transfer and partition boundaries offline."""
import gzip
import importlib.util
from pathlib import Path
import socket
import struct
import sys
import tempfile
import threading
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, ROOT / path)
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


package = load('uboot_package', 'bootloader/package.py')
client = load('uboot_client', 'scripts/uboot-usb.py')
load('fastboot_helpers', 'scripts/release-install.py')
installer = load('uboot_installer', 'scripts/install-uboot.py')


class Package(unittest.TestCase):
    def test_layout_and_truncation(self):
        header = bytearray(4096)
        header[:8] = b'ANDROID!'
        struct.pack_into('<I', header, 36, 4096)
        kernel = bytearray(128)
        kernel[56:60] = b'ARM\x64'
        dtb = b'\xd0\x0d\xfe\xed' + bytes(60)
        compressed = gzip.compress(kernel, mtime=0)
        image = package.wrap_boot(header, compressed + dtb, b'initrd')
        self.assertEqual(package.split_boot(image), (compressed, b'initrd', dtb, ''))
        self.assertEqual(len(image) % 4096, 0)
        with self.assertRaises(ValueError):
            package.split_boot(image[:4100])
        damaged = bytearray(image)
        struct.pack_into('<I', damaged, 40, 3)
        with self.assertRaises(ValueError):
            package.split_boot(damaged)

    def test_no_automatic_boot(self):
        env = (ROOT / 'bootloader/spectre.env').read_text()
        self.assertIn('bootdelay=-1\n', env)
        self.assertIn('bootretry=-1\n', env)
        preboot = next(line for line in env.splitlines() if line.startswith('preboot='))
        self.assertNotIn('bootm ', preboot)
        self.assertNotIn('bootefi ', env)
        self.assertNotIn('saveenv', env)
        self.assertIn('tftpboot ${loadaddr} spectre.itb && iminfo ${loadaddr} && bootm ${loadaddr}', env)


class Transfer(unittest.TestCase):
    def test_readonly_transfer_and_retransmission(self):
        for size in (0, 512, 1300):
            with self.subTest(size=size), tempfile.TemporaryDirectory() as temp:
                path = Path(temp) / 'payload'
                payload = bytes(n % 251 for n in range(size))
                path.write_bytes(payload)
                server = client.SingleFileTFTP(path, '127.0.0.1', '127.0.0.2', port=0, timeout=0.05)
                address = server.listener.getsockname()
                thread = threading.Thread(target=server.serve)
                thread.start()
                try:
                    with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as peer:
                        peer.bind(('127.0.0.2', 0))
                        peer.settimeout(2)
                        peer.sendto(b'\0\1../secret\0octet\0', address)
                        self.assertEqual(peer.recvfrom(4096)[0][:2], b'\0\5')
                        peer.sendto(b'\0\1spectre.itb\0octet\0blksize\01468\0', address)
                        received = b''
                        block = 1
                        while True:
                            data, sender = peer.recvfrom(4096)
                            self.assertEqual(data[:4], struct.pack('!HH', 3, block))
                            if block == 1:  # Drop the first ACK; require a retry.
                                retry, retry_sender = peer.recvfrom(4096)
                                self.assertEqual((retry, retry_sender), (data, sender))
                            peer.sendto(struct.pack('!HH', 4, block), sender)
                            received += data[4:]
                            if len(data) < 516:
                                break
                            block += 1
                        self.assertEqual(received, payload)
                        self.assertTrue(server.completed.wait(2))
                        self.assertIsNone(server.error)
                finally:
                    server.close()
                    thread.join(2)


class Phone:
    def __init__(self, product='beryllium', fail=False):
        self.product, self.fail, self.calls = product, fail, []

    def device(self, *args):
        self.calls.append(args)
        if args == ('getvar', 'product'):
            return 'product: ' + self.product
        if args == ('oem', 'device-info'):
            return 'Device unlocked: true'
        if args == ('getvar', 'partition-size:boot'):
            return 'partition-size:boot: 0x04000000'
        if args[0] == 'flash' and self.fail:
            raise RuntimeError('transfer failed')
        return 'OK'


class Install(unittest.TestCase):
    def test_test_install_and_rollback_boundaries(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            for name in ('uboot-boot.img', 'rollback-boot.img'):
                (root / name).write_bytes(b'ANDROID!' + bytes(32))
            phone = Phone()
            installer.perform(phone, root, 'test')
            self.assertTrue(any(call[0] == 'boot' for call in phone.calls))
            self.assertFalse(any(call[0] == 'flash' for call in phone.calls))
            phone = Phone()
            with self.assertRaises(RuntimeError):
                installer.perform(phone, root, 'install')
            self.assertFalse(any(call[0] == 'flash' for call in phone.calls))
            for action, phrase in [('install', 'INSTALL tested UBOOT beryllium'), ('rollback', 'RESTORE beryllium')]:
                phone = Phone()
                with patch('builtins.input', return_value=phrase):
                    installer.perform(phone, root, action, tested=True)
                self.assertEqual([c[:2] for c in phone.calls if c[0] == 'flash'], [('flash', 'boot')])
                self.assertEqual(phone.calls[-1], ('reboot',))
            phone = Phone(fail=True)
            with patch('builtins.input', return_value='RESTORE beryllium'), self.assertRaises(RuntimeError):
                installer.perform(phone, root, 'rollback')
            self.assertNotIn(('reboot',), phone.calls)
            with self.assertRaises(RuntimeError):
                installer.perform(Phone(product='other'), root, 'test')


if __name__ == '__main__':
    unittest.main()
