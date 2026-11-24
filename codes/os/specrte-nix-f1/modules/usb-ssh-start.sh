#!/usr/bin/env bash
# Runs after the installed system has populated its users and NetworkManager.
set -euo pipefail
export PATH=/run/current-system/sw/bin:/run/wrappers/bin

# The old root filesystem can ignore the runtime NetworkManager profile.
# Explicitly configure only the gadget interface, never Wi-Fi or a default route.
for attempt in {1..60}; do
    if ip link show dev usb0 >/dev/null 2>&1; then
        break
    fi
    sleep 1
done
ip link show dev usb0 >/dev/null
nmcli device set usb0 managed no
ip link set dev usb0 up
ip address replace 172.16.42.1/24 dev usb0
ip -4 address show dev usb0 | grep -q 'inet 172.16.42.1/24'

# Preserve this phone's identity across recovery-image updates.
key=/var/lib/spectre-usb-ssh/host_ed25519
install -d -m 700 /var/lib/spectre-usb-ssh
if [[ ! -s $key ]]; then
    /run/spectre-usb-ssh/dropbearkey -t ed25519 -f "$key"
fi

# The original image's firewall does not yet know about USB SSH.
# Limit the exception to the cable interface, its address and the laptop address.
rule=(-i usb0 -s 172.16.42.2/32 -d 172.16.42.1/32 -p tcp --dport 22 -j ACCEPT)
@iptables@ -C INPUT "${rule[@]}" 2>/dev/null || @iptables@ -I INPUT 1 "${rule[@]}"

# Permit authenticated Wi-Fi access as well; its IPv4 address is assigned by DHCP.
wifi_rule=(-i wlan0 -p tcp --dport 22 -j ACCEPT)
@iptables@ -C INPUT "${wifi_rule[@]}" 2>/dev/null || @iptables@ -I INPUT 1 "${wifi_rule[@]}"

# Nonempty system passwords are required. Deny root login and port forwarding.
exec /run/spectre-usb-ssh/dropbear -F -E -w -j -k \
    -p 0.0.0.0:22 -r "$key" -P /run/spectre-usb-ssh/dropbear.pid
