# Spectre Nix system project status

Date: **2026-10-21**. Domain overview for the installed Poco F1 (xiaomi-beryllium) system, the Tianma-oriented Mobile NixOS port, and the camera, boot, audio, access, and build records supplied with this checkpoint.

This file summarizes the latest dated records; no new phone inspection was performed for this documentation audit. Dated inspection and incident notes stay historical records of what was true at the time they were written. A configured driver, a successful compilation, or a userspace file from the stock camera stack is not a claim that the corresponding hardware function works.

## Running system

| Item | Verified value |
| --- | --- |
| Device | Xiaomi POCO F1 / POCOPHONE F1, codename `beryllium` |
| Panel used by this port | Tianma NT36672A, from local phone records. A model match does not detect an EBBG panel. |
| Hostname | `specrte-nix-f1` |
| OS | NixOS 26.11 (Zokor), build `26.11pre1031299.0bb7ec54c848` |
| NixOS system closure | `6lqp1f4iwpn6c3ihi7gx99j98vg4wb18-nixos-system-specrte-nix-f1-26.11pre1031299.0bb7ec54c848` |
| Kernel on the phone | `7.1.0-rc1`, `#1-mobile-nixos`, aarch64. `CONFIG_VIDEO_AK7375` is **disabled** in the running kernel. |
| Desktop | Phosh / Phoc `0.54.0` on tty1, active, `NRestarts=0` on the 2026-10-21 checks |
| Login | User `spectre` on `seat0`. Root SSH login is disabled. See the existing access notes for login configuration. |
| Root filesystem | `/dev/disk/by-label/NIXOS_SYSTEM`. On 2026-10-20: 53G total, 11G used, 43G available, 20% used. |
| Camera application | Spectre Camera **0.4.4** at `/nix/store/jqyydjqykllfcxlhjcpp8xwbkqjhp0n3-spectre-camera-aarch64-unknown-linux-gnu-0.4.4` by the end of this checkpoint. An earlier health check the same day still showed **0.4.3**. |

The phone journal during the 2026-10-20 inspection showed 19 October UTC while the session date was 20 October locally. On 2026-10-21, clock adjustments made wall-clock timestamps across boots unreliable as elapsed-time measurements.

Phosh on tty1, USB SSH, Wi-Fi address assignment, the lock screen, keyboard service activity, a 63% battery indicator, and no failed **user** services were observed on 2026-10-21. Those checks do not mean every hardware function is healthy.

## Access

USB SSH to `spectre@172.16.42.1` worked on this boot. The laptop static address is `172.16.42.2/24` with no gateway and no DNS. On the 2026-10-20 inspection computer the USB interface was `enp0s20f0u4` (`rndis_host`), with a persistent NetworkManager profile `spectre-poco-usb` and no MAC binding. Automatic reattachment of that host profile on a later reboot was still untested after it was corrected. Another USB port can change the interface name.

Wi-Fi on the phone obtained `<phone-lan-ip>` on 2026-10-21. DHCP addresses can change. IPv6 SSH is not enabled.

The installed image's diagnostic SSH is `spectre-usb-ssh.service` (Dropbear copied into `/run` by the boot initrd, password database from the installed system). `sshd.service` was absent on 2026-10-20. Future full images use NixOS OpenSSH and the Dropbear bridge skips startup when `/etc/ssh/sshd_config` exists. Switching stacks can change the host key. A verified fingerprint is not recorded in this packet.

`lsusb` on this boot named USB ID `18d1:d001` "Nexus 4 (fastboot)". Udev descriptors identified `Mobile NixOS xiaomi-beryllium`, serial `0<INITIAL_PASSWORD_REDACTED>789`, with CDC networking interfaces. `fastboot devices` found no fastboot transport. Fastboot must be identified from the transport and descriptors, not from that vendor/product pair alone.

## Boot, power, and U-Boot

`sudo reboot bootloader` during camera work was intended to prepare a RAM-only Sunny autofocus-kernel test. The phone returned to the installed Linux system. No fastboot image was transferred and no partition was written. The test kernel was not executed. Pstore was empty.

Boot `da294d7192f74a5b902de0f65399c172` shows logind handling a short power-key press at 07:38:41 (`Powering off`) and systemd completing power-off at 07:38:42, after which the journal stopped. The next boot, `d0bdfe6641b448778095fe917485c8b3`, contains `androidboot.mode=charger` while USB is attached. That supports an early short-button shutdown followed by a charger-mode boot. It does not establish a kernel panic.

The installed logind default was `HandlePowerKey=poweroff`. Phosh and GNOME media keys acquire `handle-power-key` inhibitors only after they are up, so an earlier press used logind's default. The live policy is now `HandlePowerKey=ignore` only. Phosh short-press handling remains once its inhibitor exists. The shutdown menu and hardware long-press behavior are unchanged.

Live file `/run/systemd/logind.conf.d/60-spectre-power-key.conf` was loaded with `systemctl reload systemd-logind`. The D-Bus property `HandlePowerKey` reads `ignore`. A pre-start drop-in at `/etc/systemd/system.control/systemd-logind.service.d/60-spectre-power-key.conf` recreates that runtime file because ordinary `/etc` is read-only. The service manager parsed both pre-start commands. No logind or Phosh restart and no second reboot were required for the live policy. Persistence across another physical reboot has not been tested.

`modules/phosh-console.nix` sets the equivalent NixOS logind option and copies the configuration into stage-1 runtime storage for future boot-only updates. Build-host Nix evaluation passed. That evaluation is not a phone reboot test.

Phosh tty1 startup itself was already verified across a real reboot on 2026-10-20 (boot ID `3ca2bf80-6d48-4711-b751-6f5eeb3bcf8d`, drop-in `/etc/systemd/system.control/phosh.service.d/10-spectre-vt.conf`, `ExecStartPre=+/run/current-system/sw/bin/chvt 1`). The 2026-10-21 session again shows Phosh active on tty1 with zero restarts and a lock-screen screenshot.

Custom U-Boot (release line 0.8.0, RAM-only tests through 0.8.2) is **not** the installed boot path. Offline checks and one temporary `fastboot boot` exist; USB host control of the U-Boot console is still not demonstrated. On 2026-10-19 the phone enumerated in Xiaomi fastboot as serial `<device-serial>`, then read-only queries stalled and the installer stopped before sending an image. The 0.8.2 RAM-only test reached `RNDIS ready` and `eth0: usb_ether` while the host still saw no USB device. See [uboot-goal.md](uboot-goal.md) and [uboot.md](uboot.md). Stock ABL/XBL, the partition layout, and userdata were left intact. U-Boot in this port does not flash UFS; persistent boot updates stay on Xiaomi fastboot.

The first 0.5.5 phone boot hit repeated SLPI FastRPC failures. Recovery image 0.5.7 disables SLPI in both panel DTBs (boot image 39,608,320 bytes, SHA-256 `10e098bfe6aeb390a1ad1c662f74c63413aa30a74d1e0f17d73f6a15fb0becea`). Later sessions prove the **installed** system reaches Phosh and USB SSH. This packet does not restate whether the running DTB is that 0.5.7 image or whether SLPI is still disabled. SLPI-backed sensors stay unverified. Details: [boot-recovery.md](boot-recovery.md).

## Failed units and warnings (2026-10-21)

Two system units failed. They were recorded, not fixed, by the camera or power-key work.

- `boot-control.service`: the Qualcomm A/B helper expects `androidboot.slot_suffix` and calls `split` on nil when the argument is absent. This port's device definition is non-A/B. Do not run a partition-writing boot-control action to hide the failure.
- `msm-modem-uim-selection.service`: generated QMI input has empty `slot` and `aid`. ModemManager then cannot check the primary SIM session.

Other recorded warnings: early TAS2559 firmware/routing errors, missing Bluetooth firmware, background image decoding, a missing Hunspell dictionary, desktop schemas, and portal integration. Audio sinks appeared later, so the startup TAS2559 lines do not by themselves establish a current speaker failure. Earlier audible speaker tests remain recorded in [audio.md](audio.md).

Local evidence: `.code-sync/pipeline/reboot-errors.txt`, `reboot-focused.txt`, `reboot-transition.txt`, `reboot-screen.png`.

## Camera (Sunny IMX363)

See [camera project status](camera-project-status.md) for the reviewed deployment, calibration, tests and remaining gates. The application is 0.4.4; autofocus is built but unbooted and HDR is not integrated into the Qt shutter.

## Audio

`modules/audio.nix` points vendor UCM at card `hw:F1`, hides snd-aloop, raises HiFi speaker priority, and constrains the tested speaker format to S16_LE at 48 kHz. The user confirmed an audible test tone after that constraint. Early-boot `tas2559_uCDSP.bin` is an initrd firmware change. Do not restart the live ADSP through remoteproc sysfs; that hung the phone and dropped USB SSH. A temporary fastboot boot of a known-working image recovered Phosh without flashing.

On an older running image, compatibility files live under `~/.local/share/spectre-alsa/ucm2`, user service drop-ins, and `~/.config/wireplumber/wireplumber.conf.d/51-beryllium-audio.conf`. Saved WirePlumber state can restore Earpiece (amplifier configuration 3) instead of Speaker (route 2, configuration 4, `chl=3`). Profile index 1 on this pinned UCM is `HiFi (Headphones, Mic, Speaker)`. Object IDs change every restart. Rebuilding an image does not clear saved user state. Do not force the speaker route continuously.

Media uses virtual sink `Phone Speaker` (`spectre_bottom_speaker`): stereo mixed to mono and looped only to physical front-right (bottom loudspeaker). Front-left is the quiet top receiver and is not linked. The user confirmed isolated 440 Hz and 660 Hz tones both came from the bottom speaker. `spectre-media-output.service` selects that sink once at graphical-session startup after CallAudio's initial PulseAudio routing. It does not force routes during calls or headphone use. A full phone reboot of this service has not been tested. Calls, headphones, microphones, and suspend need their own tests.

Remove user-local UCM, WirePlumber, PipeWire (`52-beryllium-speaker.conf`), and media-output compatibility files when installing a full image that already contains the same modules, so two copies are not loaded. Early-boot firmware still requires a boot-image update.

## Desktop, fonts, Nix on the phone

- Terminal and Phosh Wi-Fi password keyboard: `GTK_IM_MODULE=wayland` for Console and for `mobi.phosh.Shell.service`. Screenshots confirmed both keyboards on the existing image via user-local overrides. A new system image is still required before those source changes are in downloadable releases. See [phosh.md](phosh.md).
- Notch: installed Phosh reserves 90 physical pixels (logical height rounded up). A boot-only image does not replace the shell.
- Shabnam 5.0.1 is prepended for `lang=fa` only. The phone returns `Shabnam` for `sans-serif:lang=fa`. A Persian screenshot comparison is still pending. Remove `~/.config/fontconfig/conf.d/99-spectre-persian.conf` after a declarative image. See [fonts.md](fonts.md).
- Pinned Nixpkgs is on `NIX_PATH` as `nixpkgs` and alias `nixpkg`. On the phone, `nix-shell -p htop --run "htop --version"` returned `htop 3.5.1`. Remove `~/.config/spectre/nix-path.sh` after migrating to the declarative search path. See [nix-search-path.md](nix-search-path.md).
- Command-not-found on the phone suggests `htop` and `htop-vim`; an unknown command exits 127 without a missing-database warning. The declarative image has not been rebuilt for that change. See [command-not-found.md](command-not-found.md).
- Phone debug agent: on-demand SSH JSON, uinput, Grim screenshots. It refuses to force the OSK while the graphical session is locked. Live checks covered PNG capture, swipe-to-passcode, mouse buttons, and two-finger contacts. See [phone-debug.md](phone-debug.md).

## Installers and build host

Laptop installers use Python 3.9+ and fastboot. Full-system `install.py` requires typing `ERASE beryllium`, writes userdata before boot, and erases Android userdata. Boot-only recovery requires `UPDATE beryllium` and does not write userdata. U-Boot permanent install is a separate tool and stays blocked until USB control and Linux continuation both work on the phone. Entering fastboot with Volume Down + Power is still the documented installer path. The 2026-10-21 `reboot bootloader` result shows that command is not a reliable way to reach fastboot on the running system, and `18d1:d001` can be the Linux gadget.

Automated installer tests use a fake fastboot. A successful `install.py` run against this phone is not recorded here. The phone is nevertheless running an installed NixOS userspace, so some earlier flash of userdata and boot happened before these October sessions. Sessions on 2026-10-20 and 2026-10-21 did not flash partitions.

The build supervisor and DeltaChat notifier are build-host user services, not phone services. They poll every ten minutes, call Grok with `--model grok-4.7 --reasoning-effort low`, and do not kill a suspected hung build. See [build-supervisor.md](build-supervisor.md), [grok-build-task.md](grok-build-task.md), and [build-alert.md](build-alert.md). Image build completion and fresh-checkout reproducibility stay open in [requirements.md](requirements.md). The kernel pin recorded for this port is Linux 7.1.0-rc1 from the SDM845-mainline tree. That pin is not a dated claim that it is the newest upstream release.

## Document map

| Topic | Document |
| --- | --- |
| This status | `system-project-status.md` |
| Short system table | [system.md](system.md) |
| SSH and maintenance | [operations.md](operations.md), [usb-ssh.md](usb-ssh.md) |
| 2026-10-20 inspection | [inspection-2026-10-20.md](inspection-2026-10-20.md) |
| 2026-10-21 power key | [reboot-2026-10-21.md](reboot-2026-10-21.md) |
| Hardware matrix | [hardware.md](hardware.md) |
| Requirements | [requirements.md](requirements.md) |
| Phosh | [phosh.md](phosh.md) |
| Audio | [audio.md](audio.md) |
| U-Boot | [uboot.md](uboot.md), [uboot-goal.md](uboot-goal.md) |
| SLPI recovery boot | [boot-recovery.md](boot-recovery.md) |
| Laptop full install | `codes/spectre-nix/scripts/INSTALL.md` |
| Boot-only install | `codes/spectre-nix/scripts/INSTALL-BOOT.md` |
