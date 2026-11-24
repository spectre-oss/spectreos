"""Exercise the recovery service without touching host networking or passwords."""
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]


class USBSSH(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.log = self.root / 'calls'
        self.bin = self.root / 'bin'
        self.bin.mkdir()
        mocks = {
            'ip': '\n'.join([
                'echo "ip $*" >> "$CALL_LOG"',
                '[[ ${NO_INTERFACE:-0} == 1 ]] && exit 1',
                'if [[ $1 == address && $2 == replace ]]; then touch "$CALL_LOG.address"; fi',
                'if [[ $1 == -4 ]]; then test -e "$CALL_LOG.address" && echo "inet 172.16.42.1/24"; else exit 0; fi',
            ]),
            'nmcli': 'echo "nmcli $*" >> "$CALL_LOG"; [[ ${NM_FAIL:-0} == 0 ]]',
            'sleep': ':',
            'iptables': 'echo "iptables $*" >> "$CALL_LOG"; [[ $1 != -C ]]',
            'dropbearkey': 'echo keygen >> "$CALL_LOG"; echo key > "$4"',
            'dropbear': 'echo "dropbear $*" >> "$CALL_LOG"',
        }
        for name, body in mocks.items():
            p = self.bin / name
            p.write_text('#!/bin/bash\n' + body + '\n')
            p.chmod(0o755)
        source = (ROOT / 'modules/usb-ssh-start.sh').read_text()
        source = source.replace('export PATH=/run/current-system/sw/bin:/run/wrappers/bin',
                                f'export PATH={self.bin}:/usr/bin:/bin')
        source = source.replace('/run/spectre-usb-ssh', str(self.bin))
        source = source.replace('/var/lib/spectre-usb-ssh', str(self.root / 'state'))
        source = source.replace('@iptables@', str(self.bin / 'iptables'))
        self.script = self.root / 'start.sh'
        self.script.write_text(source)

    def run_service(self, **env):
        self.log.write_text('')
        result = subprocess.run(['bash', str(self.script)], capture_output=True, text=True,
                                env=os.environ | {'CALL_LOG': str(self.log)} | env)
        return result, self.log.read_text()

    def test_scoped_listener_and_key_reuse(self):
        result, calls = self.run_service()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn('keygen', calls)
        self.assertIn('ip address replace 172.16.42.1/24 dev usb0', calls)
        self.assertIn('nmcli device set usb0 managed no', calls)
        self.assertIn('-i usb0 -s 172.16.42.2/32 -d 172.16.42.1/32 -p tcp --dport 22', calls)
        self.assertIn('-i wlan0 -p tcp --dport 22 -j ACCEPT', calls)
        self.assertIn('-F -E -w -j -k -p 0.0.0.0:22', calls)
        self.assertNotIn(' -B ', calls)  # Never allow empty passwords.
        result, calls = self.run_service()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertNotIn('keygen', calls)

    def test_networkmanager_failure_does_not_start_ssh(self):
        result, calls = self.run_service(NM_FAIL='1')
        self.assertNotEqual(result.returncode, 0)
        self.assertNotIn('iptables', calls)
        self.assertNotIn('dropbear', calls)

    def test_missing_usb_interface_never_opens_firewall_or_server(self):
        result, calls = self.run_service(NO_INTERFACE='1')
        self.assertNotEqual(result.returncode, 0)
        self.assertNotIn('iptables', calls)
        self.assertNotIn('dropbear', calls)


if __name__ == '__main__':
    unittest.main()
