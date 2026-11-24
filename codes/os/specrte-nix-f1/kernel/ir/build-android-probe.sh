#!/bin/sh
# Build on the build host; Android NDK r30 was used for the recorded baseline.
set -eu
: "${NDK_ROOT:?Set NDK_ROOT to the Android NDK directory}"
base=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
out=${1:-.build/stock-android/ir-investigation/android-ir-probe}
mkdir -p "$(dirname -- "$out")"
"$NDK_ROOT/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android29-clang++" \
    -std=c++17 -O2 -Wall -Wextra -Werror -static-libstdc++ \
    "$base/android-ir-probe.cpp" -lcamera2ndk -lmediandk -landroid -ldl -o "$out"
