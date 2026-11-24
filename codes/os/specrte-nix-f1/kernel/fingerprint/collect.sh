#!/bin/sh
# Read-only baseline after a normal/fastboot boot. No trustlet commands.
set -eu
if [ "$(id -u)" != 0 ]; then
    echo 'Run as root to capture kernel and service diagnostics.' >&2
    exit 1
fi
out=${1:?Usage: collect.sh NEW_OUTPUT_DIRECTORY}
umask 077
mkdir "$out"
uname -a > "$out/uname.txt"
cat /proc/cmdline > "$out/cmdline.txt"
cat /proc/config.gz > "$out/config.gz"
cat /proc/uptime > "$out/uptime.txt"
cat /proc/sys/kernel/tainted > "$out/tainted.txt"
dmesg > "$out/kernel.txt"
journalctl -b --no-pager -o short-monotonic > "$out/system.txt"
ip -brief address > "$out/network.txt"
cat /proc/asound/cards > "$out/alsa.txt"
systemctl --failed --no-pager > "$out/failed-services.txt"
{
    for proc in /sys/class/remoteproc/remoteproc*; do
        [ -d "$proc" ] || continue
        printf '%s: ' "$proc"
        cat "$proc/name" "$proc/state"
    done
} > "$out/remoteproc.txt"
mkdir "$out/pstore"
cp -a /sys/fs/pstore/. "$out/pstore/"
echo "Saved diagnostics to $out"
