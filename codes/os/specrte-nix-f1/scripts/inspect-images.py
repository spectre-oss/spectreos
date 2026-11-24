#!/usr/bin/env python3
"""Validate the produced legacy Android boot image and raw ext4 rootfs."""
import argparse
import hashlib
import json
from pathlib import Path
import struct
import zlib

parser = argparse.ArgumentParser()
parser.add_argument("images", type=Path)
parser.add_argument("--panel", choices=["tianma", "ebbg"], default="tianma")
parser.add_argument("--boot-only", action="store_true", help="inspect a standalone boot image without requiring rootfs")
args = parser.parse_args()
boot = args.images if args.boot_only and args.images.is_file() else args.images / "boot.img"
rootfs = args.images / "system.img"
expected_version = json.loads((Path(__file__).resolve().parents[1] / "sources.json").read_text())["kernel"]["version"]
data = boot.read_bytes()
if len(data) < 48 or data[:8] != b"ANDROID!":
    raise SystemExit("Not a legacy Android boot image")
kernel_size, ramdisk_size, second_size, page_size, dt_size = (
    struct.unpack_from("<I", data, offset)[0] for offset in [8, 16, 24, 36, 40]
)
if page_size != 4096 or not kernel_size or not ramdisk_size:
    raise SystemExit("Unexpected Poco F1 boot-image layout")
align = lambda size: (size + page_size - 1) // page_size * page_size
required = page_size + align(kernel_size) + align(ramdisk_size) + align(second_size) + dt_size
if len(data) < required:
    raise SystemExit("Truncated boot image")
compressed_kernel = data[page_size:page_size + kernel_size]
decoder = zlib.decompressobj(16 + zlib.MAX_WBITS)
kernel = decoder.decompress(compressed_kernel) + decoder.flush()
if not decoder.eof or ("Linux version " + expected_version + " ").encode() not in kernel:
    raise SystemExit("Boot image does not contain the pinned Linux " + expected_version + " kernel")
if len(kernel) < 64 or struct.unpack_from("<I", kernel, 56)[0] != 0x644D5241:
    raise SystemExit("Kernel is not an AArch64 Linux Image")
config_start = kernel.find(b"IKCFG_ST\x1f\x8b\x08")
if config_start < 0:
    raise SystemExit("Kernel lacks the embedded configuration needed for driver validation")
config_decoder = zlib.decompressobj(16 + zlib.MAX_WBITS)
config_lines = set(config_decoder.decompress(kernel[config_start + 8:]).decode().splitlines())
required_drivers = [
    "BATTERY_PMI8998_FG", "CHARGER_QCOM_SMB2", "GPIO_SHARED_PROXY",
    "SCSI_UFS_QCOM", "MMC_SDHCI_MSM", "EXT4_FS", "ATH10K_SNOC", "DRM_MSM",
    "USB_DWC3_QCOM", "PHY_QCOM_QMP_USB",
    "REGULATOR_QCOM_LABIBB", "BACKLIGHT_QCOM_WLED", "ZRAM_BACKEND_ZSTD",
]
if args.panel == "tianma":
    required_drivers += ["DRM_PANEL_NOVATEK_NT36672A", "TOUCHSCREEN_NOVATEK_NVT_TS"]
missing_drivers = [name for name in required_drivers if "CONFIG_" + name + "=y" not in config_lines]
if missing_drivers:
    raise SystemExit("Kernel is missing built-in phone drivers: " + ", ".join(missing_drivers))
if "# CONFIG_QCOM_PD_MAPPER is not set" not in config_lines:
    raise SystemExit("Kernel does not apply the upstream Poco F1 microphone/PD-mapper workaround")
dtb = decoder.unused_data
if len(dtb) < 40 or dtb[:4] != bytes.fromhex("d00dfeed"):
    raise SystemExit("Missing appended flattened device tree")
dtb_size = struct.unpack_from(">I", dtb, 4)[0]
if dtb_size > len(dtb) or ("xiaomi,beryllium-" + args.panel).encode() + b"\0" not in dtb[:dtb_size]:
    raise SystemExit("Wrong or truncated Poco F1 panel device tree")
if not args.boot_only:
    with rootfs.open("rb") as stream:
        stream.seek(1024)
        sb = stream.read(1024)
    if len(sb) != 1024 or sb[56:58] != bytes.fromhex("53ef"):
        raise SystemExit("system.img must be a raw ext4 filesystem")
    label = sb[120:136].split(b"\0", 1)[0].decode()
    if label != "NIXOS_SYSTEM":
        raise SystemExit("Rootfs label does not match the initrd configuration")
    block_size = 1024 << struct.unpack_from("<I", sb, 24)[0]
    blocks = struct.unpack_from("<I", sb, 4)[0]
    if struct.unpack_from("<I", sb, 96)[0] & 0x80:
        blocks |= struct.unpack_from("<I", sb, 336)[0] << 32
    if blocks * block_size > rootfs.stat().st_size:
        raise SystemExit("Truncated root filesystem")
def sha256(path):
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        while chunk := stream.read(4 * 1024 * 1024):
            digest.update(chunk)
    return digest.hexdigest()
hashes = {"boot.img": sha256(boot)}
rootfs_info = {}
if not args.boot_only:
    hashes["system.img"] = sha256(rootfs)
    rootfs_info = {"rootfsBytes": rootfs.stat().st_size, "rootfsLabel": label}
print(json.dumps({
    "kernelVersion": expected_version, "architecture": "aarch64", "panel": args.panel,
    "verifiedBuiltinDrivers": required_drivers,
    "bootBytes": len(data), "kernelBytes": kernel_size,
    "initrdBytes": ramdisk_size, "dtbBytes": dtb_size,
    **rootfs_info,
    "sha256": hashes,
}, indent=2))
