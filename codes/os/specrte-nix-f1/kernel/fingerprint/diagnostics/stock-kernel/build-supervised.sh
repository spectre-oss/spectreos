#!/bin/sh
# Build only. No phone access. MUSL_LIB and MUSL_INCLUDE refer to a matching
# AArch64 musl static library/development pair, not host x86 libraries.
set -eu
: "${MUSL_LIB:?Set MUSL_LIB to AArch64 musl lib directory}"
: "${MUSL_INCLUDE:?Set MUSL_INCLUDE to matching musl include directory}"
if [ "$#" -ne 4 ]; then
 echo "usage: $0 STOCK_BOOT_IMAGE STATIC_PROBE GFBER_IMAGE CMNLIB_DIR" >&2
 exit 2
fi
base=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
out=.build/fingerprint/stock-supervised-build
mkdir -p "$out"
cc=${CC:-aarch64-linux-gnu-gcc}
compiler_include=$($cc -print-file-name=include)
$cc -static -no-pie -fno-pie -nostdinc -nostdlib \
 -isystem "$MUSL_INCLUDE" -isystem "$compiler_include" \
 -idirafter /usr/aarch64-linux-gnu/include \
 -O2 -Wall -Wextra -Werror -DDIAGNOSTIC_CHILD \
 "$MUSL_LIB/crt1.o" "$MUSL_LIB/crti.o" "$base/init.c" \
 -L"$MUSL_LIB" -Wl,--start-group -lc -lgcc -Wl,--end-group \
 "$MUSL_LIB/crtn.o" -o "$out/diagnostic"
$cc -nostdlib -static -no-pie -Wl,--build-id=none -Wl,-e,_start \
 "$base/supervisor.S" -o "$out/init"
python3 "$base/pack.py" --boot-image "$1" --init "$out/init" \
 --worker "$out/diagnostic" --probe "$2" --gfber "$3" --cmnlib-dir "$4" \
 --output "$out"
