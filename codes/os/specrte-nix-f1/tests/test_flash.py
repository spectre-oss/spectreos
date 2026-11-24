"""Exercise partition-write boundaries with a fake fastboot, never real hardware."""
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

SCRIPT = Path(__file__).resolve().parents[1] / "scripts/flash.sh"
MOCK = '''#!/usr/bin/env bash
printf '%s\\n' "$*" >> "$CALL_LOG"
if [[ $1 == devices ]]; then
  [[ ${NO_DEVICE:-0} == 1 ]] || printf 'testserial fastboot\\n'
  exit 0
fi
shift 2
case "$*" in
 'getvar product') echo "product: ${PRODUCT:-beryllium}" >&2;;
 'oem device-info') echo "Device unlocked: ${UNLOCKED:-true}" >&2;;
 'getvar partition-size:boot') echo "partition-size:boot: ${BOOT_SIZE:-0x04000000}" >&2;;
 'flash userdata '*) [[ ${FAIL_ROOTFS:-0} == 0 ]];;
 *) exit 0;;
esac
'''


class FlashBoundaries(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.path = Path(self.temp.name)
        (self.path / "boot.img").write_bytes(b"ANDROID!" + bytes(64))
        (self.path / "system.img").write_bytes(bytes(4096))
        mock = self.path / "fastboot"
        mock.write_text(MOCK)
        mock.chmod(0o755)

    def run_flash(self, mode="install", images=None, **extra):
        log = self.path / "calls"
        log.write_text("")
        env = os.environ | {
            "PATH": str(self.path) + ":" + os.environ["PATH"],
            "CALL_LOG": str(log), "SERIAL": "", "ERASE_USERDATA": "",
        } | extra
        result = subprocess.run(
            ["bash", str(SCRIPT), mode, str(images or self.path)],
            env=env, capture_output=True, text=True,
        )
        return result.returncode, log.read_text()

    def test_refuses_writes(self):
        for settings in [
            {}, {"PRODUCT": "other"}, {"UNLOCKED": "false"},
            {"NO_DEVICE": "1"}, {"BOOT_SIZE": "0x00000001"},
            {"SERIAL": "missing"},
        ]:
            with self.subTest(settings=settings):
                ack = {} if not settings else {"ERASE_USERDATA": "beryllium"}
                code, calls = self.run_flash(**(ack | settings))
                self.assertNotEqual(code, 0)
                self.assertNotIn("flash ", calls)

    def test_rootfs_failure_preserves_boot(self):
        code, calls = self.run_flash(ERASE_USERDATA="beryllium", FAIL_ROOTFS="1")
        self.assertNotEqual(code, 0)
        self.assertNotIn("flash boot ", calls)
        self.assertNotIn("reboot", calls)

    def test_temporary_boot_accepts_file_without_partition_writes(self):
        code, calls = self.run_flash(mode="boot", images=self.path / "boot.img")
        self.assertEqual(code, 0)
        self.assertIn(" boot ", calls)
        self.assertNotIn("flash ", calls)

    def test_installation_order(self):
        code, calls = self.run_flash(ERASE_USERDATA="beryllium")
        self.assertEqual(code, 0)
        self.assertLess(calls.index("flash userdata"), calls.index("flash boot"))
        self.assertLess(calls.index("flash boot"), calls.index("reboot"))
        self.assertNotIn("recovery", calls)


if __name__ == "__main__":
    unittest.main()
