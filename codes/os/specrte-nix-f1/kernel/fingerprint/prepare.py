#!/usr/bin/env python3
"""Validate and package a QSEECOM test kernel; never access or write a device."""
import argparse
import gzip
import hashlib
import importlib.util
import json
from pathlib import Path
import zlib

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location("boot_package", ROOT / "bootloader/package.py")
boot = importlib.util.module_from_spec(spec)
spec.loader.exec_module(boot)


def config(kernel):
    raw = gzip.decompress(kernel)
    if raw[56:60] != b"ARM\x64":
        raise ValueError("Not an ARM64 kernel")
    marker = raw.find(b"IKCFG_ST\x1f\x8b\x08")
    if marker < 0:
        raise ValueError("Kernel configuration missing")
    return zlib.decompress(raw[marker + 8:], 31).decode()


def settings(text):
    result = {}
    for line in text.splitlines():
        if line.startswith("CONFIG_"):
            key, value = line.split("=", 1)
            result[key] = value
        elif line.startswith("# CONFIG_") and line.endswith(" is not set"):
            result[line.split()[1]] = "n"
    return result


def prepare(backup, running_config, kernel_dir, output):
    original = backup.read_bytes()
    if len(original) != 64 * 1024 * 1024:
        raise ValueError("Expected complete 64 MiB boot partition backup")
    old_kernel, initrd, old_dtb, cmdline = boot.split_boot(original)
    before = settings(config(old_kernel))
    if before != settings(gzip.decompress(running_config.read_bytes()).decode()):
        raise ValueError("Backup configuration differs from running phone")
    kernel = (kernel_dir / "vmlinuz-7.1.0-rc1").read_bytes()
    after = settings(config(kernel))
    changes = {k: [before.get(k, "n"), after.get(k, "n")]
               for k in before.keys() | after.keys()
               if before.get(k, "n") != after.get(k, "n")}
    expected = {"CONFIG_QCOM_QSEECOM": ["n", "y"],
                "CONFIG_TEE_QSEECOM": ["n", "y"]}
    if changes != expected:
        raise ValueError("Unexpected config changes: " + json.dumps(changes))
    # This experiment changes only the transport, so retain the known boot DTB.
    image = boot.wrap_boot(original, kernel + old_dtb, initrd)
    if boot.split_boot(image) != (kernel, initrd, old_dtb, cmdline):
        raise ValueError("Boot image round-trip failed")
    output.mkdir(parents=True, exist_ok=False)
    files = {"boot.img": image, "Image.gz": kernel,
             "Image": gzip.decompress(kernel), "initrd": initrd,
             "boot.dtb": old_dtb}
    manifest = {"purpose": "experimental QSEECOM transport test",
                "phoneBootTested": False, "configChanges": changes,
                "kernelDirectory": str(kernel_dir.resolve()),
                "backupSha256": hashlib.sha256(original).hexdigest(),
                "files": {}}
    for name, data in files.items():
        (output / name).write_bytes(data)
        manifest["files"][name] = {"bytes": len(data),
                                    "sha256": hashlib.sha256(data).hexdigest()}
    (output / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    print(json.dumps(manifest, indent=2))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("backup", type=Path)
    parser.add_argument("running_config", type=Path)
    parser.add_argument("kernel_dir", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    prepare(args.backup, args.running_config, args.kernel_dir, args.output)
