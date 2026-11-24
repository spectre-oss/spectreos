#!/bin/sh
set -eu
base=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
out=.build/fingerprint/diagnostics
mkdir -p "$out/stock-uapi"
url=https://raw.githubusercontent.com/LineageOS/android_kernel_xiaomi_sdm845/aa8adfe9bf212c6e93238a86980b4024da4f819a
curl -fsSL "$url/include/uapi/linux/qseecom.h" -o "$out/stock-uapi/vendor-qseecom-uapi.h"
curl -fsSL "$url/drivers/staging/android/uapi/ion.h" -o "$out/stock-uapi/vendor-ion-uapi.h"
${CC:-aarch64-linux-gnu-gcc} -static -O2 -Wall -Wextra -Werror -I"$out/stock-uapi" "$base/stock-probe.c" -o "$out/stock-probe"
