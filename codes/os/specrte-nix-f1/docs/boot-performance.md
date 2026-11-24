# Boot performance investigation — 26 October 2026

Baseline normal reboots: 24.061, 23.907 and 23.915 seconds reported by
systemd-analyze. Approximately 18 seconds before systemd, 6 seconds userspace.
Graphical target is reached roughly 3.6–3.9 seconds into userspace; that is
not a measurement of when the lock screen is visibly interactive.

Use dmesg timestamps for early boot: journal replay timestamps incorrectly
cluster early kernel messages at about 19 seconds. Actual dmesg shows:

- 0.550972: IPA driver initialized.
- 11.647949: freeing 33,912 KiB initrd memory.
- 13.121538: running /init.
- Stage-1 starts around 13.655 seconds; switches root about 3.46 seconds later.

This isolates an approximately 11-second gap around initrd unpacking, but
initcall tracing would be required to rule out all concurrent driver work.
The XZ ramdisk is 34,728,220 bytes, unpacking to 152,222,208 bytes.

## Rejected compression experiment

Recompressed identical ramdisk contents using gzip level 6. Kernel supports
CONFIG_RD_GZIP=y. Boot image retained exactly the working kernel, DTB and
command line, passed image inspection and fit the partition. Image size
64,094,208 bytes; SHA-256
78e9e0c55e56ff680840d325e1108567a720f1f977953d6fd8af4852648a62ad.
Artifacts are under artifacts/fast-boot. This image DID NOT boot to Linux:
the device appeared in fastboot. Exact rejection/failure cause is unconfirmed.
Do not install it or enable gzip in the production configuration.

Restored artifacts/wifi-fix/boot.img via fastboot (serial <device-serial>, product
beryllium), then normal reboot returned USB SSH, Phosh and Wi-Fi, with zero
failed system units. Restored boot time: 24.176 seconds. Screen off verified.
The experimental Nix compression override was removed. No startup speed
improvement is deployed. Earlier shutdown-order and boot-service fixes remain.

Further work should inspect bootloader ramdisk constraints and trace early
initcalls, then consider reducing initrd contents without removing necessary
firmware or recovery access. Size alone is not adequate boot validation.
