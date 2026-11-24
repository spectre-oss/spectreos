#!/usr/bin/env python3
"""Check the actual kernel's board firmware requests against the pinned bundle."""
from pathlib import Path
import re
import tarfile

root = Path(__file__).resolve().parents[1]
with tarfile.open(root / "kernel/linux.tar.gz", "r|gz") as archive:
    for member in archive:
        if member.name.endswith("/arch/arm64/boot/dts/qcom/sdm845-xiaomi-beryllium-common.dtsi"):
            source = archive.extractfile(member).read().decode()
            break
    else:
        raise SystemExit("Poco F1 device tree is missing from kernel source")

requested = set(re.findall(r'"(qcom/sdm845/[^\"]+\.mbn)"', source))
if not requested:
    raise SystemExit("No Poco F1 remote-processor firmware requests found")
with tarfile.open(root / "firmware/beryllium.tar.gz") as archive:
    available = {"/".join(member.name.split("/")[1:]) for member in archive if member.isfile()}
missing = sorted(path for path in requested if "lib/firmware/" + path not in available)
sensor_root = "usr/share/qcom/sdm845/Xiaomi/beryllium"
sensor_files = [
    sensor_root + "/sensors/config/sdm845_tmd2725.json",
    sensor_root + "/sensors/config/config_list.txt",
]
missing += [path for path in sensor_files if path not in available]
if missing:
    raise SystemExit("Firmware missing for kernel: " + ", ".join(missing))
print(f"Poco F1 firmware paths: all {len(requested)} kernel requests satisfied")
print("Poco F1 sensor configuration: TMD2725 and configuration list present")
