# Historical fingerprint investigation

**Superseded conclusions:** the R4/R5 region-notification hypothesis was
rejected by the R3 control: first load fails, second load succeeds without
notification. The final source retains only the upstream transport and
LAB/IBB IRQ fix. See README.md for the current result. R3's allocation-flag
change was also ineffective and removed. The notes below record the
investigation chronologically and are not current implementation guidance.

# Poco F1 fingerprint bring-up

This experimental kernel can communicate with the phone's **physical Goodix
sensor** through its stock `gfber` trusted application. Vendor DETECT_SENSOR
and INIT commands succeed with power on and fail or return no OTP data with
power off. The sensor reports 88×108 dimensions. The ordinary kernel does not
enable this transport. Capture, enrollment, matching and desktop authentication
are **not implemented or verified** by this diagnostic.

Build on the configured build server:

```sh
nix-build --store ../nix-store \
  --option build-users-group '' --cores 12 --max-jobs 1 \
  kernel/fingerprint -o .build/roots/result-fingerprint-kernel
```

## Source provenance

`0001-qseecom-tee.patch` is the unmodified cumulative diff from
[`wrobelda/linux`](https://github.com/wrobelda/linux):

- base: `8cd9520d35a6c38db6567e97dd93b1f11f185dc6` (`v7.1-base`)
- tip: `4855f9e2f0a1e44e32404c7c4ce56e5052fd04ce` (`qcom-qseecom-tee`)
- SHA256: `6efd0b5fc73b6d0bad38c1f50d47077a4718169be6ab32fa0c8871b6f76fae74`

The files retain their upstream SPDX/copyright notices. The patch adds a TEE
interface for the command-based QSEECOM protocol, trusted-application loading,
and userspace listeners. The existing QCOMTEE driver serves a different
protocol. Enabling it alone does not provide access to the phone's fingerprint
trustlets. The patch's upstream hardware verification is on SM8250, not this
SDM845 phone; successful compilation does not establish runtime compatibility.

Research reference: [goodix-fp-spi-linux](https://github.com/wrobelda/goodix-fp-spi-linux/tree/4663eb9019f5721d62984d3a81e642c7ac8ef530).
Its Goodix client protocol is specific to `gfenu`; do not assume that its
commands or buffer sizes work with `gfber` or `fpcber`.

## Original-kernel baseline, 2026-11-03

- SSH to `poco-f1`: NixOS, Linux 7.1.0-rc1, Tianma panel.
- No fingerprint node, SPI master/device, or fingerprint character device.
- `CONFIG_MODULES=n`, `CONFIG_OF_OVERLAY=n`, `CONFIG_QCOM_QSEECOM=n`.
  Loading a module or overlay cannot supply the missing transport at runtime.
- Stock vendor/modem/persist partitions inspected read-only (ext4 mounted with
  `noload`). Both `fingerprint.fpc.so` and `fingerprint.goodix.so` are present;
  firmware includes both `fpcber` and `gfber`. This does not identify the sensor.
- TLMM GPIO 37 (active-low reset), 94 (active-high power enable), and 121
  (interrupt) are unclaimed. Reset is low; power is an input reading low.
- A GPIO character-device test enabled power, waited 10 ms, released reset,
  then applied three 5 ms reset pulses. IRQ remained low throughout. The test
  restored reset low and power low/input. This establishes GPIO access, not a
  responding fingerprint sensor or its vendor.
- `persist/data/fingerprint_version` contains four bytes `aa 51 3b 88`;
  meaning is unknown. No biometric templates were read or modified.

GPIO wiring comes from the [LineageOS Beryllium board description](https://github.com/LineageOS/android_kernel_xiaomi_sdm845/blob/aa8adfe9bf212c6e93238a86980b4024da4f819a/arch/arm64/boot/dts/qcom/beryllium-p0.dtsi).
Both vendor sensor drivers use these same lines and leave sensor communication
to secure firmware. An SPI node should not be enabled speculatively.

Local diagnostic evidence and the original boot backup are under
`.build/fingerprint/` (ignored, not synchronized). Credentials are not saved
there. Experimental kernel configuration enables QSEECOM and its TEE backend,
with the unrelated UEFI client disabled; existing camera/Wi-Fi patches remain.

## Packaging and test tools

`prepare.py BACKUP CONFIG_GZ KERNEL_DIRECTORY OUTPUT_DIRECTORY` verifies a
complete boot backup against the phone configuration, permits exactly the two
QSEECOM configuration additions, and writes kernel/initrd/DTB/boot-image files
with a checksum manifest. It preserves the backup's initrd and device tree.
The 2026-11-03 build and the project's boot-image inspector both passed.

The first kexec attempt loaded successfully (`/sys/kernel/kexec_loaded=1`),
but SSH did not return and the owner reported a frozen screen. No boot
partition was written. A power-cycle was requested to return to the original
kernel. This does not establish whether the freeze happened during shutdown,
the kexec handover, or the new kernel's startup. Runtime QSEECOM support was unverified at that point; subsequent fastboot
tests below supersede this initial result.

The laptop's USB audit subsequently confirmed that the experimental kernel
reached userspace with both QSEECOM options enabled. Normal reboot recovered
Wi-Fi, the physical ALSA card, modem and DSPs; the regulator Oops and memory
assignment failures did not recur. The Bluetooth firmware and regulatory.db
errors also occur on the original kernel. Do not use kexec for the next test:
use the laptop's USB connection and a temporary fastboot boot instead.

The reported fault was `qcom_labibb_ocp_isr+0x30` with `x0=0`. Disassembly of
the exact experimental Image using its System.map shows `ldr x0, [x19, #336]`
at +0x2c (loading `vreg->rdev`), followed by `ldr x1, [x0]` at +0x30. This
confirms the NULL rdev dereference. `0002-labibb-defer-ocp-irq.patch` requests
the interrupt with `IRQF_NO_AUTOEN` and enables it only after registration has
assigned rdev. Registration failure leaves the IRQ disabled. An OCP callback
after registration can enable its IRQ immediately. The diff passes checkpatch
without errors or warnings (sign-off/commit metadata excluded).

This targeted fix does not explain all the other experimental-boot failures.
Subsequent non-kexec boot tests recovered those functions, as recorded below.

Revision 2 built successfully on the build host and passed the boot-image inspector
and bundle checksum verification. Its kernel store path ends in
`q0270jsr8kiy3g8irs5qdsi9kgdrsl3b-linux-aarch64-unknown-linux-gnu-7.1.0-rc1`.
The handoff bundle is `.build/fingerprint/fingerprint-r2.tar.gz`, including
`LAPTOP-TEST.md`, the boot image, manifest, checksums, probe and collector.
Boot image SHA256:
`3fd35148ebd8eae939a9232189c8d26b48b52add3fed08d9121c5910f4c15656`.
Revision 2 was temporarily booted with `fastboot boot` on 2026-11-03.
The installed boot partition was not changed. USB and Tailscale SSH work;
Wi-Fi and HTTPS work; the physical ALSA card is detected; modem, ADSP and
CDSP are running; no failed system services were reported. Kernel taint is
512, matching the normal-boot warning state. The LAB/IBB Oops and memory
assignment failures did not recur in the captured baseline.

The discovery probe reports `/dev/tee0 impl=5`. Loading the stock `gfber`
trustlet from the read-only modem partition failed with SCM error `-22`
(`EINVAL`). This is a separate, explicitly triggered firmware-load failure,
not the earlier boot-time remoteproc memory-assignment failure. No sensor
commands were sent. The firmware search path was restored and the partition
unmounted afterward. Fingerprint operation remains unverified.

Full baseline and post-probe logs are in build-server artifacts
`.build/fingerprint/poco-fingerprint-r2-audit.tgz` and
`.build/fingerprint/poco-fingerprint-r2-postprobe.tgz`.

Build the small transport probe on the build server:

```sh
aarch64-linux-gnu-gcc -static -O2 -Wall -Wextra -Werror \
  kernel/fingerprint/qsee-probe.c -o .build/fingerprint/qsee-probe
```

Without arguments, it discovers QSEECOM by `TEE_IOC_VERSION`. With `gfber` or
`fpcber`, it loads that stock signed trustlet and closes the session. It never
sends sensor commands or accesses biometric storage. Successful loading alone
does not identify the attached sensor. Firmware must be available through the
kernel firmware search path.

## Next hardware checks

1. Package the kernel with the current boot backup's initrd and header, and
   compare its configuration with `/proc/config.gz` from the phone.
2. The running phone has `CONFIG_KEXEC=y`, `CONFIG_KEXEC_FILE=y` and kexec-tools
   2.0.32. A temporary kexec test can avoid flashing: use the validated initrd,
   the live `/sys/firmware/fdt`, and the current command line. The installed
   `prepare-kexec.service` skips loading when a kernel is already loaded.
   A normal power-cycle returns to the installed boot image. Temporary
   `fastboot boot` over USB is another option.
3. Verify QSEECOM version detection and TEE device discovery through
   `TEE_IOC_VERSION`; do not assume a particular `/dev/teeN` index.
4. Keep access to the experimental TEE devices root-only. Test the stock signed
   trustlets with their actual protocols, using separate scratch storage for
   any supplicant. Do not expose existing persist data to an experimental client.
5. Identify the responding sensor before selecting its kernel resource driver
   and device-tree compatible. Only then test interrupts, capture and matching.

## Firmware-load investigation (2026-11-03)

On R2, both `gfber` and `fpcber` fail at APP_START with SCM `-22`.
Dynamic debug reports QSEE result `-1`, response type/data zero. Opening
sessions by name through `/dev/tee0` returns `ENOENT` for both. The probe
accepts `gfber --lookup` / `fpcber --lookup` to perform this check without
loading firmware or sending sensor commands.

The reference is LineageOS android_kernel_xiaomi_sdm845 at
`aa8adfe9bf212c6e93238a86980b4024da4f819a`, files
`drivers/misc/qseecom.c`, `include/soc/qcom/qseecomi.h`, and the Beryllium
MP -> P2 -> P1 -> P0 device-tree include chain. Inspection confirms:

- APP_START uses owner 50, service 1, command 1, three value arguments:
  metadata length, full concatenated length, physical image address.
- Both fingerprint trustlets are ELF32. Each split image consists of its
  metadata followed by all eight segment files in order.
- Vendor TA staging memory is restricted to below 4 GiB by the reserved
  DMA pool's `alloc-ranges`, independently of the address-field width.
- Vendor initialization announces a secure app region and handles common
  libraries. Those steps are absent from the experimental TEE loader.
  Beryllium P0 overrides the region to base `0x86d00000`, size `0x03c00000`;
  this is inside mainline Beryllium's no-map TZ reservation, not the separate
  `qseecom@8ab00000` region. Do not confuse these regions.

R3 logs the staging address and raw SCM return on failed app-manager calls.
Its attempted `GFP_DMA32` restriction is ineffective: `dma_alloc_attrs()`
clears zone flags and uses the device coherent DMA mask. Thus R3 is a
diagnostic build, not a valid test of the below-4-GiB restriction. A real
restriction needs a suitable coherent DMA mask.

R3 was booted via fastboot. APP_START for `gfber` used physical address
`0xd0400000`, metadata length 7004, image length 1562194. It returned raw SCM
status `-2`, QSEE result `-1`, type/data zero. The address was already below
4 GiB, so high staging memory does not explain this observed failure.
The ineffective GFP/pool change was removed; address/error diagnostics remain.
Logs: `.build/fingerprint/poco-fingerprint-r3.tgz`.

R4 adds a deliberately explicit, temporary debug parameter
`qcom_scm.fingerprint_secapp_notify`. Writing `1` to its sysfs parameter
announces the exact vendor Beryllium app region. It is not run during probe.
It rejects non-Beryllium machines and verifies that the current device tree
reserves the entire region as no-map before making the call. Successful
notification is recorded, and repeat requests are rejected. This is a
bring-up diagnostic, not a proposed upstream ABI.

### R4 hardware evidence

On the first R4 boot, `gfber` failed to load before APP_REGION_NOTIFICATION.
The announcement returned raw `0xfffffff8` (mapped to `EINVAL`) with QSEE
result/type/data zero, but the immediately following load succeeded. Three
subsequent loads also succeeded and closed cleanly. The error must not be
silently reclassified as success: this is observed firmware behavior needing
further explanation.

`gfber-identify.c` implements only the vendor class-1 commands 1000 (356-byte
zeroed payload) and 1001 (168-byte zeroed payload), confirmed from the shipped
`libgf_hal.so`/`libgf_ca.so`. Its header has token at 0, class at 4, timestamp
at 8, command at 16, patched 32-bit payload address at 20, length at 24.
It holds the known reset/power GPIOs during the test and restores them after.
It does not send enrollment, authentication, or template-management commands.

The powered test returned data through the TEE, including dimensions 88 and
108 at offsets 100 and 104 of command 1001. Bytes 132..163 are stable and
nonzero with power enabled, all zero with `--power-off`, and return to the
same nonzero value when powered again; offset 164 changes from 32 to 0 and
back. This is strong evidence of communication with physical Goodix hardware,
not just loading a trustlet. The precise meaning of that 32-byte field is
not established. Do not publish it as a chip identifier or biometric sample.

Logs: `.build/fingerprint/gfber-identify-r4.txt`,
`gfber-poweroff-r4.txt`, `gfber-powered-repeat-r4.txt`, and
`poco-fingerprint-r4.tgz`. Kernel taint remained 512, the ALSA card remained
present, and no failed services were reported after testing.

A second R4 boot reproduced failure before announcement and success afterward.
ELF relocation-aware decoding of the vendor HAL's exported `cmd_id_table`
and `err_table` establishes command 1000 as `GF_CMD_DETECT_SENSOR`, command
1001 as `GF_CMD_INIT`, and status 1062 as `GF_ERROR_SENSOR_NOT_AVAILABLE`.
A later power-off control returned exactly 1062; powered runs on either side
returned zero and a 32-byte OTP field. The vendor's Milan HV initialization
code calls `gf_hal_common_save_otp_info_into_sdcard` with that field, confirming
its OTP role. The diagnostic now reports presence/length without printing it.

R5 replaces the manual parameter with lazy Beryllium-only initialization on
first APP_START. It validates the reserved region and serializes setup. A
narrow firmware quirk recognizes only owner 50/service 1/command 5, raw
status `0xfffffff8`, zero QSEE result/type/data, and Beryllium compatibility.
It keeps a warning in the log and does not change other SCM error handling.
This quirk is based on the two repeated boot experiments; its firmware-internal
meaning remains unknown. The R5 boot test is pending.

`run-info.sh` runs the diagnostic as root, mounts the modem partition read-only,
and restores the firmware search path and mount state afterward. Keep its
compiled `gfber-identify` binary alongside it. `--power-off` is a negative
control and may correctly return a nonzero status because the sensor is absent.
