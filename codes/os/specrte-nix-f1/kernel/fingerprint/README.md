# Poco F1 fingerprint kernel bring-up

**Update 2026-11-04:** Android uses `gfdip`. Linux tests with that signed
trustlet now pass wrapped-key import, capture, SAVE, enumeration and one
match to the saved template in an isolated RAM test. Controlled same-finger
acceptance and different-finger rejection now both pass.
The `gfber` results below are historical; they do not identify the correct
authentication trustlet. See [current enrollment findings](ENROLLMENT.md).
Persistence across reboot is not yet implemented.

The experimental kernel and diagnostic can detect and initialize the phone's
**physical Goodix fingerprint sensor** through its stock `gfber` trustlet.
It reports 88×108 dimensions and 32 bytes of sensor OTP data with power on.
A power-off control returns `GF_ERROR_SENSOR_NOT_AVAILABLE` (1062), or zero
OTP data; restoring power restores successful detection and initialization.
The diagnostic reports OTP presence, never its contents.

**This is not fingerprint login:** persistent enrollment, broader matching
validation, libfprint/fprintd integration and desktop unlocking remain unfinished. There is no dedicated fingerprint GPIO driver; the diagnostic uses
the kernel GPIO character interface for power/reset and the QSEECOM TEE driver
for secure communication. The normal installed kernel remains unchanged.

## Final implementation

- `0001-qseecom-tee.patch` adds the experimental QSEECOM TEE transport.
- `0002-labibb-defer-ocp-irq.patch` fixes the observed regulator IRQ race by
  keeping its interrupt disabled until regulator registration assigns `rdev`.
- `bringup.config` enables QCOM_QSEECOM and TEE_QSEECOM only for the separate
  experimental kernel build, with the UEFI client disabled.
- `gfber-identify.c` loads the stock trustlet, sends only class-1 commands
  1000 (`GF_CMD_DETECT_SENSOR`, 356 bytes) and 1001 (`GF_CMD_INIT`, 168 bytes),
  checks transport and vendor status, and restores the GPIOs on completion.
- The first trustlet load after boot can fail with `EINVAL`. The diagnostic
  retries that specific error **once**, after 50 ms; another error is reported.
  Its underlying firmware/driver cause is not yet established.
- No region-notification workaround, SCM error override, DMA allocation
  change, or speculative SPI node remains in the final source.

## Build and temporary boot

Build on the configured build server:

```sh
nix-build --store ../nix-store \
  --option build-users-group '' --cores 12 --max-jobs 1 \
  kernel/fingerprint -o .build/roots/result-fingerprint-kernel

aarch64-linux-gnu-gcc -static -O2 -Wall -Wextra -Werror \
  kernel/fingerprint/gfber-identify.c -o .build/fingerprint/gfber-identify
```

The final kernel store basename is
`q0270jsr8kiy3g8irs5qdsi9kgdrsl3b-linux-aarch64-unknown-linux-gnu-7.1.0-rc1`.
It is the R2 kernel, with the corrected userspace diagnostic. `prepare.py`
validates the original 64 MiB boot backup and its configuration, permits only
the two QSEECOM option changes, and preserves the original initrd and DTB.

Use **temporary `fastboot boot`**, not kexec. Kexec caused broad hardware
failures in the earlier experiment; normal boot recovered them. Do not flash
a partition as a substitute for a failed temporary boot.

Place `run-info.sh` and the compiled `gfber-identify` together on the phone:

```sh
sudo sh ./run-info.sh
# Optional negative control; a nonzero exit is expected if detection fails:
sudo sh ./run-info.sh --power-off
```

The wrapper mounts the modem partition read-only and restores the firmware
search path and mount state afterward. It does not read enrolled templates,
send enrollment/authentication commands, or write phone partitions. The GPIO
mapping is TLMM reset 37, power 94, IRQ 121; chip label is validated before use.

## Provenance and evidence

The transport patch is the unmodified cumulative diff from
[wrobelda/linux](https://github.com/wrobelda/linux), base
`8cd9520d35a6c38db6567e97dd93b1f11f185dc6`, tip
`4855f9e2f0a1e44e32404c7c4ce56e5052fd04ce` (`qcom-qseecom-tee`). Its SHA256 is
`6efd0b5fc73b6d0bad38c1f50d47077a4718169be6ab32fa0c8871b6f76fae74`.
SPDX and copyright notices are preserved. Its upstream device was SM8250;
this experiment tests the relevant paths on this SDM845 Poco F1.

GPIOs and vendor QSEECOM behavior were checked against
[LineageOS Beryllium sources](https://github.com/LineageOS/android_kernel_xiaomi_sdm845/tree/aa8adfe9bf212c6e93238a86980b4024da4f819a).
The actual `gfber` message layout and command/error names were verified from
the phone's `libgf_ca.so` and `libgf_hal.so` (`cmd_id_table`/`err_table`).
The gfenu protocol in [Goodix research](https://github.com/wrobelda/goodix-fp-spi-linux/tree/4663eb9019f5721d62984d3a81e642c7ac8ef530)
is different and was not used blindly.

Full build logs, boot backups, hardware logs and test images are under
`.build/fingerprint/` on the server. `INVESTIGATION.md` preserves the historical
experiments, including the rejected region-notification hypothesis. Those
changes are not in the final kernel. No biometric enrollment or matching has
been claimed from a successful transport response.

## Final test result, 2026-11-03

The cached R2 kernel plus updated diagnostic passed a fresh fastboot boot:
initial APP_START EINVAL recovered with the single retry; DETECT_SENSOR and
INIT both returned transport/vendor status zero; power off returned 1062;
power on again restored successful detection and 32-byte OTP readout.
Wi-Fi HTTPS, physical ALSA-card detection, modem/ADSP/CDSP and system services
passed checks. Kernel taint stayed 512; no regulator Oops recurred. Audible
playback and biometric capture/enrollment/matching were not tested.

The installed boot partition still hashes to
`8c27a090763dc1a31ec4cd8b0419756a81d876cf40e9b5b9589f28ffebbbfdf0`, identical
to the original backup. The phone was left on the temporary test kernel.
Final logs: `.build/fingerprint/final-test.txt` and
`.build/fingerprint/poco-fingerprint-final.tgz`.
Verified image/tools bundle: `.build/fingerprint/fingerprint-tested.tar.gz`.
