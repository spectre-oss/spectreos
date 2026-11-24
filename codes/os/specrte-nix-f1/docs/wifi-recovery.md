# Wi-Fi recovery investigation — 25 October 2026

The running phone reports Linux 7.1.0-rc1 and ath10k SNOC WCN3990 hw1.0.
At 21:23:42 UTC wpa_supplicant logged a WNM preferred-list notification.
At 21:24:38 UTC the phone disassociated (reason 1); at 21:24:41 UTC
pairwise-key removal timed out with -110. The AP rejected authentication
with status 37; association succeeded at 21:25:02 UTC. This happened even
with the profile pinned to the 2.4 GHz BSSID and power saving disabled.
Roaming avoidance alone therefore did not resolve the disconnects.

## Kernel change

`kernel/patches/0004-ath10k-wcn3990-pairwise-key-removal.patch` adapts
David Heidelberg's RFC, restricted to WCN3990:
https://www.mail-archive.com/linux-kernel@vger.kernel.org/msg2657311.html

WCN3990 firmware does not acknowledge pairwise cipher-none key deletion.
Skip only that completion wait after a successful command send. Retain
installation and group-key acknowledgement waits and propagate send errors.
The patch is wired into `kernel/default.nix` and dry-runs against the exact
archived source. It compiled successfully and was installed after explicit user authorization.
Post-reboot validation is pending: USB RNDIS returned but SSH was unreachable. It addresses
the three-second teardown delay, not the original AP disassociation.

Build context correction: the user confirmed this session runs on the build
server itself (<build-host-ip>). The earlier SSH/sync attempts incorrectly treated
it as a separate workstation. Build directly here with `make kernel`, using
the existing workspace Nix store. Set TMPDIR to `.build/wifi-tmp` because
host `/tmp` is full. The workstation sync helper is not needed on this host.

## Live settings

The <wifi-ssid> profile originally had no BSSID or band restriction and power
saving set to default. The initial 2.4 GHz BSSID lock failed the longer test.
It was replaced with band `a` (5 GHz), no BSSID restriction, and powersave 2.
This is a mitigation under test, motivated by the router steering requests;
it is not proof that the AP caused every prior failure.

To restore the original connection settings:

```sh
sudo nmcli connection modify <wifi-ssid> 802-11-wireless.bssid "" 802-11-wireless.band "" 802-11-wireless.powersave 0
sudo nmcli connection up <wifi-ssid>
```

The user's display idle-delay was changed from 0 to 60 seconds. Backlight
bl_power was verified as 4 (off), and diagnostics use USB SSH.

## Validation still required

Build the patched kernel and a boot image on the configured build server.
After a separately authorized deployment, test repeated disconnect/reconnect,
key negotiation, screen-off idle, and router steering for at least an hour.
Check both packet loss and kernel/supplicant logs; absence of a key timeout
alone does not establish that the disconnect bug is fixed.

At 22:04 UTC the live profile was confirmed on 5 GHz channel 52, with
10/10 router pings received and backlight still off. This short check is not
a long-duration stability result. Documentation vector update was attempted
but failed because its local project/index is not registered.

## Installation attempt

The user authorized installation, then requested doing it without fastboot.
The patched kernel was wrapped with the backed-up phone boot image's original
initrd, DTB and command line. The only configuration addition relative to the
running kernel is CONFIG_VIDEO_AK7375=y, already in project source.

Artifacts: `artifacts/wifi-fix/`. The 64 MiB original boot partition backup
SHA-256 is 4573f7ac5a8eca19ee2fd49515d232e8d3542463afeb49d590ec00dca7d5e4eb.
The new boot.img is 50,802,688 bytes, SHA-256
b9011381868e9147d0cc4d1499f58533453a4dcff6bf368450c2fb75d543341b.
Image inspection passed; the image was written to /dev/disk/by-partlabel/boot
with fsync and compared byte-for-byte before normal reboot. No userdata write.
USB RNDIS reappeared, but initial post-reboot USB and Wi-Fi SSH attempts timed
out. Do not report installation as operationally verified until access returns.
