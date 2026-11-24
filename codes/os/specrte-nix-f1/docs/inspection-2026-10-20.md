# USB inspection — 2026-10-20

## Reboot validation completed

On 2026-10-20 the existing fix was verified through an actual
`sudo systemctl reboot`, without flashing, rebuilding or erasing anything.

| Check | Result |
| --- | --- |
| Previous boot ID | `c345ca64-87fc-4adf-99ca-0776e9bf35bf` |
| New boot ID | `3ca2bf80-6d48-4711-b751-6f5eeb3bcf8d` |
| Active console after boot | `tty1` |
| Phosh state | `active`, `running`, `NRestarts=0` |
| Pre-start command | `/run/current-system/sw/bin/chvt 1`, exit status 0 |
| Loaded persistent drop-in | `/etc/systemd/system.control/phosh.service.d/10-spectre-vt.conf` |
| Login session | `spectre`, `seat0`, `tty1` |
| Recovery SSH | `spectre-usb-ssh.service` active after reboot |

Before reboot, the session had been running since the earlier test with no
restarts. The logs showed suspend/resume activity during the USB disappearance,
not another Phosh crash loop. The screenshot helper could not capture a native
image because `grim` was unavailable in its environment; the reboot checks
establish service/session startup rather than visual confirmation.

### Computer USB networking correction

The host lost the temporary `172.16.42.2/24` address whenever the USB interface
was recreated. A persistent NetworkManager profile named `spectre-poco-usb`
was installed on the computer:

```text
connection.interface-name=enp0s20f0u4
match.driver=rndis_host
connection.autoconnect=yes
connection.autoconnect-priority=100
ipv4.method=manual
ipv4.addresses=172.16.42.2/24
ipv4.never-default=yes
ipv6.method=disabled
```

The phone's host-side USB MAC changed from `xx:xx:xx:xx:xx:01` to
`xx:xx:xx:xx:xx:02` during reboot, so the profile deliberately has no MAC binding.
It is scoped to this USB interface/driver and does not install a default route.
If another USB port changes the interface name, update the profile accordingly.
The host-side profile was corrected/activated after this reboot; its automatic
reattachment on another reboot has not yet been tested.

To remove this host-only profile, run `nmcli connection delete spectre-poco-usb`.
The phone's persistent VT fix and declarative source were already present and
required no new edit for this successful reboot.

## Follow-up: VT workaround succeeds

**Later update:** a persistent override was successfully installed in the
writable `/etc/systemd/system.control/phosh.service.d/` directory and tested by
starting Phosh from tty2. See [operations](operations.md) for its exact contents,
verification, and removal. This supersedes the temporary-only limitation below;
the earlier attempt used the read-only Nix-managed unit directory.

The active console was `tty2`. The requested sequence succeeded:

```bash
sudo systemctl stop phosh
sudo chvt 1
sudo systemctl reset-failed phosh
sudo systemctl start phosh
```

After 20 seconds and again during a subsequent check, Phosh was running with
`NRestarts=0`. The active console was `tty1`, and logind reported the Wayland
session on VT 1 as `Active=yes`, `State=active`. Visual confirmation from the
phone owner is still needed. USB diagnostic SSH remained active.

Persistence was attempted using the owner-provided systemd drop-in, but directory
creation failed with:

```text
mkdir: cannot create directory '/etc/systemd/system/phosh.service.d': Read-only file system
```

No drop-in was written. `/run/current-system/sw/bin/chvt` exists and is executable.
The owner reports an equivalent declarative fix in `modules/phosh-console.nix`;
that source is not present in this computer's current project directory and has
not been inspected or deployed here. The successful VT switch currently applies
to this boot only. No remount, rebuild, reboot, flash, or erase was performed.

## Connection

The computer runs Arch Linux. Its phone USB interface is `enp0s20f0u4`,
identified by udev as `Mobile NixOS xiaomi-beryllium`, using `rndis_host`.
It initially had no IPv4 address, so traffic to the phone followed the Wi-Fi
default route. After the owner configured `172.16.42.2/24` on this interface,
SSH to `spectre@172.16.42.1` succeeded with the supplied password.

The active SSH service is `spectre-usb-ssh.service` (Spectre diagnostic SSH
over USB). `sshd.service` and `display-manager.service` do not exist in the
inspected system. Use the actual SSH service name for subsequent diagnostics.

## Verified system details

| Item | Observed value |
| --- | --- |
| Hostname | `specrte-nix-f1` (spelling as reported) |
| OS | NixOS 26.11 (Zokor) |
| Build ID | `26.11pre1031299.0bb7ec54c848` |
| Kernel | `7.1.0-rc1`, `#1-mobile-nixos`, aarch64 |
| Phosh / Phoc package versions | `0.54.0` as shown in service paths |
| Root filesystem | `/dev/disk/by-label/NIXOS_SYSTEM` |
| Root capacity | 53G total, 11G used, 43G available, 20% used |

The phone journal reported October 19 UTC during this inspection; the document
date uses the computer owner's October 20 local session date. Phone clock
accuracy has not been established.

## Phosh startup failure

`phosh.service` exists and repeatedly restarts, with exits roughly every ten
seconds. The initial restart counter was 38. A requested
`sudo systemctl restart phosh` succeeded as a systemd operation but did not
resolve the failure: the next process exited and restarted again.

The Phoc log reports:

```text
Seat opened with backend 'logind'
Waiting for a session to become active
Timeout waiting session to become active
failed to start a session
failed to add backend 'drm'
Failed to create server: Could not create backend
```

It also reports no `/run/seatd.sock`, then successfully falls back to logind.
The failure observed is the timeout waiting for an active session; its underlying
cause remains undetermined. A momentary `active (running)` status is insufficient
to establish recovery because the service uses `Restart=always`.

The service launches `phosh-session` as `spectre` on `/dev/tty1`, with
`PAMName=login`, and conflicts with `getty@tty1.service`.

## Other failures and scope

`boot-control.service` (Mark boot as successful) is failed. Its status shows
a Ruby traceback in `QualcommGPTBootControl`; the complete error and underlying
cause were not yet collected. No boot-control action was attempted.

Only the requested Phosh restart changed phone runtime state. Nothing was
flashed, erased, rebuilt, or changed in the persistent phone configuration.
