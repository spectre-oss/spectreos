# Access and maintenance

## Connect over USB

Connect the phone with a USB cable that supports data. The expected USB network
configuration is:

- Phone: `172.16.42.1`
- Computer: `172.16.42.2/24` on the interface associated with the phone

Inspect the computer's network addresses before connecting:

```bash
ip -br address
ip route get 172.16.42.1
ssh spectre@172.16.42.1
```

If the computer lacks the expected address, configure the phone's USB network
interface using the computer's network manager. Identify that interface first;
its name is not yet documented.

## Connect over Wi-Fi

Use a network that can reach the phone:

```bash
ssh spectre@<phone-lan-ip>
```

`<phone-lan-ip>` is the last known address and may change. Check the phone's current
address or the router's DHCP leases if it no longer works. Keep the screen awake
if Wi-Fi SSH becomes unresponsive.

The initial password for `spectre` and sudo is `123456`. Enter it at the password
prompt. Change the default password after connecting, especially while SSH is
available over Wi-Fi:

```bash
passwd
```

Root SSH login is disabled; connect as `spectre` and use sudo when necessary.
On first connection, verify the SSH host-key fingerprint through a trusted
channel before accepting it. A verified fingerprint has not yet been recorded.

## Inspect the running system

Run these commands on the phone after connecting. They inspect state without
changing the configuration:

```bash
hostname
uname -a                                      # Running kernel
cat /etc/os-release                           # System version
ip -br address                                # Current network addresses
lsblk -o NAME,SIZE,FSTYPE,MOUNTPOINTS           # Storage layout
df -h                                         # Available storage
readlink -f /run/current-system                # Active NixOS system closure
ls -la /etc/nixos                             # Check for configuration files
systemctl --failed --no-pager                  # Failed system services
systemctl status phosh --no-pager              # Graphical interface
systemctl --user status mobi.phosh.OSK.service --no-pager
journalctl -b -p err --no-pager                 # Errors from this boot
```

Run `systemctl --user` as `spectre` within that user's session. If journal access
is restricted, use sudo for the relevant journal command. A missing
`/etc/nixos` does not establish that configuration sources are absent elsewhere.
Review captured output for private data before adding it to this documentation.

For more detail about the interface and keyboard:

```bash
journalctl -b -u phosh --no-pager
journalctl --user -b -u mobi.phosh.OSK.service --no-pager
systemctl cat phosh
systemctl --user cat mobi.phosh.OSK.service
```

## Troubleshoot SSH

Collect connection details from the computer:

```bash
ssh -vv -o ConnectTimeout=10 spectre@172.16.42.1
```

| Symptom | Checks |
| --- | --- |
| Connection refused | Confirm the address belongs to the phone; check that SSH is listening and the firewall allows it. |
| Timeout / no route | Check USB networking, the computer address and route, cable, current Wi-Fi address, and phone wake state. |
| Permission denied | Confirm the username and current password; the initial password may have changed. |
| Host-key warning | Verify the device identity and fingerprint before changing the saved host key. |
| Local SSH configuration ownership error | Repair the computer's SSH configuration permissions; the failure happens before connecting to the phone. |

If a local SSH configuration error prevents diagnosis, this command bypasses
the computer's SSH configuration files for a direct connection:

```bash
ssh -F /dev/null -o ConnectTimeout=10 spectre@172.16.42.1
```

With access to a terminal on the phone, inspect SSH and its listener:

```bash
systemctl status sshd --no-pager
sudo ss -ltnp
sudo journalctl -b -u sshd --no-pager
```

## Rebuilds, upgrades, and recovery

### Recover the Phosh display after reboot

**Installed fix (2026-10-20):** the phone now has a persistent drop-in at
`/etc/systemd/system.control/phosh.service.d/10-spectre-vt.conf`:

```ini
[Service]
ExecStartPre=+/run/current-system/sw/bin/chvt 1
```

Systemd's unit search path includes this writable directory ahead of the
Nix-managed unit directory. The root filesystem is writable ext4; the usual
`/etc/systemd/system` points into the read-only Nix store. After daemon-reload,
systemd confirmed the drop-in was loaded. Testing stopped Phosh, switched to
tty2, then started Phosh: the pre-start command returned success, tty1 became
active, and Phosh remained running with zero restarts after 20 seconds. USB SSH
remained active. An actual reboot was subsequently tested successfully on 2026-10-20; see the
inspection report for the fresh boot ID and startup result.

To remove this override, delete only the file above and run
`sudo systemctl daemon-reload`. Incorporate the equivalent change into the Nix
configuration for future image builds.

The verified startup problem was an active `tty2` while Phosh needed `tty1`.
Connect through USB SSH and run:

```bash
sudo chvt 1
sudo systemctl restart phosh
```

Check that `NRestarts` stays unchanged beyond the previous ten-second crash
interval:

```bash
cat /sys/class/tty/tty0/active
systemctl show phosh -p ActiveState -p SubState -p NRestarts
loginctl list-sessions
```

Keep `spectre-usb-ssh.service` running. The owner identifies the reliable
permanent fix as rebuilding and installing the image with the existing
`modules/phosh-console.nix` fix. That file is now available in this checkout
after source synchronization with the build server. Use the
project source documentation and
`infra/README.md` in the source project for the updated source/build layout;
no new image has been installed during this session.

The attempted system drop-in could not be written because its `/etc` path is
read-only. Do not enable a `spectre-activate-vt.service` user workaround: it
depends on passwordless sudo and its timing is not established. That user
service was checked and is absent; none was created or enabled.

The synchronized project now includes device-specific image inspection,
release installation and U-Boot tooling, plus their existing documentation.
Those installation procedures have not been exercised in this session. Build
only on the server, identify the exact release and panel, and follow a reviewed
installation/recovery procedure when deployment is requested.

## Inspection record

On **2026-10-20**, SSH to both supplied addresses returned `Connection refused`
on port 22. No authenticated inspection was possible. The inspection client also
encountered a local SSH configuration permissions error; using `-F /dev/null`
bypassed that local error for the connection attempts. Neither the phone's
runtime state nor the cause of the refusals was determined.
