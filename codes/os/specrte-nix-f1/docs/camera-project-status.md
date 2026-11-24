# Spectre Camera project status

Audit date: 2026-10-21. This page is the current-state map for the owner’s Poco F1 rear camera (beryllium, Sony IMX363, Sunny module). It reconciles the supplied documentation with the 2026-10-21 checkpoint in [camera-pipeline-progress.md](camera-pipeline-progress.md). Dated measurements in the other pages stay historical evidence for the version that produced them.

This audit had the supplied Markdown only. It did not rebuild packages, boot a kernel, read source, or repeat phone tests.

## Current deployment

| Item | Verified state on 2026-10-21 |
| --- | --- |
| Phone application | Spectre Camera **0.4.4**, store path `/nix/store/jqyydjqykllfcxlhjcpp8xwbkqjhp0n3-spectre-camera-aarch64-unknown-linux-gnu-0.4.4` |
| libcamera | Pinned **0.7.0** simple pipeline and CPU software ISP. The 0.4.4 notes do not publish a new libcamera store hash; the last hash written beside a phone install is the 0.4.0 ISP path in [camera-noise.md](camera-noise.md). |
| Running kernel | Still `7.1.0-rc1` as previously running. `CONFIG_VIDEO_AK7375` still disabled on the phone. No boot, system, or userdata partition was flashed for 0.4.x. |
| Autofocus kernel | Built on the mini PC as `/nix/store/67mzws0bhsycgayij5p7pxmrx3bq8h3s-linux-aarch64-unknown-linux-gnu-7.1.0-rc1`. Packaged as a RAM-only test image under `artifacts/sunny-focus-boot/`. **Not transferred with fastboot, installed, or booted.** Autofocus is not operational. |
| HDR helper | Bundle `compat-0.1.1`. Fixture and phone synthetic tests passed on 2026-10-20. **The Qt app does not call it.** |
| Stock equivalence | **No complete Xiaomi / CamX / CHI / ArcSoft equivalence.** |

Managed launcher and GC root for 0.4.4 point at that package. Installation uses incremental Nix NAR import. Installers take a prompted password or an explicitly named environment variable; access notes may contain owner-supplied credentials.

## Architecture

```text
IMX363 (Sunny EEPROM, vendor ID 1)
  -> qcom-camss, packed SRGGB10, full mode 4032×3024
  -> libcamera 0.7.0 simple pipeline + CPU software ISP
       unity digital gain 256 on processed paths
       measured black level
       one constant Sunny IPE CC13 matrix
       highlight-guard exposure policy
       opt-in full-sensor preview reduction (binned 1920×1080 mode excluded)
       149-point bright-defect map (this module)
       colour-only lens-shading mesh (NEON on ordinary ARM processing)
  -> Qt 6 Quick Spectre Camera 0.4.4
       small preview in the scene graph
       stills: up to three aligned processed frames, chroma denoise,
       asynchronous JPEG quality 100 or lossless PNG
```

Separate, not on the shutter path:

```text
Three processed YUV frames + explicit AE words
  -> compat-0.1.1 (private PID namespace, Android linker, Bionic)
  -> original libarcsoft_high_dynamic_range.so
  -> tight YUV + one JSON record
```

Manual exposure, analogue gain, and colour gains exist in the libcamera request path and in `spectre-capture-probe` (verified 0.4.2). The Qt shutter does not build HDR brackets and does not call the ArcSoft worker.

The running kernel still binds the actuator address to **BU64748** and exposes no lens V4L2 subdevice. EEPROM describes **`beryllium_imx363_ak7374_sunny` at I²C 0x0c**. Those two facts disagree. The AK7374 patch is Sunny-specific and is not running.

Megapixels remains a separate raw/still bring-up (2026-10-20). Daily Spectre photos go through the Qt app.

Original Android architecture (static ROM evidence, 2026-10-20) is in [camera-pipeline.md](camera-pipeline.md) and [camera-pipeline-inventory.md](camera-pipeline-inventory.md): 45 Titan use cases / 115 pipelines, 36 compiled Xiaomi ZSL pipelines, ArcSoft HDR and SuperNight plugin bindings. That graph is not what NixOS executes.

## Version sequence

| Version | What that deployment showed | Where |
| --- | --- | --- |
| Megapixels fixes | Rear RAW streaming on the existing kernel; config and V4L2 queue fixes. Blank-frame photo-save regression was still in progress in the write-up. | [camera.md](camera.md) |
| 0.2.0–0.3.0 | libcamera software ISP, full-frame preview later replaced, rotation experiments, digital-gain **zero restored to driver default 1024**. | [qt-camera.md](qt-camera.md) |
| 0.3.1–0.3.2 | Portrait pixel rotation, persistent offset, QML URL content hash so disk cache cannot keep an old UI. | [qt-camera.md](qt-camera.md) |
| 0.3.4 | Opt-in skip of broken 1920×1080 binning; 4×4 same-CFA preview reduce. 0.3.3 cropped instead of scaling and is not the preview fix. | [camera-colour.md](camera-colour.md) |
| 0.3.5 | Processed paths write digital gain **256**. Fixed-exposure RAW slope 1024 vs 256 was **3.99034**. Kernel patch exists; live kernel was not replaced. | [camera-colour.md](camera-colour.md) |
| 0.3.6 | Black subtraction before gains/gamma. Covered-lens pedestal **64** RAW10 (libcamera 16-bit value **4096**). | [camera-colour.md](camera-colour.md) |
| 0.3.7 | Highlight quantile 0.95 / target 0.80 exposure policy. | [camera-colour.md](camera-colour.md) |
| 0.3.8 | One Sunny base CC13 matrix plus WB clamp before mixing. Preview fell to about **21.5 fps**. | [camera-colour.md](camera-colour.md) |
| 0.3.9 | Shutter no longer opens the Stevia text input. Included in later versions. | [qt-camera.md](qt-camera.md) |
| 0.4.0 | Gain-dependent RAW denoise, three-frame motion-gated merge, preview/still chroma filters. Visible preview about **20.3 fps**. Shutter-to-save about **8.2 s**. | [camera-noise.md](camera-noise.md) |
| 0.4.1 | **149** repeatable bright defects on this module. Preview about **20.1 fps** in that test. | [camera-pipeline-progress.md](camera-pipeline-progress.md) |
| 0.4.2 | Manual exposure/gain/WB. Probe matched metadata at preview and full resolution. Shutter still does not bracket. | [camera-pipeline-progress.md](camera-pipeline-progress.md) |
| 0.4.3 | Colour-only shading. Wall R/G RMS **11.14% → 1.91%**, B/G **2.94% → 1.74%**. Visible preview about **17.8 fps**. | [camera-pipeline-progress.md](camera-pipeline-progress.md) |
| **0.4.4** | Same shading on ARM NEON (scalar 6.14 ms vs NEON 1.46 ms per 1008×756 frame in a 30-frame benchmark). Visible preview about **19.5 fps**. Full-resolution denoise still about **4.3 s**. | [camera-pipeline-progress.md](camera-pipeline-progress.md) |
| compat **0.1.1** | Original three-frame HDR engine on synthetic NV12/NV21. Phone full-frame worker time about **335–353 ms**, excluding capture and the Qt app. | [camera-xiaomi-compat.md](camera-xiaomi-compat.md) |

Treat each fps and latency figure as that warm-session observation. None of the docs establish a cold-boot SLA.

## Calibration (this Sunny module)

- **Module identity.** Two identical 256-byte EEPROM reads, vendor ID **1** (Sunny). QTech’s descriptor expects **6**. Prefix SHA-256 `40da223c4ad7839bf03811e46df520232eda8cf2eb9ef62ef4e1d6ecab7d8cef`. Full 8192-byte reads matched, SHA-256 `80ba43461c6259aa823c71461bc25be7998c49ad3e557ec1a1ca8dd72172c696`. No EEPROM checksum algorithm has been recovered. Raw bytes stay in excluded research storage.
- **Digital gain.** Stock helpers encode unity as **256**. The pinned Linux default remains **1024** (4×) and is still what the running kernel advertises. Processed camera configurations write 256. A future-build kernel patch exists and is not flashed.
- **Analogue gain.** Stock encoding `512 - 512/gain`, code cap 480, range 1–16×, matches the libcamera helper patch.
- **Black level.** Covered lens, unity digital gain, analogue 4× and 16×: channel means about **63.9–64.1**, medians **64**. YAML uses 16-bit **4096**. ISP subtracts black before gains and gamma from 0.3.6 onward.
- **Colour matrix.** One constant Sunny IPE CC13 matrix from the first DRC/HDR/no-flash/AEC branch (both CCT records identical). Coefficients:

```text
 1.68869996  -0.77240002   0.08370000
-0.19410001   1.39740002  -0.20330000
 0.04450000  -0.75260001   1.70809996
```

  Provenance: Sunny tuning SHA-256 `554d5195774fec8b07166cf83ae37e9a0bca952b01a993f98e73c68e5e174480`, root symbol 43, mode 3, child symbols 568→572, first float at file offset **471676**. This is not Qualcomm’s full CCT/exposure/LED selector. Generic libcamera AWB remains. The `ct: 5000` knot is a lookup anchor, not a measured scene temperature.
- **Highlight policy.** Quantile exposure guard (0.3.7) plus clamp of white-balanced inputs before the CCM (0.3.8). Neither is HDR.
- **Defects.** 149 isolated bright sites agreed across four covered-lens frames at gains 4× and 16×. Median of eight same-CFA neighbours. Enabled only for full 4032×3024 SRGGB10 when `SPECTRE_IMX363_SUNNY_DEFECTS=1` (owner launcher default). Cold, dead, and temperature-dependent defects are not calibrated.
- **Noise.** Pairs at unity digital gain: covered lens 4× and 16×, plus one stationary lit scene at 4×. Read variance and shot slope feed the RAW filter. Other gains are interpolated. Not a full flat-field sweep.
- **Lens shading.** Colour-only mesh from golden/module LSC34 (root 9, symbols 432→437, golden 441), centre-normalized by mean green. `SPECTRE_IMX363_SHADING=colour` on the owner launcher. `full` is experimental and not the deployed mode. One accepted wall set (exposure 1383 lines, analogue code 256, digital 256). Brightness shading, chromatic aberration, and trigger selection are not validated.
- **EEPROM focus and WB numbers (decoded, not applied as a running AF stack).** Infinity code 370, macro 859, margins 0.3 / −0.15 recorded and not used as new physical limits. WB ratios R/G 548/1024, B/G 682/1024, Gr/Gb 1026/1024. Illuminant mapping through stock AWB is unresolved. LSC OTP is 17×13 per channel; generic stock EEPROM LSC enable is false.
- **Preview geometry.** 1920×1080 mode programs binning register `0x0901` to `0x42` and showed a four-column pattern. Full mode matches two-column RGGB. The app excludes that binned mode and reduces 4032×3024 by averaging 16 samples at each CFA site (about 1008×756, then about 1000×750 after the ISP border). The 1024×768 preview size belongs to 0.3.2, not to 0.4.4.

## Reverse engineering

ROM: `miui_POCOF1Global_V12.0.3.0.QEJMIXM_cf3fccffce_10.0.zip`, SHA-256 `aeb05ec30c8feccc65d2152cfa8bdbbdc9800874aff4c570057d9326e9038630`. APK `com.android.camera` 4.2 / version code 5, SHA-256 `126b9b96e1541d20171ccec57f463f1f75d195d72a9bbd76270f80ffe495539b`. Extraction used read-only `debugfs` on reconstructed images. Placeholders `<BUILD_HOME>`, `<BUILD_HOST>`, and `<IP_REDACTED>` stand in for server location and are not filled here.

Tools recorded in the docs: REA 6.7.0, JADX headless MCP 0.7.1, Ghidra 12.1.4, Node 26.12.0. Ghidra addresses in the HAL/CHI notes include image base `0x10000`; ELF inventory addresses do not.

Recovered and still applicable:

- Camera2/vendor-tag methods do not embed an IMX363 matrix.
- Chromatix 1.3 container, CC13 tree, Sunny vs QTech module IDs.
- CHI feature selection and the compiled ZSL catalogue. Generic Titan HDR uses `com.qti.node.memcpy`; Xiaomi’s compiled merge uses `com.arcsoft.node.hdr.so`. SuperNight uses `com.arcsoft.node.superlowlight.so`.
- HDR ABI: 44-byte image, NV12 `0x801`, NV21 `0x802`, three frames, six AE words. Test vector `[50, 1, 1, 1, 8, 4]` is the node fallback, not measured AE. Engine hash `3e2a5db30a41b17e28d79ace12b60c5df0f7bc4c6b8688a403f624d18c65df0a`.
- Low-light HDR and SuperNight libraries were hashed and are not wrapped.

Decompiler output is interpretation. Several caller conventions and register addresses stay unresolved (digital-gain register addresses outside `FillExposureSettings`, AEC trigger semantics, JPEG main/thumbnail roles, GPU node property 12).

## Testing that the docs actually record

Server-side, where the pages say so: native Qt RGB/stride/rotation/QML/JPEG/PNG/reopen; ASan/UBSan on black level, highlight guard, preview reduce, defects, shading scalar-vs-NEON, noise filters; Chromatix/LSC/EEPROM/profile decoders; HDR `test.py` under a PID namespace and QEMU user ARM for bundle `compat-0.1.1`.

Phone-side highlights that remain the latest evidence for each claim:

- 0.4.4 warm preview about 19.5 fps; first availability/presentation 336/383 ms; shading photo `IMG_20261021_075226_647_286d3497.jpg`; denoise about 4355 ms; save 243 ms; keyboard stayed hidden.
- 0.4.2 manual probe: exposures 12000/11994, 24000/24006, 6000/6006, 12000/11994 µs and gains 2, 2, 2, 4 observed; at least four matching frames each; AE resumed afterward.
- 0.4.1 defect photo and 20.1 fps preview. The 0.4.4 session did not repeat that defect protocol.
- 0.4.0 denoise settled pair at gain 16×, exposure 54262 µs, 6048/6048 alignment tiles.
- compat 0.1.1 full-frame synthetic hash `fb755aa601625ffd79572d0d9765ca82ffdf9edd67e94fa20230fcfdad542072`, inputs unchanged.
- Unity-gain, black-level, highlight, and Sunny-matrix sessions on 2026-10-20, each with its own store path in [camera-colour.md](camera-colour.md).

Synthetic HDR, synthetic denoise ratios, and single-scene JPEGs are not photo-quality or Delta-E proof.

## Limitations

- Picture quality is not factory-matched. Noise, colour casts, clipped lamps, and uncalibrated tone/detail remain visible in the scene reviews.
- Colour-only shading is not full LSC. One wall scene is not a certification.
- Defect map is one module, bright sites only.
- Burst align is bounded translation, up to three frames, not stock MFNR (five/eight-frame helper, hardware BPS/IPE).
- HDR worker is isolated from the app. Low-light HDR and SuperNight are not implemented. AE words, frame order, and YUV range for real brackets are not validated.
- Autofocus code is compiled only. No lens subdevice, sweep, or focus algorithm on the phone.
- Binned-mode register repair is not done; the workaround avoids the mode.
- `test_pattern=2` still produced black frames in the early libcamera diagnosis; the control was restored to 0. That check is unresolved.
- Megapixels blank-frame photo-save retest has no recorded pass or fail after “in progress”.
- Persisted `rotationOffset` after the 0.4.4 install is not stated. The 0.3.2 rotation test left the setting at 90. Earlier 0.3.1 owner requests moved 270, 0, 270, and 0.
- Reboot-to-fastboot returned to ordinary Linux. USB ID 18d1:d001 was Mobile NixOS CDC networking, not fastboot.
- Delegated source review under `.code-sync/grok/` is hypothesis only. Preferred model identity was not recorded. The overlapping-shutter report did not reproduce: `CameraController::capture()` refuses a second capture while `pending` is true.
- Redacted host, IP, and password paths are intentional. This audit does not guess them.

## Next evidence gates

1. With owner authorization for the hardware test, boot the already built AK7374 image only through a verified recoverable RAM path; confirm the lens subdevice; then a position sweep between EEPROM codes 370 and 859. Until then, autofocus stays off.
2. Real HDR brackets: measured AE words, order, YUV range, and an asynchronous Qt path. Fixture success is not that gate. Keep low-light HDR and SuperNight out until the three-frame HDR path is honest.
3. Colour chart under known light, or a verified branch selector. EEPROM WB ratios need an illuminant map before use. Full-brightness shading, CA, and LSC triggers need their own captures.
4. Defect classes beyond the 149 bright points; noise across gain, exposure, and temperature.
5. Close or explicitly abandon the Megapixels blank-frame save test. Record the live rotation offset on 0.4.4 without assuming the 0.3.2 test value.
6. Do not describe CamX, CHI, or ArcSoft presence as stock photo equivalence.

## Where the detail lives

| Topic | Document |
| --- | --- |
| 0.4.1–0.4.4 checkpoint | [camera-pipeline-progress.md](camera-pipeline-progress.md) |
| App history, rotation, keyboard, early gain | [qt-camera.md](qt-camera.md) |
| Colour, black level, CCM through 0.3.8 | [camera-colour.md](camera-colour.md) |
| Noise 0.4.0 | [camera-noise.md](camera-noise.md) |
| Stock graph | [camera-pipeline.md](camera-pipeline.md), [camera-pipeline-inventory.md](camera-pipeline-inventory.md) |
| APK, gain, EEPROM, CC13 | [camera-xiaomi-research.md](camera-xiaomi-research.md) |
| HDR helper | [camera-xiaomi-compat.md](camera-xiaomi-compat.md), module `xiaomi-compat/README.md` |
| Megapixels bring-up | [camera.md](camera.md) |
