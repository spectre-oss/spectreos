#!/usr/bin/env python3
"""Wrap U-Boot and a verified Linux FIT using the proven Android v0 layout."""
import argparse
import gzip
import hashlib
import json
from pathlib import Path
import struct
import subprocess
import zlib


def split_boot(data):
    if data[:8] != b'ANDROID!' or len(data) < 4096:
        raise ValueError('Expected Android boot image')
    kernel_size, ramdisk_size = (struct.unpack_from('<I', data, o)[0] for o in (8, 16))
    page = struct.unpack_from('<I', data, 36)[0]
    if page != 4096 or struct.unpack_from('<I', data, 40)[0] != 0:
        raise ValueError('Only the verified legacy v0/4096-byte layout is supported')
    end = page + align(kernel_size, page)
    kernel = data[page:page + kernel_size]
    ramdisk = data[end:end + ramdisk_size]
    if len(ramdisk) != ramdisk_size or not kernel_size or not ramdisk_size:
        raise ValueError('Truncated or empty boot payload')
    decoder = zlib.decompressobj(31)
    raw = decoder.decompress(kernel) + decoder.flush()
    dtb = decoder.unused_data
    if not decoder.eof or raw[56:60] != b'ARM\x64' or dtb[:4] != b'\xd0\x0d\xfe\xed':
        raise ValueError('Missing ARM64 kernel or appended DTB')
    cmdline = (data[64:576].split(b'\0')[0] + data[608:1632].split(b'\0')[0]).decode()
    return kernel[:-len(dtb)], ramdisk, dtb, cmdline


def align(n, page=4096):
    return (n + page - 1) // page * page


def wrap_boot(template, kernel, ramdisk):
    header = bytearray(template[:4096])
    for offset, value in ((8, len(kernel)), (16, len(ramdisk)), (24, 0), (40, 0)):
        struct.pack_into('<I', header, offset, value)
    digest = hashlib.sha1()
    for blob in (kernel, ramdisk, b''):
        digest.update(blob)
        digest.update(struct.pack('<I', len(blob)))
    header[576:608] = digest.digest() + bytes(12)
    image = bytes(header) + kernel + bytes(align(len(kernel))-len(kernel))
    image += ramdisk + bytes(align(len(ramdisk))-len(ramdisk))
    if len(image) > 64 * 1024 * 1024:
        raise ValueError('U-Boot plus Linux payload exceeds Poco F1 boot partition')
    return image


FIT = '''/dts-v1/;
/ {
    description = "Spectre Poco F1 Linux payload";
    #address-cells = <1>;
    images {
        kernel {
            description = "Pinned ARM64 Linux";
            data = /incbin/("Image.gz");
            type = "kernel"; arch = "arm64"; os = "linux";
            compression = "gzip"; load = <0xa0000000>; entry = <0xa0000000>;
            hash { algo = "sha256"; };
        };
        ramdisk {
            description = "Mobile NixOS initrd";
            data = /incbin/("initrd");
            type = "ramdisk"; arch = "arm64"; os = "linux";
            compression = "none"; load = <0xa8000000>;
            hash { algo = "sha256"; };
        };
        fdt {
            description = "Poco F1 Tianma Linux DTB";
            data = /incbin/("linux.dtb");
            type = "flat_dt"; arch = "arm64";
            compression = "none"; load = <0xa7000000>;
            hash { algo = "sha256"; };
        };
    };
    configurations {
        default = "spectre";
        spectre {
            description = "Spectre Linux with existing userdata";
            kernel = "kernel"; ramdisk = "ramdisk"; fdt = "fdt";
        };
    };
};
'''


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--linux-boot', type=Path, required=True)
    parser.add_argument('--uboot', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=True)
    template = args.linux_boot.read_bytes()
    kernel, ramdisk, dtb, cmdline = split_boot(template)
    if b'xiaomi,beryllium-tianma\0' not in dtb:
        raise ValueError('This build currently supports only the confirmed Tianma panel')
    (out / 'Image.gz').write_bytes(kernel)
    (out / 'initrd').write_bytes(ramdisk)
    (out / 'linux.dtb').write_bytes(dtb)
    subprocess.run(['fdtput', '-t', 's', str(out / 'linux.dtb'), '/chosen', 'bootargs', cmdline], check=True)
    (out / 'payload.its').write_text(FIT)
    subprocess.run([str(args.uboot / 'bin/mkimage'), '-f', 'payload.its', 'spectre.itb'], cwd=out, check=True)
    uboot = args.uboot.joinpath('u-boot-nodtb.bin').read_bytes()
    if uboot[56:60] != b'ARM\x64':
        raise ValueError('U-Boot lacks Qualcomm-compatible ARM64 image header')
    uboot_dtb = (args.uboot / 'dtbs/sdm845-xiaomi-beryllium-tianma.dtb').read_bytes()
    image = wrap_boot(template, gzip.compress(uboot, mtime=0) + uboot_dtb, (out / 'spectre.itb').read_bytes())
    (out / 'uboot-boot.img').write_bytes(image)
    # The rollback image is byte-for-byte the Linux boot image we wrapped.
    (out / 'rollback-boot.img').write_bytes(template)
    (out / 'build-info.json').write_text(json.dumps({
        'linuxBootSha256': hashlib.sha256(template).hexdigest(),
        'ubootBootSha256': hashlib.sha256(image).hexdigest(),
        'bootBytes': len(image), 'hardwareTested': False,
        'bootPolicy': 'wait for explicit USB host command',
    }, indent=2) + '\n')
    # Keep only distributable payloads; no duplicate extracted Linux components.
    for name in ('Image.gz', 'initrd', 'linux.dtb', 'payload.its'):
        (out / name).unlink()


if __name__ == '__main__':
    main()
