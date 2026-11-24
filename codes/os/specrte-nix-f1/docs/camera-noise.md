# IMX363 noise reduction

Implemented and deployed 2026-10-21 in Spectre Camera 0.4.0. Final phone
measurements and installed paths are recorded below. This is a new software
pipeline; it does not claim to port Qualcomm's hardware BPS/IPE algorithms.

## Processing stages

1. The libcamera software ISP filters full-resolution packed RGGB RAW10 with
   a same-CFA 3×3 range-gated neighbourhood (two-pixel spacing). R, Gr, Gb and B
   never mix. Thresholds use a gain-dependent noise model before demosaic,
   white balance, colour matrix and gamma. Source buffers remain immutable.
   Four bounded workers process independent output rows; range-gated weights
   use fixed normalization to avoid per-pixel division.
2. Preview retains its existing 16-sample same-CFA RAW averaging. A small
   guided chroma filter with a single guided lookup per pixel operates on the processed preview; no temporal frame
   history or additional startup frames are required for preview.
3. Still capture retains up to three full-resolution processed frames. It
   rejects exposure/gain changes above 3.5% and white-balance changes above 5%.
   Capture starts after at least six frames and a stable exposure/WB pair,
   with an eight-frame warmup fallback. A later stable pair replaces a
   mismatched early reference. Burst collection stops after eight attempts or
   once five seconds have elapsed at a completed capture; configuration and
   frame processing can exceed that time, so it is not a hard shutter deadline. Insufficient matching frames
   fall back to fewer frames, including a single reference.
4. A coarse global translation search, local block search and full-resolution
   refinement align the burst. Poor block matches and inconsistent pixels are
   excluded. Accepted pixels are averaged in linear sRGB. This handles bounded
   translation; it is not an optical-flow/rotation/rolling-shutter solution.
5. Guided chroma denoising uses stronger still settings and preserves the
   reference brightness channel apart from rounding/gamut clipping. The
   half-resolution chroma grid is guided back to full resolution using colour
   and brightness differences to reduce bleeding across edges.
6. Preview capture resumes before burst processing and JPEG/PNG encoding run
   asynchronously. Only one still job is active; retained bursts are capped at
   three images. Metadata records frame counts, accepted blocks, blended pixel
   samples, gain, processing time and fallback errors.

`SPECTRE_DENOISE_OFF=1` disables app denoising and burst collection for diagnosis.
`SPECTRE_IMX363_RAW_DENOISE=0` separately disables the upstream RAW filter. Both
are necessary for an unfiltered comparison. Normal launch enables both stages.
The RAW stage is restricted to the opted-in full 4032×3024 SRGGB10 sensor mode.
Preview buffers are recycled before filtering to permit overlap with the next ISP frame.
Its gain comes from the IPA's latest observed sensor gain, so rapid exposure
changes may precede corresponding filter parameters by pipeline latency.

## Measured noise profile and limits

`modules/camera/calibrate-noise.py` consumes pairs of original packed frames.
`imx363-noise-profile.json` records input hashes, per-CFA measurements, signal
bins, control provenance and limitations. `noise-profile.h` is generated from
that report and shared by the RAW and app builds.

The retained 2026-10-20 captures have unity digital gain 256 and fixed controls:

- Covered lens at analogue gains 4× and 16×, exposure 3126 lines.
- Stationary illuminated scene at 4×, exposure 1000 lines.
- Each analysis uses the last two consecutive frames; temporal difference
  variance divided by two removes static scene/fixed-pattern structure.
- Measured RAW10 read variances at 4×: R 1.502, Gr 3.051, Gb 3.053, B 1.501.
- At 16×: R 13.250, Gr 26.057, Gb 26.049, B 13.142.
- Shot-noise slopes at 4×: R 0.5273, Gr 0.5263, Gb 0.5316, B 0.5298.

Read variance is modelled as affine in gain squared; shot slope scales with
analogue gain. Gain is bounded to 1–16×. Other gains are interpolated or
extrapolated, not independently measured. A controlled flat-field sweep across
exposure/gain and temperatures is still needed for full calibration. The
illuminated scene fit rejects bins with a mean frame difference above one RAW
code and records sample counts; it cannot prove the absence of all subtle
motion. RGB filtering thresholds approximate propagation through gamma and
include empirical allowances for AWB/CCM. They are not factory tuning.

## Validation

The build-server test suite exercises the production filters, including RAW
buffer bounds, padding, CFA separation, strong edges, gain scaling, chroma
variance reduction, brightness preservation, burst noise reduction, known
translation and moving-object rejection. Existing Qt rendering, rotation and
JPEG/PNG save/reopen tests remain part of `make qt-camera-test`.

RAW tests run under AddressSanitizer and UndefinedBehaviorSanitizer. On the
synthetic flat RAW fixture, the final filter retains 0.286892 of the input
squared error. This is a regression metric, not a real-photo improvement claim.

Official upstream context: libcamera's proposed [software ISP temporal denoise](https://patchwork.libcamera.org/patch/28103/)
uses a signal-dependent noise model and motion gating. That proposal targets
its GPU debayer and is not an enabled feature of our pinned CPU pipeline.

Local capture/build evidence: `.code-sync/camera-tests/denoise/`. Original
calibration pairs and generated artifacts remain outside source sync.

## Initial measured validation (2026-10-21)

- Build-server production tests pass: RAW memory/edge tests with sanitizers,
  colour variance and brightness checks, static burst, known translation,
  ambiguous repeated-pattern rejection and moving-object rejection.
- Native ARM phone noise self-test passes with identical metrics: chroma
  squared-error ratio 0.09418 and burst luminance squared-error ratio 0.25817
  on deterministic synthetic fixtures. These do not claim real-photo ratios.
- Controlled production-filter comparison on the same real baseline JPEG:
  selected wall/cardboard/phone-body regions retained 0.1795 / 0.0924 / 0.1330
  of their original high-frequency chroma variance. A 2×2 mixed-difference
  operator measures this metric; texture and JPEG artifacts also contribute.
  Mean absolute brightness-channel change across the image was 0.284/255.
- Both server builds and the existing Qt rendering/rotation/repeated JPEG/PNG
  capture/reopen tests passed. ARM phone installation uses the standard NAR
  installer; no kernel or phone partition was flashed.

Initial installed app:
`/nix/store/v2fpik9v6mq5a3jmrdqxsr9kbk9v4562-spectre-camera-aarch64-unknown-linux-gnu-0.4.0`.
Software ISP:
`/nix/store/z2z7cmaycbnq89lsx1rw0aydh9nlsnmd-libcamera-aarch64-unknown-linux-gnu-0.7.0`.

## Final deployed verification (2026-10-21)

Installed app:
`/nix/store/y5sj8liwiwclm18ry7bwvhhn0favvwfb-spectre-camera-aarch64-unknown-linux-gnu-0.4.0`.
Software ISP:
`/nix/store/pxnz4wkanflhlz26ahkfp7z3s5y5bnzf-libcamera-aarch64-unknown-linux-gnu-0.7.0`.
The managed desktop entry and GC root target this app. Build artifacts are local
under `artifacts/qt-camera-denoise-settled/`; all compilation ran on the build host.

Two consecutive real shutter captures succeeded with the camera visible and
the keyboard hidden. Both saved 3024×4024 JPEGs at gain 16× and exposure
54,262 µs, merging three of three frames with all 6,048 tested alignment tiles
accepted. Colour gains were approximately R 2.59 / B 1.58 on both captures.
They visibly reduce coloured speckling against the earlier 0.3.9 baseline.
Framing and light can change between captures, so this is not a controlled
sensor-noise percentage or a claim of Xiaomi-equivalent quality.

| Measurement | Final observation |
| --- | --- |
| First preview available / presented | 314 / 366 ms on this launch |
| Steady visible preview | about 20.3 fps; all-denoise-off same-scene comparison about 20.5 fps |
| Shutter to resumed preview | 3.594 s / 3.523 s |
| Burst and chroma processing after preview restart | 4.456 s each |
| JPEG encoding and atomic save | 306 / 300 ms |
| Approximate shutter to completed save | 8.2 s each; preview operates during processing |
| Peak resident memory after two captures | 460,036 KiB (about 449 MiB) |

Final source tests pass on the server, including RAW ASan/UBSan checks and Qt
rendering, rotation, repeated JPEG/PNG saves and reopen. Deterministic final
filter ratios: RAW squared error 0.286892, still chroma 0.0941784, preview chroma
0.604002 and burst luminance error 0.258174. Synthetic fixtures validate code
behaviour; they do not establish real-scene detail retention in every case.

During validation, shortening warmup to three frames caused a green image:
libcamera's initial unity white-balance values appeared stable before statistics
arrived. The deployed version restores the six-frame minimum and can replace
an early burst reference when a later pair settles differently. Six frames
remain a practical guard for this pipeline, not proof of convergence under
all lighting or motion conditions.

Final evidence in `.code-sync/camera-tests/denoise/`:

- `settled-final.jpg` / `.jpg.json`: phone `IMG_20261021_063321_786_0633d959.jpg`.
- `settled-repeat.jpg` / `.jpg.json`: phone `IMG_20261021_063401_104_5032ef46.jpg`.
- `phone-settled.log`, `final-verification.json`, `build-settled-native.log` and
  `build-settled-arm.log` retain timing, metadata and build evidence.
- `baseline.jpg` is the earlier noisy image. `final.jpg` is the rejected early
  warmup test with unity WB; it must not be presented as the deployed result.

Remaining quality limits include incomplete colour/temperature noise calibration,
no lens-shading correction or hot-pixel map, bounded translational alignment,
and the CPU processing cost above. Existing stock-derived colour tuning remains
in use, but these new denoising stages do not execute Xiaomi's proprietary MFNR
or HDR implementation. RAW filtering is internal; ordinary photos save as JPEG.
