#!/bin/sh
# Native phone entry point; no Python, Android services, root or camera device.
set -eu
bundle=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
(cd "$bundle" && sha256sum --status -c SHA256SUMS) || {
    echo 'Camera compatibility bundle failed integrity check' >&2
    exit 1
}
operation=${1:-}
case "$operation" in
    probe|hdr) shift ;;
    *) echo 'Usage: run probe | run hdr nv12|nv21 W H IN0 IN1 IN2 NEW_OUT AE0 AE1 AE2 AE3 AE4 AE5' >&2; exit 2 ;;
esac
exec unshare --user --map-root-user --pid --fork --kill-child \
    env -i LD_LIBRARY_PATH="$bundle/lib" \
    "$bundle/linker" "$bundle/xiaomi-camera-helper" "$operation" \
    "$bundle/lib/libarcsoft_high_dynamic_range.so" "$@"
