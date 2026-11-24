# Minimal stock-kernel diagnostic image

Build-only tooling. Nothing here connects to, boots, mounts or flashes a phone.
The packer preserves MIUI's original kernel plus every appended DTB, the boot
header version and any recovery DTBO payload. It replaces the ramdisk, updates
its size/offsets/digest, and appends explicit boot safety arguments recorded in
the manifest: `rdinit=/init init=/diagnostic-never-run-root
rootfstype=diagnostic-no-rootfs panic=0`. These prevent default init fallback
and disk-root mounts if the ramdisk is missing. A pre-PID1 failure hangs for
manual recovery; only after PID 1 starts can its watchdog reboot to bootloader.

```
kernel/fingerprint/diagnostics/stock-kernel/build.sh \
  <path-to-miui-rom.zip> \
  .build/fingerprint/diagnostics/stock-probe \
  .build/fingerprint/stock-kernel/gfber.img \
  .build/fingerprint/miui-reference
```

The optional third argument is passed into RAM as `/gfber.img`. The optional
fourth argument explicitly includes only `cmnlib.mdt` and `cmnlib.b00` through
`cmnlib.b05` in `/lib/firmware`, for the stock driver firmware loader. No
other trustlets are included. Both `/init` and
`/probe` must be static AArch64 executables. The independent probe must itself
be reviewed to avoid persistent writes; the init program does not enforce a
syscall sandbox around a root hardware diagnostic.

PID 1 mounts only proc, sysfs, tmpfs and configfs. It does not include a shell,
Android init, TWRP, qseecomd, block device nodes or a filesystem listener. Only
`/dev/null`, `/dev/console`, `/dev/qseecom` and `/dev/ion` nodes are created.
The probe runs once, with a 40-second timeout. All output goes to a RAM-only
log on a 2 MiB tmpfs. A fixed-response HTTP server exposes at most the first
64 KiB of that log at `http://172.16.42.1:8080/`. There is no command endpoint
or selectable file path. USB uses NCM, with device `172.16.42.1/24`; the host
must configure its corresponding interface to `172.16.42.2/24`.

An independent watchdog requests `reboot("bootloader")` 180 seconds after
PID 1 starts. The main loop also enforces this deadline. A failed reboot
syscall leaves the process paused; hardware restart remains the fallback.
The probe timeout cannot interrupt an uninterruptible kernel driver call,
but the separate watchdog can still request reboot.

MIUI's kernel is 4.9.186-perf-g10af704. It has QSEECOM, ION, configfs USB NCM,
FunctionFS and gzip initramfs built in. It lacks devtmpfs, so PID 1 resolves
QSEECOM/ION device numbers through sysfs. This image has not been boot-tested.
Do not use the original TWRP ramdisk: it starts secure-storage-related services.

The captured phone command line has no `skip_initramfs`; nevertheless, this
flag cannot be undone by `rdinit=` if a bootloader unexpectedly adds it. The
invalid root filesystem type and explicit nonexistent fallback init prevent
that failure from mounting a disk root or executing Android init, provided the
bootloader does not override those explicit guards. Absolute certainty about
unobserved bootloader command-line overrides is not possible from the image.

## Hardware result: unsuccessful boot

Image SHA256 `8242eebae2495546bf3a94b340b4c4ec357d7211c16cbe3829d0b1657fe3da5c`
was accepted by temporary `fastboot boot`, but no USB network/log endpoint
appeared. The user reported a stuck screen; the automatic watchdog did not
restore fastboot within the observed interval. Whether PID 1 ran is unknown.
Manual Power + Volume Down recovery was requested. Do not repeat this image
as a validated diagnostic; it produced no Keymaster comparison result.
No flash or erase command was issued.

## Prepared early-userspace sentinel

`reboot-sentinel.S` is a separate, syscall-only AArch64 PID1. It requests a
bootloader reboot immediately, without libc, mounts, USB, probe execution or
firmware loading. An unexpected syscall failure exits PID1; `panic=5` requests
an automatic restart after a panic. A pre-userspace hang can still require
Power + Volume Down, so have physical recovery available before booting.

Build and package with the existing TWRP-derived kernel, replacing its entire
ramdisk (TWRP init is never included):

```sh
mkdir -p .build/fingerprint/stock-sentinel
aarch64-linux-gnu-gcc -nostdlib -static -no-pie \
  -Wl,--build-id=none -Wl,-e,_start \
  kernel/fingerprint/diagnostics/stock-kernel/reboot-sentinel.S \
  -o .build/fingerprint/stock-sentinel/init
python3 kernel/fingerprint/diagnostics/stock-kernel/pack.py \
  --boot-image <path-to-twrp.img> \
  --reboot-sentinel --init .build/fingerprint/stock-sentinel/init \
  --output .build/fingerprint/stock-sentinel
```

Prepared image SHA256:
`ae07c63790f3fca33777ec9d133066de9f8f1831c7d24877293b3c6d649ca1a2`.
Static verification passed: original kernel unchanged, complete CPIO containing
only `/init`, no dynamic interpreter, guarded command line, correct boot ID.
The image was temporarily booted and Fastboot was observed afterward. Without
an execution log, this is consistent with its intended action but does not
independently prove PID1 execution or fingerprint functionality.


## Supervised diagnostic and live limits

`supervisor.S` is a syscall-only PID1. It starts `/diagnostic` as a child,
checks for child exit, and requests bootloader reboot after at most90 one-second
sleeps. Build `init.c` with `DIAGNOSTIC_CHILD` for this arrangement, and package
with `--init SUPERVISOR --worker DIAGNOSTIC --probe PROBE`. The packer preserves
the kernel and adds guarded root/init arguments and `panic=5`. This bound assumes
responsive kernel syscalls; it is not a guarantee against a kernel/firmware hang.
The worker delays the probe15 seconds so USB logs can become available first.

A static musl1.2.5 worker was built from cached cross-toolchain libraries and
strictly compiled. The supervisor syscall arguments, staticELFs, complete CPIO
and boot digest were independently checked. Both the original full TWRP-derived
diagnostic and supervised variant were temporarily booted, but neither exposed
USB logs. The user confirmed manual Fastboot recovery. Return timing cannot
establish that either watchdog executed, and no Keymaster result was obtained.
The known working R2 image was then restored. Do not repeat these as validated
stock-fingerprint diagnostics or infer a libc/kernel failure without logs.
