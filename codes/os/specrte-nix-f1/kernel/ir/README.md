# Poco F1 IR camera and illumination — Android baseline, 2026-11-04

Confirmed on rooted stock MIUI before returning to Linux. No partitions were
flashed, no illumination setting was manually changed, and no face images or
enrolled face templates were exported. Evidence lives in the ignored local
`.build/stock-android/ir-investigation/` directory. The phone's clock reports
2020-12-27; use host collection date and kernel monotonic time for this session.

## Measured hardware and software

| Component | Evidence |
|---|---|
| IR sensor | OV7251 vendor module/library; stock probe slot 3 reports ID 0x7750 and 8-bit slave address 0xc0 (7-bit 0x60). Mainline OV7251 driver expects ID bytes 0x77/0x50. |
| Android camera | ID 5, front-facing, orientation 270. `com.miui.face` camera-service events and its Camera2Util bytecode both identify this ID. |
| Direct capture | 146 YUV frames at 640×480 in a five-second request; final `capture_complete ... success=yes`. Only aggregate luminance statistics recorded. |
| Illuminator | `leds-lm3644`, I²C 0-0063 under a88000.i2c. Stock interfaces include `/sys/class/leds/ir`, `/sys/class/leds/ir_torch`, and `/dev/flood`. These are different entry points/modes, not interchangeable names. |
| Stock modulation | Device tree and runtime expose period 25000 µs and duty 2500 µs: 40 Hz, 10%. This does not establish frame-synchronized triggering. |
| Face software | `/system/app/MiuiBiometric/MiuiBiometric.apk`, package `com.miui.face`; SenseTime Java API loads `libjni_stfaceunlock_api.so`, which depends on `libstfaceunlockoclqcom.so`; the latter depends on OpenCL. APK bundles liveness and verification model resources. |

A scoped dynamic-debug enable of `leds-lm3644.c` line 195 recorded:

```
[4631.131912] lm3644_control: brightness = 72, opmode = 1
[4631.132391] Simulative PWM enabled
[4631.641757] lm3644_control: brightness = 0, opmode = 1
```

The user confirmed a face-unlock test; camera-service events recorded camera 5
opening/closing in that interval. In the referenced driver, mode 1 is IR and
brightness zero switches to standby. 72 is a driver register value, **not mA
or a percentage**. Do not substitute it into a different driver's current API.
The debug callsite was initially disabled and restored to disabled afterward.
Normal face unlock may briefly reactivate illumination; subsequent idle reads
returned zero. No explicit manual illumination writes were made.

## Wiring captured from the running stock device tree

Illuminator node `lm3644@63`:

- HW enable: TLMM GPIO 93; torch/PWM: GPIO 96; TX: GPIO 135.
- Three pinctrl states: active, active PWM, suspend.
- `lm3644,use-simulative-pwm` and the period/duty above.
- Stock boot reports no configured IR protection timer. A Linux implementation
  must preserve bounded operation and turn off on stop, error and suspend.

Camera slot 3:

- CCI master 1; CSI PHY index 1; MCLK3 at 24 MHz (GPIO 16).
- Reset GPIO 25; named analog/digital enable GPIOs 91 and 7.
- I/O supply references `camera_iovdd_ldo` (1.8 V, shared GPIO 50).
- Analog upstream references `pmi8998_bob` (3.3–3.6 V).
- Digital upstream references `camera_vdig_ldo` (1.35 V, shared GPIO 99).

These are stock board resources, not a verified mainline power sequence. Resolve
module power-setting order, actual sensor rails, shared regulators, lane mapping
and polarity before writing a Linux device-tree patch. The captured module and
tuning binaries are retained for that work.

## Direct-capture probe

`android-ir-probe.cpp` uses Android's NDK camera API, camera 5, a 640×480 YUV
analysis stream and a private GPU preview stream matching the stock service's
two-stream arrangement. It acquires/discards preview buffers and reports only
frame count, dimensions and luminance aggregates from the analysis buffers.

A standalone native process must start its Binder callback pool. The first
single-stream and two-stream tests omitted this and returned no frames, then
hit the external 12-second timeout. The camera-service event log reported a
Binder client death. Camera provider PID 769 and cameraserver PID 1449 remained
unchanged; this is **not evidence of a camera-provider crash**. Adding
`ABinderProcess_startThreadPool` resolved capture and clean shutdown. The
successful result does not isolate whether two streams are strictly required.

Build with `NDK_ROOT=... kernel/ir/build-android-probe.sh`. Run only when camera 5
is idle, as root, with `timeout 12 /data/local/tmp/android-ir-probe`. The helper
requests five seconds of capture and closes its camera afterward. The external
watchdog also covers blocked vendor calls. Do not install it as a background
service. The device copy is removed after this baseline.

## Original Linux implementation plan (historical)

1. Reuse `drivers/media/i2c/ov7251.c` and its `ovti,ov7251.yaml` binding in the
   current kernel tree. Add verified board power, clock, CCI and CAMSS endpoint
   wiring; enable the sensor config. No OV7251 node is presently wired in the
   inspected beryllium DTS files.
2. Port the LM3644 IR functionality or implement an appropriate upstream-style
   driver. The inspected tree has LM3642 support but no LM3644 IR equivalent;
   blindly binding the LM3642 driver is not justified. Preserve PWM, fault checks
   and off-on-close/error/suspend behavior before enabling illumination.
3. Test Linux image capture and illumination independently, then together.
4. Treat recognition/liveness and lock-screen integration as separate userspace
   work. Android JNI/OpenCL libraries and model files are evidence, not a proven
   Linux-compatible authentication implementation.

Reference driver: LineageOS android_kernel_xiaomi_sdm845 commit
`aa8adfe9bf212c6e93238a86980b4024da4f819a`,
`drivers/leds/leds-lm3644.c`. Its control logging and binding agree with the
running stock kernel, but it is not claimed to be the exact stock build source.

Evidence: `stock.dtb`, `stock.dts`, `sensor-ov7251.so`, `module-ov7251.bin`,
`tuning-sunny-ov7251.bin`, `face-bytecode.txt`, `live-led.txt`,
`after-direct-kernel.txt`, `camera-after.txt`, `direct-capture-binder.txt`,
`debug-restored.txt`, and the native-library/APK copies. SHA256 inventory is in
`manifest.sha256`; generated artifacts and proprietary binaries are not committed.

## Experimental Linux implementation

`default.nix` opts into OV7251 and adds `board.dtsi` plus the diagnostic
`leds-lm3644-ir.c`. It is separate from the normal kernel configuration.
The sensor has CCI1 address 0x60, MCLK3, CSI PHY1, reset 25, analog enable 91
and digital enable 7. Fixed regulators represent the board enable gates and
avoid requesting a guessed sensor voltage. The shared digital upstream is
`cam_ldo`. CSI data lane 0 on CAMSS has now delivered complete frames; see the
follow-up results below.

The LM3644 diagnostic identifies the chip with outputs off. Its root-only
`pulse_ms` attribute accepts 1–500 ms, uses the observed stock register value
72 and 40 Hz/10% PWM, and deasserts HW enable after every pulse or error.
Suspend/shutdown/removal turn it off. There is no persistent brightness control
and no automatic illumination at boot. This is a bring-up interface, not the
final V4L2 flash/illumination API. The stock GPIO/pinctrl/PWM behavior must still
be checked on Linux before illumination can be claimed working.

The RAM init has a six-minute reboot watchdog and mounts no phone partitions.
The packer must receive the **matching new DTB** via `--dtb`; replacing only
Image.gz leaves the old wiring in place. `pixel-stats.c` summarizes unpacked little-endian Y10 or packed Y10P
(`--packed`) in RAM without printing pixels. `--frame-pixels 307200` reports
per-frame aggregates and rejects incomplete frames. Raw capture files must
stay outside the HTTP log directory and be removed after analysis.

## Linux RAM test results — 2026-11-04

The opt-in kernel built successfully and the RAM image booted. Linux detected
`ov7251 17-0060`, revision 7 (1F), and LM3644 at `10-0063`, ID 2.
The first test rejected a 501 ms illumination request and accepted 250 ms;
PWM became enabled during the request, then disabled with HW enable low.
The requested 40 Hz/10% PWM was quantized by LPG to period 23,071,290 ns and
active time 2,197,266 ns. This establishes software control, **not optical
emission**, and requires review before treating it as equivalent to stock.

Capture is **not working yet**. Routing PHY1 through CSID1 or CSID0 to
VFE0 RDI0 produced no frames before timeout. A separate repeat with the sensor's
built-in test pattern also produced zero bytes: `VIDIOC_STREAMON` succeeded,
but the eight-second timeout ended capture with status 143. No image was
exported. The raw RAM file was deleted, and the phone was rebooted to Android.
The repeat's logs are in `.build/ir/test-r1-repeat-logs/`; the first boot's
partial log archive is `.build/ir/test-r1-logs/`. The first boot's late test
logs were not downloaded before its automatic reboot; do not treat those as
preserved evidence.

`decode-stock-module.py` follows the extracted module's symbol references and
checks record lengths to decode lane metadata and the single mode's 146
register writes. The captured module reports lane assignment `0x10` and
combo mode false. Translating that encoding to the mainline endpoint remains
a separate driver comparison; the number alone does not validate our wiring.
Stock differs from the Linux mode tables at several registers, including
MIPI timing register 0x4837 (stock 0x32 versus Linux 0x19). These differences
are investigation leads, not established causes. The next step is to trace
PLL/output settings and CSI lane encoding before applying a targeted change.
Recognition and lock-screen integration remain unimplemented.

Run the read-only decoder with:

```sh
python3 kernel/ir/decode-stock-module.py \
  .build/stock-android/ir-investigation/module-ov7251.bin
```


## Capture breakthrough and integration — 2026-11-04

The initial timeout was not fixed by changing sensor PLLs or copying Xiaomi's
mode table. Two missing configuration details were found by controlled tests:

1. **Receiver route:** CSID0 → VFE0 RDI0 captured ten internally generated
   frames. CSID1 → VFE0 RDI0 timed out. The tested sensor route is PHY1 → CSID0
   → VFE0 RDI0 → `msm_vfe0_video0`.
2. **Board switch:** stock camera slot 3's pinctrl also references
   `cam_sensor_ir_oe_active` (TLMM 8, output low, pull-down) and
   `cam_sensor_ir_sel_active` (TLMM 36, output high, pull-up). These had been
   omitted. Linux left GPIO36 low. Applying the stock output levels immediately
   produced ten 640×480 Y10P frames at approximately 30 fps. `board.dtsi` now
   selects those states and uses stock MCLK3 drive strength 4 mA.

The successful data-lane assignment remains 0. Stock PHY register dumps show
lane-enable 0x81, matching Linux. Stock settle counts were 14 versus Linux 17;
changing settle counts alone did not restore capture and is not in the patch.
The temporary `iomem=relaxed` diagnostic was used only to compare receiver
registers; it is absent from the normal test image.

The analog GPIO gate also lacked its stock upstream supply: PMI8998 `bobb1`,
requested within the stock 3.3–3.6 V range. The test reported 3.328 V. Both
`cam3_avdd` and `cam0_avdd` now reference it, since stock assigns the same BOB
supply to both cameras. With only the IR reference added, one fresh boot saw a
rear IMX363 chip-ID mismatch and CAMSS withheld the completed graph; rebinding
the rear sensor succeeded. Sharing the regulator reference is intended to
prevent IR power-down from removing the rear sensor's supply. The final shared-supply configuration passed two fresh boots; see below.

After the mux change, the standard sensor test pattern produced five complete
frames with range 0–1023 and mean 513.098. A 90-frame real capture ran at
approximately 30 fps. No raw frames left the phone; all files were in `/run`
and deleted after aggregate analysis.

### Sensor strobe and illumination

The stock LM3644 pulse request alone did not change captured brightness with
the generic OV7251 output settings. Replaying stock sensor settings produced
a response. This was narrowed to eleven sensor output/strobe register writes,
without replacing PLLs or the mode table. In a 120-frame run, frames 60–74
rose from a mean near 16.84 to roughly 23.9 during a 500 ms LM3644 request,
then returned to baseline. This is a repeatable pulse-correlated response;
this early dark-scene measurement alone did not distinguish optical response
from a power-related black-level shift. The subsequent nearby-target test
below confirmed a much larger reflected-light response.

`0001-ov7251-beryllium-strobe.patch` applies those observed settings only when
`spectre,enable-ir-strobe` is present, immediately after mode programming and
before controls/stream-on. This is an explicit diagnostic extension, not an
upstream binding or a finished V4L2 flash API. The LM3644 hardware-enable pin
remains low outside its bounded request, including at boot; sensor strobe
configuration alone does not enable illumination.

`capture.sh` is a RAM-test helper: it selects the tested route, captures 1–150
frames, verifies the expected byte count, reports only per-frame statistics,
and deletes raw data even on error. Use `/opt/ir-capture 30 0` for real frames
or `/opt/ir-capture 5 1` for the test pattern in an image bundling these helpers.
It resets the media graph and must not be run alongside another camera client.

Local evidence: `.build/ir/test-r1-60fps-corrected-logs/tpg0.txt`,
`.build/ir/test-r1-phy-logs/`, `.build/ir/test-r2-bob-60fps-logs/mux.txt`, and
`.build/ir/test-r3-mux-logs/`. The Android comparison still captured 146 frames;
its temporary `csiphy_dump` setting was restored to zero and its probe binary
removed. `phy-baseline-kernel.txt` and `phy-registers.json` preserve the
working stock PHY configuration under `.build/stock-android/ir-investigation/`.

Build the opt-in kernel on the build host with `nix-build --store STORE
kernel/ir/default.nix`; it does not enable IR in the installed system. Package
its kernel and matching Tianma DTB with `kernel/sensors/pack.py --dtb ...`,
`--init kernel/ir/init`, and an `--extra-root` containing the v4l-utils closure,
`opt/pixel-stats`, and `opt/ir-capture` (a copy of `capture.sh`). The current
init references the pinned v4l-utils store path used by this diagnostic.
Build `pixel-stats.c` statically for ARM64 on the build host; its host build can be
checked with `python3 kernel/ir/test-pixel-stats.py PATH_TO_HOST_BINARY`.
All hardware tests here use `fastboot boot`, never `fastboot flash`.


## Integrated r4 verification

The complete opt-in kernel built successfully with the device-tree and sensor
strobe changes. Kernel output:
`/nix/store/svq020jdvab0c0j9dppgr130g4966270-linux-aarch64-unknown-linux-gnu-7.1.0-rc1`.
The RAM image is `.build/ir/test-r4-integrated/boot.img`, SHA256
`bec3d3e1807a21c266ba203eca2d2f33444b973aa462e0824ba32731df50fc29`.

Two independent boots of that image passed without manual GPIO/register writes
or rebinding drivers:

- Both sensor probes completed; no rear probe failure and no kernel Oops.
- Each captured five full-range sensor test-pattern frames and 90 real IR frames.
- Requests of zero and 501 ms were rejected; a 500 ms request succeeded.
- PWM was disabled and TLMM 93 (LM3644 HW enable) was low after the request.
- Real-frame means changed from approximately 16.7 to 23.8 during the request,
  then returned to baseline. These initial dark-scene runs alone did not
  establish useful scene illumination; see the subsequent target test below.
- Raw capture files were deleted. The final image does not use `iomem=relaxed`.

On the second boot, the rear camera also captured three complete 4032×3024
RAW10 frames (45,722,880 bytes total), followed by another 30 IR frames after
switching the media graph back. This verifies the shared-power change through
both sensor paths, not only probe messages. Evidence and machine-readable
summaries are in `.build/ir/test-r4-boot1-logs/` and
`.build/ir/test-r4-boot2-logs/`; the image manifest records the same results.

The 500 ms limit is a **requested software interval**, with scheduling and
setup overhead; it is not an independent hardware watchdog. This remains a
RAM diagnostic driver, with production V4L2 flash controls, suspend testing,
face recognition/liveness, and lock-screen integration outside this result.
The installed Android partitions have not been changed.

After r4 testing, normal Android boot completed (`sys.boot_completed=1`),
Magisk root returned UID 0, and fingerprint service reported one enrolled
fingerprint with zero HAL deaths. No partitions were flashed or erased.

## Nearby-target optical verification

A third fresh boot of the same r4 image captured five test-pattern frames and
90 real 640×480 IR frames at approximately 30 fps with the user-positioned
nearby reflecting target. Frames 26–40, coinciding with the 500 ms requested
illumination pulse, had means 343.658–389.199 on the 10-bit scale. The average
was 16.587 before the pulse (excluding the first startup frame), 363.478 during
it, and 16.606 afterward. Illuminated frames spanned roughly 13–1023, compared
with a narrow dark range before and after. This large, target-dependent response
confirms reflected IR illumination rather than only the earlier small baseline
shift. Some pixels saturated, so exposure/illumination tuning remains necessary
for useful face images; recognition quality was not measured.

Zero/501 ms requests were rejected, the valid pulse completed, PWM was disabled,
and HW enable was low afterward. Both sensor probes succeeded with no kernel
Oops. Raw captures were deleted on the phone; only aggregate statistics were
exported. Evidence: `.build/ir/test-r4-target-logs/`, including
`optical-result.json`. Across the three boots, 300 real IR frames and 15 pattern
frames have passed, in addition to the earlier three rear-camera frames.

After the target test, Android again completed normal boot, Magisk returned
UID 0, and fingerprint service retained one enrollment with zero HAL deaths.
No partitions were flashed or erased.
