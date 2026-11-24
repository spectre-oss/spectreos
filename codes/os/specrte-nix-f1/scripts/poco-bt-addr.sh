#!/bin/sh
set -eu

for attempt in $(seq 1 30); do
    [ -d /sys/class/bluetooth/hci0 ] && break
    sleep 0.5
done
[ -d /sys/class/bluetooth/hci0 ] || exit 0

config=$(btmgmt --index 0 config)
case "$config" in
    *"missing options:"*"public-address"*)
        # Use a stable local unicast address even when NetworkManager randomizes wlan0.
        digest=$(sha256sum /etc/machine-id)
        address=$(printf '%s\n' "$digest" | cut -c1-10 | sed 's/../:&/g')
        btmgmt --index 0 public-addr "02$address"
        ;;
esac
