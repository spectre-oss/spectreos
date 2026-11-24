#!/bin/sh
# Test the stock Goodix firmware with the experimental QSEECOM kernel. No flashing.
set -eu
[ "$(id -u)" = 0 ] || { echo 'Run as root.' >&2; exit 1; }
base=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
[ -x "$base/gfber-identify" ] || { echo 'gfber-identify binary missing' >&2; exit 1; }
exec 9>/run/poco-fingerprint.lock
flock -n 9 || { echo "Another fingerprint test is running" >&2; exit 1; }
# Keep the read-only mount and lookup tree in /run for this boot. The lock
# above serializes initialization and all users of the sensor.
cache=/run/spectre-goodix-firmware
modem="$cache/modem"
combined="$cache/firmware"
firmware_path=/sys/module/firmware_class/parameters/path
previous=$(cat "$firmware_path")
install -d -m 0700 "$cache"
if [ ! -d "$combined" ]; then
    mkdir -p "$modem"
    mountpoint -q "$modem" || mount -o ro /dev/disk/by-partlabel/modem "$modem"
    staging=$(mktemp -d "$cache/preparing.XXXXXX")
    trap 'rm -rf "$staging"' EXIT
    for source in /run/current-system/firmware "$modem/image"; do
        [ -d "$source" ] || exit 1
        for file in "$source"/*; do
            [ -e "$file" ] || continue
            dest="$staging/${file##*/}"
            [ -e "$dest" ] || ln -s "$file" "$dest"
        done
    done
    mv "$staging" "$combined"
    trap - EXIT
fi
restore_path() {
    if [ "$(cat "$firmware_path")" = "$combined" ]; then
        printf '%s\n' "$previous" > "$firmware_path"
    fi
}
trap restore_path EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
printf '%s\n' "$combined" > "$firmware_path"
"$base/gfber-identify" "$@"
