# Host-controlled Poco F1 boot

Requested flow: power on → custom U-Boot → wait indefinitely with USB networking/control → host chooses to continue or supplies replacement kernel/initrd/DTB → Linux starts. No timeout may silently start Linux.

Implementation targets:

- Pin and clone upstream U-Boot under `u-boot/`; track build configuration, patches and scripts in `bootloader/`, not generated binaries or the upstream checkout.
- Chainload through Xiaomi's existing bootloader using an Android boot image. Preserve original ABL/XBL and partition layout.
- Provide a USB network console with a fixed phone/host address and explicit boot command. This pre-OS console is distinct from Linux SSH and stops when Linux takes over.
- Carry a known Linux payload for the default continue path; provide a host-controlled RAM-only replacement path.
- Disable automatic boot and disk-writing fastboot commands in the custom recovery build where possible. Use stock fastboot for deliberate persistent updates until UFS write support is independently validated.
- Build versioned artifacts, preserve rollback images, and test temporary `fastboot boot` before permanent installation.
- Verify USB enumeration, console input/output, no automatic boot, default boot, replacement boot, unplug/replug behavior and cold-boot rollback on hardware.

Status: implemented and built as release 0.8.0; physical phone boot and USB control are still pending. The 51,449,856-byte image fits the 64 MiB boot partition. Offline checks validated the bundled FIT hashes, ARM64 U-Boot header, Tianma DTB, rollback identity, host transfer retries and partition-write boundaries. See [the operating guide](uboot.md). The host's running kernel (7.2.9) currently lacks matching installed USB network modules (7.2.11); a new bootloader does not repair that mismatch. USB network console hardware testing requires a working host driver.

## Hardware attempt on 2026-10-19

The phone enumerated in Xiaomi fastboot as serial <device-serial> after a temporary USB ACL grant. Read-only fastboot queries then stalled or returned inconsistent responses with both the host android-tools 37 client and Google's official 35.0.2 client. A USB connection reset did not restore reliable queries. The test installer stopped before sending the boot image; no partitions were flashed. Rebooting the host into its installed kernel and restarting the phone's fastboot session are the next hardware steps. This observation does not establish a defect in the unexecuted U-Boot image.

### First temporary phone boot: USB Ethernet registration

The phone accepted the RAM-only fastboot boot and displayed the U-Boot prompt.
A camera preview captured via ADB on a separate phone showed `Net: No ethernet found.`
The Qualcomm board startup did not call `usb_ether_init()`, although the gadget
driver was enabled. The build now patches board late initialization to register
the USB Ethernet device before network enumeration and report failures.
USB enumeration, remote commands, and Linux continuation still require a new
hardware test. The startup message describes configuration, not a live connection.

The 0.8.2 RAM-only test reached `RNDIS ready` and `eth0: usb_ether`.
The host still saw no USB device. Source inspection showed console_init_r runs
before board_late_init; netconsole startup returns ENONET without an Ethernet
device and console multiplexing silently skips it. Netconsole is now attached
in preboot, after network initialization. This second correction needs testing.
