#!/bin/sh
set -eu
# Builds files only. No SSH, adb, fastboot, device access or boot commands.
base=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
if [ "$#" -lt 2 ]; then
    echo "usage: $0 MIUI_ROM_ZIP STATIC_PROBE [GFBER_IMAGE] [CMNLIB_DIRECTORY]" >&2
    exit 2
fi
out=.build/fingerprint/stock-kernel
mkdir -p "$out"
${CC:-aarch64-linux-gnu-gcc} -static -O2 -Wall -Wextra -Werror "$base/init.c" -o "$out/init"
if [ "$#" -gt 3 ]; then
    python3 "$base/pack.py" --rom "$1" --init "$out/init" --probe "$2" --gfber "$3" --cmnlib-dir "$4" --output "$out"
elif [ "$#" -gt 2 ]; then
    python3 "$base/pack.py" --rom "$1" --init "$out/init" --probe "$2" --gfber "$3" --output "$out"
else
    python3 "$base/pack.py" --rom "$1" --init "$out/init" --probe "$2" --output "$out"
fi
