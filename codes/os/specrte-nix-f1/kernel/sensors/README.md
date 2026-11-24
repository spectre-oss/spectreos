# RAM-only SLPI diagnostic

The stock Android baseline proves TMD2725 proximity/light response. Linux's
firmware derivation dropped the archive's `usr/share/qcom` tree, and the normal
kernel disables SLPI after an earlier restart loop. Firmware packaging now
preserves the HexagonFS tree at `$out/share/qcom`.

`pack.py` builds a temporary Android-format boot image from the tested R2 Linux
kernel and board DTB. It enables only the SLPI DT status, replaces the ramdisk,
and includes a FastRPC listener and sensor configuration. `init` never mounts
phone partitions. It serves diagnostic logs at `http://172.16.42.1/` over USB
RNDIS and requests a normal reboot after 180 seconds. This watchdog is a
userspace fallback, not protection against every kernel hang. Use `fastboot boot`,
never flash this image. Android's installed Magisk boot remains intact.

SLPI firmware is initially outside the firmware search path. After USB/logging
is set up, init exposes it, disables automatic SLPI recovery, and starts SLPI
once. This captures the first failure without repeatedly reassigning memory.
Other DSPs/graphics firmware are intentionally absent in this isolated image;
their missing-firmware errors are not full-system regressions.

The first test (diagnostic-r3) booted Linux and exposed USB logs. SLPI started,
and FastRPC assigned reserved memory without error. Root-PD listener attach
failed with EPERM. At 42.7 seconds SLPI reported:
`USER-PD DOG detects stalled initialization`. It remained crashed without a
restart storm. The next image selects the sensor PD with hexagonrpcd `-s`.
No claim of working Linux sensors is made yet.

Local artifacts: `.build/sensors/`; original device sensor registry backup is
private and is not written back to Android. Images currently use the archive's
registry in an ephemeral RAM copy. Firmware/configuration provenance must be
compared to working Android if sensor initialization still fails.

References:
- https://github.com/linux-msm/hexagonrpc (HexagonFS layout and listener)
- https://lists.openwall.net/linux-kernel/2026/09/22/3194 (reported restart ownership failure; the proposed ignore-error patch is not applied)

Further tests:
- R3 automatically returned to rooted Android successfully after its timer.
- R5 added sensor-PD attach and libssc tools. Attach still returned EPERM;
  SLPI again hit its sensor-process initialization watchdog after ~43 seconds.
  It automatically returned to Android successfully.
- Reconstructed stock `slpi_br` with pil-squasher and compared SHA256 to the
  shipped Linux `slpi.mbn`: both are
  `0175456a8cc9b2848a07ea94cef745e9edf7f8df6048b9dced0879ca761a596a`.
  The Linux archive already contains the correct Android firmware variant.
- R6 added pd-mapper and a light query. Attach still failed, and libssc reported
  SSC QMI service not found. Tracefs is unavailable in the existing kernel;
  no function-trace result is claimed. Building an explicit logging-only
  diagnostic kernel through `kernel/sensors/default.nix` next.

Normal kernel SLPI remains disabled until a working configuration is demonstrated.
The packaging fix alone does not establish working Linux sensors.

R6 also returned to Android with `sys.boot_completed=1` and root access intact.
The first failed attach precedes the initialization watchdog, so restart-loop
patches alone cannot establish working sensors. No failed secure-memory check
is ignored in these experiments.

The next diagnostic uses logging in FastRPC send/callback/init-call paths because
tracefs is not enabled in the baseline configuration. It can optionally duplicate
the SDSP session via `--compare-client-ids`: test normal client ID 1 first, and
only if that listener exits, hold its session slot open and try client ID 2.
This is a diagnostic hypothesis, not a production DT change. Android debugfs
confirms its working sscrpcd uses domain 2, PD 2, no SMMU, and physical buffers
in the same 0xFEC00000 region. Its client identifier is its process ID.

R7 accepted client ID 2 after rejecting ID 1; successful listener traffic then
exposed registry rebuild requests. All 29 configuration files match stock vendor
byte-for-byte. Their registry cache records mtime 1230748200, whereas the test
CPIO normalized timestamps to zero. R8 restores that mtime in RAM, and the
registry write/rebuild requests disappear. Critically, R8 also accepted client
ID 1 on its first attempt. Therefore the client-ID hypothesis is NOT confirmed;
the candidate ID-offset build was stopped and must not be promoted. Timing or
other initialization state may explain the earlier paired result.

## Confirmed Linux sensor response (2026-11-04)

R9 ran the logging kernel in RAM and exposed SSC service 400. The first
sensor-PD listener attach failed, but the delayed retry with the SAME client
ID succeeded; the second session was not used. Restored configuration mtimes
prevented registry rebuild requests. The user covered/uncovered the earpiece
and exposed it to light, confirming completion of that interaction.

- TMD2725 proximity: FAR → NEAR → FAR → NEAR → FAR.
- TMD2725 ambient light: 43 samples, 0–36 lux.
- SLPI remained running at 160 seconds; no processor restart was requested.
- Evidence: `.build/sensors/logs-r9/`, including `result.json`, raw QMI logs,
  kernel log and QRTR service list. These are local generated artifacts.

This establishes actual proximity/light response under Linux 7.1, not just
sensor enumeration. It does not establish suspend/resume, continuous uptime,
Phosh integration, fingerprint authentication, or IR camera support.

`integration.nix` builds a kernel without QSEECOM experiments or diagnostic
logging. With `sensorBringup = true`, SLPI is enabled but its automatic boot
and recovery are disabled specifically on Beryllium. The opt-in NixOS module
(`spectre.sensors.enable = true`) copies the preserved configuration into RAM,
restores the stock mtimes, starts SLPI once, and supplies the sensor-PD listener
with bounded retries. It refuses to restart a crashed SLPI. The project config
enables this option after the R10 kernel test; full-system integration remains
unverified. SLPI firmware
is also excluded from the early initrd.

The RAM diagnostic offers a root debugging shell on USB address
172.16.42.1:2323 only. It is not part of the NixOS service or installed Android.

R9 returned to Android automatically; `sys.boot_completed=1` and Magisk root
were verified. The generated `poco-sensors.service` and its pre-start script
build successfully. The LAB/IBB OCP IRQ ordering fix is now applied to normal
kernels as well as fingerprint experiments: the registration race is independent
of QSEECOM. The integration kernel retains that fix without enabling QSEECOM.


## Integration kernel R10

Built and RAM-booted the kernel from `integration.nix` successfully. Its SHA256
is `65f6d2cb62b9f8626a29d5a31dc568824de97e98c7c47c8333102dee42f95f37`.
The RAM boot image SHA256 is
`543864b9997ba51d4fbfe07459c1125fa3bb7c587bff62134294fa110843f96f`.
The running config confirms QCOM_QSEECOM is disabled. The original one-session
DT is used; no client-ID offset or extra session is needed. First listener
attach failed and the bounded retry succeeded. SLPI was offline until init
started it; recovery remained disabled. Captured taint is 512 (warning), with
no Oops or SLPI watchdog failure in the captured log.

The user repeated the physical test on R10: proximity produced 11 alternating
FAR/NEAR reports; light produced 14 samples from 0 to 14 lux. SLPI remained
running at 150 seconds. Evidence is in `.build/sensors/logs-r10/`.

The NixOS configuration now enables `spectre.sensors.enable`. Its built systemd
unit waits for SSC service 400 before allowing iio-sensor-proxy to start. The
complete NixOS image has NOT been installed or desktop/suspend-tested. Android
partitions remain untouched by all R3–R10 diagnostics. The full-system service
is build-validated; the RAM test validates the kernel and equivalent startup
steps, not a running systemd/Phosh session.

R10 was then rebooted normally from the RAM shell. Android returned with
`sys.boot_completed=1` and Magisk root verified. No image was flashed.
