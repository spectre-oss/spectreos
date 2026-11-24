# USB and Wi-Fi SSH

The diagnostic boot image enables the RNDIS USB Ethernet gadget and supplies a runtime NetworkManager profile plus a supervised SSH service to the installed 0.5.5 filesystem. It does not reflash userdata. The sensor-DSP recovery workaround remains enabled. Real USB connectivity still requires phone/laptop testing.

## Install the boot update

Extract the entire release ZIP, connect the phone in FASTBOOT mode, and run `python3 install.py`. Type `UPDATE beryllium` exactly. Let the phone boot and keep its USB data cable connected.

## Configure the laptop

Set the new USB Ethernet interface to **172.16.42.2**, netmask **255.255.255.0** (/24), with **no gateway and no DNS**. The phone is **172.16.42.1**. This release uses static addresses, not DHCP.

On a Linux laptop with NetworkManager, identify the new Ethernet interface with `nmcli device status`, then replace `USB_INTERFACE` below with its actual name (often `enx...`):

```sh
sudo nmcli connection add type ethernet ifname USB_INTERFACE con-name spectre-phone \
  ipv4.method manual ipv4.addresses 172.16.42.2/24 ipv4.never-default yes ipv6.method disabled
sudo nmcli connection up spectre-phone
ssh spectre@172.16.42.1
```

Use the existing `spectre` password (initially `123456`, unless already changed). Confirm the host-key prompt for this directly connected phone. Run `passwd` after logging in to change the initial password. Root SSH login and SSH port forwarding are disabled in the diagnostic bridge; use `sudo` after logging in.

On Windows, configure the USB/RNDIS Ethernet adapter's IPv4 settings as above, then use `ssh spectre@172.16.42.1` in PowerShell. If the adapter does not appear, the host needs RNDIS support. This image does not provide a macOS-specific USB networking driver.

## Collect desktop failure logs

Run these commands **inside the SSH session**:

```sh
sudo systemctl --failed --no-pager
sudo systemctl status phosh display-manager spectre-usb-ssh --no-pager -l
sudo journalctl -b -u phosh -u display-manager --no-pager
sudo journalctl -b -k --no-pager | grep -Ei 'drm|adreno|gpu|firmware|error|fail'
```

To save the complete boot log on the laptop (the `-t` lets sudo prompt):

```sh
ssh -t spectre@172.16.42.1 'sudo journalctl -b --no-pager' > phone-boot.log
```

Review the log before sharing; it can contain identifiers or network names. If SSH reports connection refused, wait for userspace startup. If it times out, check the cable, USB adapter and static IP. Reconnect the cable if necessary. Do not erase userdata to troubleshoot connectivity.

## Implementation and lifetime

The boot initrd copies a static Dropbear binary, startup script, NetworkManager profile and systemd unit into `/run`, which survives switch_root. The service uses the installed system's password database, shell and tools. Its host key is generated on the phone under `/var/lib/spectre-usb-ssh`, never embedded in the image. It listens on IPv4 port 22; firewall exceptions permit usb0 from laptop address 172.16.42.2 and Wi-Fi through wlan0. Restart the diagnostic service after manually reloading the old system firewall.

This bridge is specific to the existing pinned 0.5.5 system. Its command dependencies were checked against that image. Future full images enable NixOS OpenSSH with password login for spectre, root login disabled, and firewall openings on usb0 and wlan0. The bridge skips startup when `/etc/ssh/sshd_config` exists. Switching from Dropbear to OpenSSH may produce a host-key change because they maintain separate keys.

Flashing the previous 0.5.7 boot image removes the runtime bridge on the next boot while keeping userdata. No permanent edits to the installed NixOS generation are needed.

### Recovery network initialization

The boot-only bridge explicitly brings `usb0` up and assigns `172.16.42.1/24`
after marking that interface unmanaged in NetworkManager. This avoids depending
on the old root filesystem loading an initrd-provided runtime connection profile.
Failures are reported to the journal and phone console, and the service retries.
SSH remains password-authenticated with root login disabled.
The host needs `172.16.42.2/24`; its interface name may change when U-Boot
hands control to Linux. No default route is required.

## Wi-Fi access

Once the phone joins Wi-Fi, use its DHCP address, currently:

```sh
ssh spectre@<phone-lan-ip>
```

The address may change with DHCP. IPv6 SSH is not enabled. Wi-Fi login was
verified from the build host against the phone's existing host key.

For the currently installed older image, the updated recovery script is also
stored persistently at `/var/lib/spectre-usb-ssh/start.sh`, selected by
`/etc/systemd/system.control/spectre-usb-ssh.service.d/50-wifi.conf`.
The override survives reboots and uses the boot image's Dropbear binary.
Remove this compatibility override when updating the recovery script via a
new boot image; new full images use the declarative OpenSSH configuration.
A service restart was tested; a full reboot was not performed.
