# IMX363 colour diagnosis and calibration

Follow-up: [original Xiaomi camera reverse engineering](camera-xiaomi-research.md)
recovered stock sensor gain logic with REA/JADX and Ghidra. It establishes
digital-gain unity as 256; the pinned Linux driver defaults to 1024 and writes
that value directly. The fixed-exposure comparison below now verifies the
4× signal difference on this phone. The old zero-gain recovery patch's use of
the driver default is superseded by explicit unity gain in version 0.3.5.

## Unity-gain correction (0.3.5)

Three full-resolution RAW-only captures used the same scene, exposure control
1000, analogue-gain code 384 (4×), and digital gain 1024, 256, then 256 again.
Controls were read before and after every capture; they remained fixed. Each
capture contained eight frames, and the final frame was analysed.

- Across 2,884 non-clipped 32×32 patches, the fitted 1024-versus-256 signal
  slope was **3.99034**, with intercept **−191.16** RAW10 codes. This is
  consistent with a ~64-code pedestal and 4× amplification of signal above it;
  it is not a substitute for dark-frame black-level calibration.
- The repeated unity capture differed by only **0.1375 RAW10 codes** on average
  over the same tile grid, supporting scene stability.
- Saturated RAW pixels fell from **1.0505% to 0.2550%** at identical exposure
  and analogue gain. This measures fixed-exposure headroom, not the final
  brightness after auto-exposure convergence.

`patches/libcamera-imx363-digital-gain.patch` now writes 256 on every IMX363
software-processed configuration, including preview/photo switches and camera
reopen. It checks the advertised control range and reports failures. It no
longer trusts the old kernel default or preserves arbitrary positive digital
gain in the software ISP path. Raw-only diagnostic streams keep their explicit
gain setting. Exposure and analogue gain remain under libcamera's control.

`kernel/patches/0002-imx363-unity-digital-gain.patch` corrects the default for
future system builds. Its application was checked against the exact pinned
driver. The live camera package can correct the control on the existing kernel;
deploying this camera update does not install or flash a new kernel.

Evidence is local under `.code-sync/camera-tests/gain-unity/`, including control
readbacks, RAW captures, `raw-comparison.json`, build logs and deployment tests.

### Deployed validation

Installed camera:
`/nix/store/wby7j5wnaypql060ky46gr9q95455653-spectre-camera-aarch64-unknown-linux-gnu-0.3.5`

Libcamera:
`/nix/store/058di8zray9b7im1amxcrmcy680xnfn3-libcamera-aarch64-unknown-linux-gnu-0.7.0`

- ARM compilation and native RGB/stride/rotation, QML, and repeated JPEG/PNG
  save/reopen regressions passed on the mini PC.
- Two 3024×4024 real JPEG photos and camera reopen passed on the phone. Starting
  from a deliberately set 1024, each processed configuration logged unity and
  the final control readback was 256. A separate zero-gain startup test also
  restored 256.
- A three-frame RAW-only regression retained a requested gain of 512. Opening
  the normal camera afterward reset it to 256, confirming the intended scope.
- In the visible sustained preview, first presentation was **290 ms** from
  launch, and displayed preview updates settled at **23.8–23.9/s**. This is a
  warm-machine observation, not a cold-boot guarantee.
- Full-resolution capture through preview restart was about **964 ms** in the
  second photo test, first resumed preview **1104 ms**, JPEG save **382 ms**.
- The managed launcher and GC root point to 0.3.5, so reopening uses the fix;
  it does not rely on a one-time diagnostic control write. Reboot was not tested.
- Visual review of both the screenshot and saved JPEG confirms the broad,
  correctly oriented view and green toy, but **shadow colour cast, noise and
  lamp-area highlight clipping remain**. This is a verified gain correction,
  not a completed sensor colour calibration. The final photo metadata reports
  analogue gain 16× and exposure 54,262 µs; auto-exposure still reaches the
  configured limits in this scene.

The package was transferred through a checksum-verified incremental NAR and
installed with the existing camera installer. No system partition was flashed;
the running kernel still advertises its historical default of 1024, while the
processed camera explicitly writes 256.

## Evidence collected on 2026-10-20

The owner supplied a photo of the same scene from another camera: a neutral
wall, brown cardboard, a black lamp and a green toy. It is a useful visual
reference, not a measured colour chart or an IMX363 sensor profile. Its camera
processing, white balance, exposure and exact viewpoint are not known.

The previous 0.3.2 preview showed pink/green stripes and almost no green in the
toy. Paired RAW/processed captures with libcamera `cam` reproduced the error
independently of Qt.

- The 1920×1080 `SRGGB10_CSI2P` RAW stream has a four-column repeating pattern.
  In one wall region, the first row's column means repeat approximately
  `[182, 146, 335, 280]`; the next row repeats `[337, 281, 242, 234]`.
  This is inconsistent with the advertised two-column RGGB layout.
- The 4032×3024 RAW stream has the expected two-column pattern. Another wall
  region measured approximately `[347, 486, 351, 487]` and
  `[488, 414, 489, 417]`. Ordinary RGGB reconstruction restores the green toy
  without the strong stripes.
- These are different regions/exposures; the values diagnose periodic layout,
  not colour accuracy. A temporary reconstruction averaging paired columns
  also removed the stripes from the small mode, but loses information and is
  not installed as a colour correction.
- The pinned IMX363 driver programs binning register `0x0901` to `0x42` in its
  1920×1080 mode. The full mode disables binning. This narrows the fault to the
  small-mode capture path; the correct register change still needs validation.
- The reported upstream AWB statistics bit-depth issue concerns **unpacked**
  10/12-bit input. Our CSI-2 packed path already computes 8-bit sums; that patch
  does not explain the observed four-column pattern.

## Local workaround

`patches/libcamera-imx363-preview-mode.patch` adds an opt-in
`SPECTRE_IMX363_FULL_SENSOR=1` workaround to the simple pipeline. For IMX363
only, it excludes the faulty 1920×1080 sensor mode. The app's wrapper enables
it by default. Version 0.3.3 verified the colour-layout workaround, but revealed
that this pinned CPU ISP **crops**, rather than scales, smaller outputs. It
delivered 30 fps at the cost of an unacceptable tight view. That intermediate
build must not be treated as the final preview fix.

Version 0.3.4 adds `libcamera-softisp-preview-reduce.patch` and the small
`modules/camera/raw10-reduce.h` helper. With the same opt-in, the 4032×3024
packed RAW10 input is reduced to 1008×756 by averaging 16 samples at each
individual CFA position. The two greens stay separate; averaging happens in
linear 10-bit sensor space before libcamera's black-level, AWB, gamma and
demosaicing. Libcamera then produces a 1000×750 preview with only a small
interpolation border. Full-resolution photos bypass this reduction.

The reduction test uses a known CFA colour ramp with an analytic expected
average, nonzero low packed bits, padded rows and output guards. It passed
AddressSanitizer and UndefinedBehaviorSanitizer on the build server.

### Phone validation

Installed app:
`/nix/store/0vz8ar656n02bkrmlww8lyrmhf3rj109-spectre-camera-aarch64-unknown-linux-gnu-0.3.4`

- Actual displayed preview: approximately 23.8–24.0 updates/s.
- First displayed frame: 369 ms in the sustained-preview run; 300 ms in the
  photo-test run. These are observed warm-machine timings, not a cold-boot SLA.
- Compositor screenshot confirms the broad view, green toy and absence of the
  strong alternating colour stripes.
- Two 3024×4024 JPEG captures and camera close/reopen passed. Preview returned
  about 1.00–1.08 seconds after shutter; JPEG saves took 373–396 ms.
- The final saved photo was inspected against the owner's reference. It is
  still too bright, with clipped highlights and a green/cyan bias, and has more
  noise than the reference. This fixes the broken preview colour layout;
  calibrated white balance, exposure tuning and final colour accuracy remain
  unfinished.

Evidence files include `colour-final-preview.log`, `colour-final-screen.png`,
`colour-final-photo-test.log`, `colour-final-photo.jpg` and its JSON metadata
under the local camera-tests directory. No system partition was flashed.

Setting the environment variable to `0` allows controlled comparisons with the
old mode. Other sensors and applications without the opt-in retain their
normal enumeration. This is a project-specific workaround, not an upstream
libcamera option or a validated kernel repair. No colour matrix is fabricated.

## Original vendor tuning recovered

Source archive on the build server:

`<path-to-miui-rom.zip>`

The archive's `vendor.new.dat.br` and `vendor.transfer.list` were reconstructed
into a temporary filesystem image and inspected with `debugfs`, without
mounting or flashing it. Relevant original files:

| Vendor path | Bytes | SHA-256 |
| --- | ---: | --- |
| `/lib/camera/com.qti.tuned.qtech_imx363.bin` | 5021985 | `ef652a443d51be8dfcd8fef2f3ba5536587c2a9713f08811250f0d9542d61a0a` |
| `/lib/camera/com.qti.tuned.sunny_imx363.bin` | 4915029 | `554d5195774fec8b07166cf83ae37e9a0bca952b01a993f98e73c68e5e174480` |

The archive also contains INDIA variants, sensor module `.bin` files and
`com.qti.sensor.*imx363.so` drivers. Both tuning candidates contain a
`QTI Chromatix Header` and `cc13` block names. They are Qualcomm serialized
tuning data, not ICC profiles or libcamera YAML files. The phone's QTech/Sunny
module identity and the correct tuning-branch selection have **not** been
established. Neither candidate has been applied.

`infra/inspect-camera-tuning.py` is a read-only exploratory decoder for the
observed 1.3 container. It validates section bounds and extracts the named
`mod_cc13_cct_data` records into JSON with source hashes and exact offsets.
The inferred record layout is two temperature-bound floats, nine matrix
coefficients and three offsets. It found 55 QTech branches / 188 records and
45 Sunny branches / 158 records. All extracted matrix rows sum to one within
`7.9e-8`. This is a useful consistency check, **not** validation of the vendor
schema, channel conventions or selection logic. Outputs are
`vendor-tuning/qtech-ccm-candidates.json` and
`vendor-tuning/sunny-ccm-candidates.json` in local camera-tests.

Extracted candidates, the owner's reference, diagnostic raw captures and
comparison images remain local under
`codes/spectre-nix/.code-sync/camera-tests/`, outside source sync and Git.
Server scratch extraction is `/tmp/spectre-vendor-colour/`.

## Guidance from libcamera

- [Sensor model](https://docs.libcamera.org/master/camera-sensor-model.html):
  sensor binning, skipping, crop and output format must describe the actual
  data delivered to the pipeline.
- [ISP feature matrix](https://docs.libcamera.org/master/isp-feature-matrix.html):
  the CPU software ISP supports black-level subtraction, AWB, demosaicing,
  colour matrices, gamma and downscaling. The matrix does not establish that
  this particular sensor has calibrated tuning.
- [Maintainer review of colour calibration](https://lists.libcamera.org/pipermail/libcamera-devel/2026-February/057561.html):
  colour matrices require measured calibration; rows should sum to one to
  preserve neutral balance rather than introduce arbitrary channel gain.
- [CPU CCM performance discussion](https://lists.libcamera.org/pipermail/libcamera-devel/2024-December/047182.html):
  CCM has material CPU overhead on tested platforms. Measure this phone rather
  than assuming that adding a matrix preserves preview speed.
- [Packed/unpacked statistics distinction](https://patchwork.libcamera.org/cover/26256/):
  the proposed normalization fix targets unpacked 10/12-bit statistics.

Current master documentation and mailing-list proposals may describe features
newer than our pinned libcamera 0.7.0. Check the pinned source before adding
tuning keys. The next calibration steps are to identify the module, validate
its black level and neutral response, and obtain chart captures under known
lighting (or verifiably decode the matching vendor tuning). A same-scene
consumer-camera image can check plausibility but cannot establish a general
colour profile.

## 2026-10-20: measured black level and CPU ISP processing order

The owner covered the rear lens with an opaque object for a fresh measurement.
At exposure 3126 lines and **unity digital gain 256**, raw-only capture retained
explicit analogue codes 384 (4×) and 480 (16×). Three full 4032×3024 packed
RAW10 frames were collected per setting; statistics below use the last frame.
These are current hardware measurements, not values inferred from vendor files.

| Analogue gain | R mean | Gr mean | Gb mean | B mean |
| --- | ---: | ---: | ---: | ---: |
| 4× | 63.920 | 64.004 | 63.990 | 63.909 |
| 16× | 64.044 | 64.012 | 63.890 | 64.128 |

All channel medians were 64. This supports a common RAW10 black pedestal of 64
(the libcamera 16-bit tuning value is 4096), not unequal channel offsets to
compensate for a pink cast. Noise increases with analogue gain, as expected.

The pinned CPU ISP built its gamma table with black subtraction, then indexed
it with white-balanced RAW values. This applies gains **before** subtracting
the sensor pedestal. At non-unity white balance, neutral dark samples become
coloured. The error is independently described in the official libcamera
[black-level patch review](https://lists.libcamera.org/pipermail/libcamera-devel/2026-May/058613.html).

`patches/libcamera-softisp-black-level.patch` adapts that correction to our
pinned 0.7.0: subtract/clamp black, apply gains/matrix, then gamma. In the CCM
branch each input column uses its own channel offset, including off-diagonal
terms. No vendor CCM or scene-fitted colour profile was invented. Lookup tables
keep the existing per-pixel processing path.

`modules/camera/test-black-level.py` extracts the actual production LUT methods
and compiles them with the upstream RGB/Matrix types on the build server.
AddressSanitizer/UBSan tests pass for black neutrality, non-unity white balance,
monotonicity, red/blue swapping and unequal black offsets with an off-diagonal
CCM. The same test against unpatched source fails the black-neutrality assertion.

Camera 0.3.6 was built on the build host, imported through Nix and tested on the
phone. Two real photos plus close/reopen passed. First displayed preview was
290 ms in the scene test; sustained displayed preview was about 23.8–23.9 fps.
The formerly purple black objects are visibly darker and much more neutral in
the new scene photo. Broad wall/lamp clipping and imperfect colour rendering
remain; this test does not establish full sensor colour calibration.

Evidence: local `.code-sync/camera-tests/colour-processing/`, including
`dark-statistics.json`, raw last frames, production-LUT test logs (patched and
unpatched), `scene-test.log` and `IMG_20261020_195715_803_7efd320b.jpg` plus metadata.
The covered-lens before/after images had unity AWB, so they establish sensor
black/noise but do not themselves demonstrate removal of a non-unity-AWB cast.

### IMX363 highlight protection (0.3.7)

`modules/camera/imx363.yaml` pins the measured 4096 black level and opts into
`highlightQuantile: 0.95`, `highlightTarget: 0.80`. These are project additions
to the pinned simple IPA, implemented by
`patches/libcamera-softisp-highlight-guard.patch` and `highlight-guard.h`.
They are not generic upstream tuning keys. Other sensor tuning files retain
legacy exposure behavior unless they opt in.

The guard examines the 95th percentile of linear, black-subtracted luminance.
Above target plus 0.04, it requests the existing bounded 10% exposure/gain
reduction. Near target it suppresses further brightening; below target minus
0.04, normal AGC resumes. This allows small light sources to clip but protects
broader bright areas. Existing exposure decreases always win. The config is
validated, empty histograms do not alter normal AGC, and black frames can still
brighten. This is an exposure policy, not HDR or a calibrated colour matrix.

`test-highlight-guard.cpp` passed AddressSanitizer/UBSan on the build server:
empty/dark/saturated inputs, small specular areas, broad clipping, disabled
configuration, and the hold/decrease behavior near the threshold. Both server
ARM builds and native Qt RGB/QML/rotation/JPEG/PNG/reopen tests passed.

Final phone validation of 0.3.7:

- Installed app:
  `/nix/store/l1d4j9b7687jjd2j2z33as2gjqjwlmyd-spectre-camera-aarch64-unknown-linux-gnu-0.3.7`.
  The managed desktop entry points to this version, with its Nix GC root retained.
  No system image was flashed and the running kernel was not changed.
- First displayed preview: **288 ms** on the warm phone. Sustained sensor and
  displayed preview: **23.9–24.0 fps** before the session automatically locked.
  The locked-session photo test is not evidence of displayed frame rate.
- Settled analogue gain decreased from 16× to **8.126985×**; exposure stayed
  54,262 microseconds and digital gain stayed 256. White-balance metadata became
  `[2.174509, 1.400036]`; black-level metadata remained `[4096,4096,4096,4096]`.
- Two 3024×4024 JPEGs and camera reopen passed. Final observed preview resume:
  998 ms after shutter; JPEG encoding/save: 407 ms.
- Final image: `IMG_20261020_200237_123_8e713b7f.jpg`. Compared with the 0.3.6
  scene capture, the fraction of pixels with any channel >=250 decreased from
  **17.05% to 5.83%**, and all-channel >=250 from **11.08% to 4.50%**. These are
  descriptive JPEG statistics from two scene captures, not a controlled RAW
  dynamic-range measurement; small framing/light differences remain possible.
- Direct visual inspection shows much less purple in dark objects and more
  neutral whites. The lamp still clips, and the toy's colour is less saturated
  than the owner's reference. The sensor colour matrix, lens shading, and
  noise reduction are still uncalibrated; this is not a finished colour profile.
- The phone locked before the final compositor screenshot, so the stalled
  screenshot was cancelled. Saved photo tests succeeded. The corrected normal
  app was left running for the owner to unlock and inspect.

Evidence includes `highlight-preview.log`, `highlight-hardware-test.log`,
`scene-highlight-statistics.json`, build/native/test logs, final JPEG/JSON,
and the local artifact manifest `artifacts/qt-camera-highlight/closure.json`.
To reproduce the standalone tests, run on the build server:

```bash
python3 modules/camera/test-black-level.py /path/to/pinned/libcamera-source
# The same command with --unpatched must fail the black-neutrality assertion.
g++ -std=c++17 -O1 -g -fsanitize=address,undefined \
  modules/camera/test-highlight-guard.cpp -o /tmp/test-highlight
/tmp/test-highlight
```

### Sunny stock colour matrix (0.3.8)

Later on 2026-10-20, two identical camera EEPROM reads established vendor ID 1,
matching **Sunny**, rather than QTech's ID 6. This supersedes any earlier
actuator-based module guess. [Native Xiaomi research](camera-xiaomi-research.md#sunny-module-and-native-colour-processing)
records the original binary hashes, REA/Ghidra function addresses and bounded
Chromatix decoder. `modules/camera/imx363-sunny-profile.json` records the exact
stock matrix provenance; `imx363.yaml` applies it after AWB and before Adjust.

This is one constant base IPE CC13 matrix from the first DRC/HDR/no-flash/AEC
branch, whose two CCT records have identical coefficients. It is **not** a port
of Qualcomm's full exposure/mode/CCT selection. No scene-fitted or guessed
matrix was substituted. Generic libcamera AWB remains, and EEPROM AWB ratios,
lens shading, noise reduction and autofocus have not been ported.

The first matrix-only trial strengthened the toy's green and cardboard's
brown, but turned saturated highlights pink. The software ISP combined white
balance and CCM without bounding intermediate white-balanced values. Large
red/blue values then contributed excessive negative terms to the green row.
`libcamera-softisp-ccm-highlight.patch` clamps each input after white balance
and before colour mixing, accounting for gains already included in the combined
matrix. This is an explicit project highlight policy, not a claim to duplicate
Qualcomm's implementation. It runs during LUT construction, without adding
operations to the per-pixel loop, and leaves unsaturated values unchanged.

Validation on the build host:

- `infra/test-camera-profile.py` matched the original Sunny file hash, matrix,
  child path and byte offset; checked Sunny/QTech module IDs; rejected a
  truncated file, non-finite coefficient, bad child count and invalid mode parent.
- `modules/camera/test-black-level.py` exercised the actual production LUT
  methods under AddressSanitizer/UBSan: existing black/WB tests plus saturated
  white neutrality and unsaturated colour preservation with the Sunny matrix,
  including swapped RGB storage. All passed.
- The same test with `--without-highlight-fix` failed the saturated-white
  assertion, confirming sensitivity to the observed problem.
- ARM camera/libcamera build completed on the build host. Two missing store paths,
  3,519,824 bytes of NAR, were transferred and checksum-verified before import.

Installed application:
`/nix/store/0ppfvg2pwl9kyzypd37wrbg4j5286w5m-spectre-camera-aarch64-unknown-linux-gnu-0.3.8`.
Libcamera:
`/nix/store/j61858v238jfydj69r0jf739nmgy8sdg-libcamera-aarch64-unknown-linux-gnu-0.7.0`.
The managed desktop launcher and GC root point to the new app. The previous
launcher is retained as `.previous`. No system image, boot partition or kernel
was changed. The launcher/import survive reboot; `/tmp` trial profiles are not
required.

Phone observations:

- Two full-resolution JPEG saves with intervening camera reopen passed.
- Final photo `IMG_20261020_202124_608_9f48a862.jpg`: 3024×4024, rotation 90°;
  metadata reports the exact nine stock coefficients, measured black 4096,
  analogue gain 10.24, exposure 54,262 µs and WB gains 2.062005/1.346473.
- Warm first preview available at 257 ms and first visible presentation at
  305 ms. Steady acquisition and presented updates approximately 21.5 fps.
  This is slower than the previous roughly 24 fps; the new CCM costs CPU time.
- Full-resolution capture/preview restart approximately 1.55 s, first resumed
  preview 1.70 s, JPEG encoding/save 451 ms.
- Visual inspection shows stronger greens/browns and neutral blown highlights.
  Noise, green/cyan bias and loss of detail in the lamp-lit wall remain. This
  is an incremental colour improvement, not camera-quality parity with the
  owner's reference camera.

A descriptive highlight check counted pixels with R/B > 240 and G < 225:
44,924 in the matrix-only trial, 12 after bounding WB inputs (of 12,168,576).
Median green where R/B > 240 returned from 244 to 254. The phone framing moved
before the final capture; these are descriptive diagnostics, not a registered
pixel comparison or a colour-accuracy measurement. The reference photo was
used qualitatively, not to claim a measured Delta-E calibration.

Local excluded evidence: `.code-sync/camera-tests/vendor-profile/` contains
baseline/trial/final JPEG+JSON, build/export/install logs, hardware/benchmark
logs, `highlight-comparison.json`, matrix provenance and private EEPROM reads.
Artifact manifest: `artifacts/qt-camera-sunny/closure.json`; installation record:
`artifacts/qt-camera-sunny/installed.json`. Original native binaries and full
REA reports remain on the build host in `camera-research/`.
