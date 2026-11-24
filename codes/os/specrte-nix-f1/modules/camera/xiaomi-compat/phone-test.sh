#!/bin/sh
# Run against generated fixtures only. Uses no camera device or privileged API.
set -eu
if [ "$#" -ne 4 ]; then
    echo 'Usage: phone-test.sh BUNDLE FIXTURE_DIRECTORY WIDTH HEIGHT' >&2
    exit 2
fi
bundle=$1
fixtures=$2
width=$3
height=$4
work=$(mktemp -d)
trap 'rm -rf -- "$work"' EXIT HUP INT TERM
sha256sum "$fixtures"/frame-0.yuv "$fixtures"/frame-1.yuv "$fixtures"/frame-2.yuv > "$work/before"
"$bundle/run" probe
for iteration in 1 2; do
    "$bundle/run" hdr nv12 "$width" "$height" \
        "$fixtures/frame-0.yuv" "$fixtures/frame-1.yuv" "$fixtures/frame-2.yuv" \
        "$work/output-$iteration.yuv" 50 1 1 1 8 4
done
cmp "$work/output-1.yuv" "$work/output-2.yuv"
test "$(wc -c < "$work/output-1.yuv")" -eq "$((width * height * 3 / 2))"
sha256sum -c "$work/before"
sha256sum "$work/output-1.yuv"
echo 'PASS: original HDR engine, exact size, repeatability, input preservation'
