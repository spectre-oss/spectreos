# Spectre USB-controlled U-Boot (Poco F1, Tianma)

## What this build does

Power on → Xiaomi ABL → custom U-Boot → wait indefinitely for USB host input → explicit host command → Linux. It has no timed Linux boot. The package embeds the current Linux kernel, initrd and device tree as a SHA-256 checked FIT. `run spectre_continue` boots that payload and the existing userdata filesystem. `run spectre_replace` downloads a replacement FIT from the host into RAM and boots it; it does not save a new kernel to flash.

This is an experimental bootloader integration. Compilation and offline checks do not establish successful phone operation. Test before permanently installing. Keep Xiaomi fastboot, ABL/XBL, the partition layout and userdata intact. U-Boot's documented fastboot backend does not support UFS flashing, so this build does not offer that feature. Persistent changes use Xiaomi fastboot to update boot only.

## First test

1. Extract the complete distribution on the USB host.
2. Enter Xiaomi FASTBOOT (Power + Volume Down) and connect the phone.
3. Run `python3 install-uboot.py check` and `python3 install-uboot.py test`. This uses `fastboot boot`, not `fastboot flash`. If the stock bootloader refuses temporary boot, stop and capture that error rather than blindly flashing.
4. Configure the newly appearing USB Ethernet interface with static address **172.16.42.2/24**, no gateway/DNS. The phone uses **172.16.42.1**. On Linux the RNDIS host driver must match the running kernel. The known 7.2.9-running/7.2.11-installed mismatch on the build host requires repair or a host reboot before this can work.
5. Run `python3 uboot-usb.py console`. The expected U-Boot prompt is `=>`. It must remain waiting until you request boot.
6. Enter `run spectre_continue`. The USB bootloader console disconnects when Linux takes over; use the Linux USB SSH connection for subsequent logs.
7. Test reset, waiting without a cable, connecting later, and kernel replacement before installation.

This console uses unencrypted UDP on a direct USB link, not SSH. It grants bootloader command access to the connected host. Do not bridge this interface to another network. The host may need its firewall to allow UDP 6666, UDP 1069 and reply traffic for TFTP. This tool is intended for a Linux/Unix host.

## Commands from the host

```sh
python3 uboot-usb.py console
python3 uboot-usb.py continue
python3 uboot-usb.py replace --fit spectre.itb
```

The replacement server exposes only the specified FIT under the fixed name `spectre.itb` and accepts requests only from the configured phone IP. It runs on unprivileged UDP port 1069. Keep the client open while transferring/booting; Ctrl-C ends the client. Replacement FIT images must match this phone and use suitable non-overlapping load addresses. This build's FIT uses kernel 0xa0000000, DTB 0xa7000000, initrd 0xa8000000; these require hardware validation. FIT hashes detect damaged transfers, not malicious images. Never load an untrusted payload.

## Permanent installation and rollback

Only after USB control and Linux boot both work on the actual phone:

```sh
python3 install-uboot.py install --tested-on-phone
```

Type `INSTALL tested UBOOT beryllium` when asked. This replaces boot only; future power-ons enter the waiting U-Boot image. Holding Power + Volume Down still accesses Xiaomi fastboot. From there, restore direct Linux boot with:

```sh
python3 install-uboot.py rollback
```

Type `RESTORE beryllium`. The packaged rollback image is the exact Linux boot image used to create this bundle. No userdata wipe is involved.

## Rebuild

`make uboot-package JOBS=8 MAX_JOBS=2` fetches the pinned Git revision, builds through the project's pinned cross-Nixpkgs, wraps the current `result-boot-tianma`, and exports an immutable versioned directory under dist. The upstream checkout is ignored by the parent Git repository; configuration and packaging code are tracked. Build Linux with `make boot-image` first if updating its payload. Run `make test` for offline checks.

Sources: [Qualcomm phones](https://docs.u-boot.org/en/latest/board/qualcomm/phones.html), [Qualcomm boot-image layout](https://docs.u-boot.org/en/latest/board/qualcomm/board.html), [network console](https://docs.u-boot.org/en/latest/usage/netconsole.html).
