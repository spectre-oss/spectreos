# Original Xiaomi camera research

## Scope and current result

The later native-HAL analysis identified this phone as **Sunny**, extracted
its stock CC13 matrix, and verified its parser with REA/Ghidra. See
[Sunny module and native colour processing](#sunny-module-and-native-colour-processing).
The APK-only findings below are an earlier stage of the investigation.

The [full pipeline investigation](camera-pipeline.md) additionally recovers
45 generic use cases / 115 pipelines and all 36 compiled Xiaomi ZSL pipelines,
including the actual ArcSoft HDR/night plugin bindings and feature selection.

On 2026-10-20, the original Xiaomi Camera APK was extracted from the owner's
archived MIUI ROM and inspected with REA on the mini PC. This is static analysis
of the archived Android implementation, not evidence of what the current NixOS
phone is running. No phone partitions or camera configuration were changed.

The most actionable finding is in the native driver: **digital-gain unity is
256, while the pinned Linux driver's original default is 1024 and is written
directly to the sensor**. That encoding makes the original default 4× digital
gain. The [subsequent controlled comparison and 0.3.5 deployment](camera-colour.md#unity-gain-correction-035)
verified a 3.99× signal ratio and corrected processed capture to unity. The
remaining colour errors show that gain was not the entire calibration problem.

The inspected APK paths request white balance, exposure, saturation and
sharpness through Android Camera2 and Qualcomm vendor tags. They do not supply
a directly reusable IMX363 colour matrix. The matching vendor Chromatix and
sensor-module binaries are preserved for further analysis. This does not prove
that every APK method or native library has been exhaustively inspected.

See [current colour diagnosis](camera-colour.md) for the observed sensor-mode
fault, the deployed preview workaround and the remaining colour-quality issues.

## Provenance and files

All paths in this section are on the build host (`<build-host>`).
The research root is `<camera-research-dir>`.

- Original ROM:
  `<path-to-miui-rom.zip>`
- ROM SHA-256:
  `aeb05ec30c8feccc65d2152cfa8bdbbdc9800874aff4c570057d9326e9038630`
- Original filesystem APK path: `/system/priv-app/MiuiCamera/MiuiCamera.apk`.
- Extracted APK: `original/MiuiCamera.apk`, 62,079,682 bytes.
- APK SHA-256:
  `126b9b96e1541d20171ccec57f463f1f75d195d72a9bbd76270f80ffe495539b`
- Package: `com.android.camera`, version name `4.2`, version code `5`.
- REA inventory: 6,013 top-level class rows, complete coverage, no duplicate
  display names; 2,547 resources reported during load. Inner classes are reported
  separately by class inspection.
- ZIP CRC verification passed. APK signing-certificate verification was not
  performed; provenance is the owner's archived ROM plus the recorded hashes.

`system.new.dat.br` was decompressed and its `new` extents reconstructed using
`system.transfer.list`. `debugfs` read the resulting image and dumped the APK;
no filesystem mount or flash was needed.

| Relative path | Contents |
| --- | --- |
| `original/system.transfer.list` | Original system extent map |
| `original/system.dat` | Decompressed system payload |
| `original/system.img` | Reconstructed filesystem for read-only inspection |
| `original/MiuiCamera.apk` | Original APK, not modified or executed |
| `original/vendor/` | QTech/Sunny IMX363 tuning and module binaries, plus three native sensor libraries |
| `tools/rea/` | Official REA source checkout, revision `4bfdadb7de25981422572fb7ebdc3d79520ef832` |
| `tools/rea-camera` | CLI wrapper selecting Java, JADX and bounded worker resources |
| `tools/ghidra_12.1.4_PUBLIC/` | Official native-code analysis engine |
| `evidence/package.json` | REA package/manifest inspection with artifact identity |
| `evidence/classes.json` | Complete REA class-name inventory |
| `evidence/com.*.json` | Selected class fields, methods and inner-class inventories |
| `evidence/method-*.json` | Selected Camera2 method decompilations and any explicit errors |
| `evidence/qcom-*.json` | Qualcomm-specific control-method decompilations |
| `evidence/dex-colour-strings.json` | Relevant original DEX strings with DEX file and string-table index |
| `evidence/camxoverridesettings.txt` | Original Qualcomm CamX/CHI stack settings file |
| `evidence/vendor-camera-files.txt` | Original `/lib/camera` directory inventory |
| `evidence/apk.sha256`, `rom.sha256`, `vendor.sha256` | Original-file hashes |
| `evidence/native-*.json` | REA/Ghidra sensor API, gain functions, register writes and code-section hashes |
| `evidence/linux-imx363-from-archive.c`, `linux-driver-provenance.json` | Pinned Linux driver and its archive member/hash provenance |

Large images, binaries, tool installations and decompilation outputs stay on
the mini PC, outside the two-way source sync. This Markdown report is synced.

## Findings from the APK

Each REA evidence JSON retains the target SHA-256, provider, operation and raw
observations. Decompiled Java is an interpretation of bytecode, not recovered
original source. Reported method source coverage was complete for the methods
described below.

| Inspected method | Observed behavior and implication |
| --- | --- |
| `CaptureRequestBuilder.applyAWBMode` | Writes `CONTROL_AWB_MODE`. Automatic WB is requested from the Android camera stack; this method does not calculate RGB gains. |
| `CaptureRequestBuilder.applyCustomAWB` | Delegates to `MiCameraCompat`; the Qualcomm `applyCustomWB` writes `USE_CUSTOM_WB`. No sensor calibration matrix appears in these bodies. |
| `MiCameraCompatQcomImpl.applySaturation` | Maps UI levels 0–6 to vendor values 0, 2, 4, 5, 6, 8, 10. These are vendor controls, not libcamera saturation multipliers. |
| `MiCameraCompatQcomImpl.applySharpness` | Writes a vendor sharpness level 0–6. It does not implement a sharpening filter. |
| `CaptureRequestBuilder.applyNoiseReduction` | Sets `NOISE_REDUCTION_MODE` to 2. Inspection of this helper alone does not prove which capture paths call it. |
| `CaptureRequestBuilder.applyExposureCompensation` | Writes Camera2 EV compensation and includes a conditional long-exposure preview adjustment. Copying that adjustment into normal preview would be unjustified. |
| `MiCameraCompatQcomImpl.applyISO` | Uses vendor priority and ISO selection tags, mapping standard ISO values to enum values; other values use a separate absolute-ISO tag. These are not sensor analogue-gain register codes. |
| `MiCameraCompatQcomImpl.applyExposureTime` | Uses vendor exposure-priority tags when appropriate, clears standard sensitivity in that path, then delegates to the base implementation. |
| `MiCamera2.applySettingsForPreview` | Applies common settings, AE/AWB locks, FPS range, session parameters, colour-enhancement and scene controls. This method has no embedded sensor matrix. |
| `Camera2CompatAdapterCommon.init` | Enumerates cameras and reads Android camera characteristics, including facing direction. It does not select QTech versus Sunny by inspecting sensor EEPROM in this method. |

The APK includes `com.mi.device.Beryllium`; most of its feature-method names
are obfuscated. Finding this class establishes device-specific application
configuration, not the module actually fitted in this phone.

The DEX string-table scan found AWB vendor identifiers including
`org.quic.camera2.statsconfigs.AWBFrameControl`,
`com.qti.stats.awbwrapper.AWBCCT`, and `xiaomi.camera.awb.colorTemperature`.
`AWBFrameControl` declares R/G/B gain floats and a colour-temperature integer.
These are useful result-metadata leads; their presence is not a calibration
profile. The scan found no `imx363`, `qtech` or `chromatix` strings. Matches for
`sunny` concerned a named image effect, not verified module identification.

REA resolved the AWB decoder using dot-separated inner-class names. Its
`unmarshal` method reads three consecutive floats followed by one integer and
passes them to the `AWBFrameControl` constructor. Java output was unavailable;
REA returned complete smali with an explicit fallback marker. The initial
`$`-separated class lookup failed and its error remains in the evidence;
`method-awb-unmarshal-resolved.json` is the successful result. This does not
establish the caller's byte order or a way to access vendor AWB results on NixOS.

The APK assets include scene/beauty/normal filter images. Those artistic filters
must not be treated as a linear sensor-to-sRGB calibration.

## Vendor configuration and calibration leads

The archived `/etc/camera/camxoverridesettings.txt` contains:

- `tintlessEnable=TRUE`;
- `useHardcodedGamma=FALSE` and `ignoreChromatixRGammaFlag=FALSE`;
- `disablePDAF=FALSE`;
- `disableAECconvergeDependency=FALSE`;
- `debugDataSizeAWB=650000`.

These are original configuration values, not proof that every feature was
active in every original capture. They support investigating the vendor ISP's
tint, gamma, exposure-convergence and AWB implementations rather than assuming
that the APK supplies a universal colour lookup table.

Both `com.qti.sensormodule.qtech_imx363.bin` and
`com.qti.sensormodule.sunny_imx363.bin` were recovered alongside their matching
`com.qti.tuned.*` files. See `evidence/vendor.sha256` for all hashes and
[the existing Chromatix analysis](camera-colour.md#original-vendor-tuning-recovered)
for the exploratory CC13 decoder. At that earlier stage, module selection,
tuning-branch selection and matrix conventions were unverified and neither
candidate was installed. The Sunny identification and later 0.3.8 deployment
are documented below.

## Native gain investigation: concrete Linux mismatch

REA/Ghidra recovered `GetSensorLibraryAPIs`, `CalculateExposure`,
`FillExposureSettings`, `RealToRegisterGain`, `RegisterToRealGain`,
`CalculateDigitalGain` and `CalculateDigitalGainNew` from the original generic
IMX363 sensor library. The module-specific QTech and Sunny APIs and digital-gain
helpers were also inspected. These are static observations of ARM code;
decompiler parameter types and calling conventions are not fully recovered.

- Analogue gain is clamped to 1–16×, then encoded as
  `512 - 512 / gain`. Decoding uses `512 / (512 - code)` with code capped at
  480. This agrees with `patches/libcamera-imx363-helper.patch`.
- The generic, QTech and Sunny `CalculateDigitalGain` helpers multiply their
  selected linear gain by **256**. The inspected helper selects unity below
  its analogue-gain threshold and caps its computed digital multiplier at 2×.
  The generic `CalculateDigitalGainNew`, used by `FillExposureSettings`, uses
  the same encoding but caps at 8×. These caps describe different helpers,
  not a proven hardware limit or a single universal stock gain policy.
- `CalculateExposure` divides the encoded digital-gain value by **256** when
  reporting real gain. `FillExposureSettings` likewise uses `/ 256.0` and
  emits the computed gain as high/low register bytes through supplied register
  addresses. Its decompilation alone does not identify those external addresses.
- The QTech and Sunny `.text` sections are byte-identical (8,444 bytes,
  SHA-256 `0c849bb58e2396f4dfc5a8849b969dae0ba2d407998ab9866e8f45af54f8a198`).
  Their `.rodata` sections also match. The generic `.text` hash differs;
  the corresponding generic gain helpers were inspected separately.

The pinned Linux `drivers/media/i2c/imx363.c` declares
`IMX363_DGTL_GAIN_DEFAULT` as **1024**, and `imx363_update_digital_gain` writes
that control value without rescaling to all four channel registers:
`0x020e`, `0x0210`, `0x0212`, `0x0214`. The same driver contains a full-mode
initialization write of `0x0100` at `0x020e/0x020f`, consistent with 256.
The comparison copy was verified byte-for-byte against the member in
`kernel/linux.tar.gz`, under
`linux-85f1df2a4ec71d7a91dd95a7a49f889d1595ffa8/`; its SHA-256 is
`13d15b54c0a45598d0d165b338c42ee9bbd627cb6a072bdad272e32a20c0583b`.

The former libcamera zero-gain recovery patch trusted the driver's declared
default, so it could restore 1024 after a zero-gain state. This was a known
problem with that assumption: **nonzero does not mean unity**. The register
encoding implies 1024 is 4× digital gain, or two stops above unity. This can
alter highlight headroom and the interpretation of exposure/black-level tests.
It does not by itself establish the live register state, brightness after AE
convergence, or the cause of a colour cast.

The follow-up test compared 256 and 1024 at identical exposure,
analogue gain, sensor mode and lighting, using non-clipped RAW regions and
verified the gain ratio. Subsequent covered-lens measurements confirmed a
neutral RAW10 pedestal of 64; see [camera colour](camera-colour.md). Full colour
calibration remains outstanding. Camera 0.3.5 enforces unity for processed captures, and a
kernel default patch is included for future system builds. The current kernel
was not replaced. Keep the existing bad-binning-mode workaround until its
independent register-layout issue is resolved.

## Reproduce on the mini PC

REA CLI version: `6.7.0`. Node: `26.12.0`. Full JDK: `21.0.12.1`.
JADX engine: `jadx-headless-mcp 0.7.1`, verified SHA-256
`6e5eacf500b64292bfb73c49797c1958f6ee44646e43e868039ae7feb573ff75`.
The wrapper uses a 4 GiB heap, four available processors and a 600 s operation
deadline. The engine reported one JADX worker for these inspections.

```bash
cd <camera-research-dir>
tools/rea-camera doctor --provider jadx --format json
tools/rea-camera inspect-android-package original/MiuiCamera.apk --format json
tools/rea-camera inspect-android-method original/MiuiCamera.apk \
  com.android.camera2.compat.MiCameraCompatQcomImpl applyCustomWB --format json
```

Optional-engine and missing agent-registration warnings in the full doctor
report do not invalidate a healthy explicit JADX check. CLI analysis does not
require editing Codex registrations. The npm warning about the blocked Frida
install script did not prevent this static JADX analysis.

Ghidra 12.1.4 was downloaded from the official NSA/Ghidra release. Archive hash:
`ddac49f903da9d5bac833e5cc79395098b9c33cfd3279be5f31bd00387d2d4db`,
verified against release API metadata retained in `evidence/ghidra-release.json`.
Use `GHIDRA_INSTALL_DIR=$PWD/tools/ghidra_12.1.4_PUBLIC` for native REA commands.

Tool references: [REA Android analysis](https://github.com/morluto/rea/blob/main/docs/android-analysis.md),
[REA Ghidra provider](https://github.com/morluto/rea/blob/main/docs/ghidra-provider.md),
[official Ghidra release](https://github.com/NationalSecurityAgency/ghidra/releases/tag/Ghidra_12.1.4_build).

## Sunny module and native colour processing

Observed on 2026-10-20 after the APK investigation. The actual camera EEPROM
reports vendor ID **1**, matching Sunny's original module descriptor. The QTech
descriptor requires **6**. Two separate 256-byte reads were identical. The
kernel's BU64748 actuator node is therefore not reliable evidence that a QTech
module is fitted. This also leaves a possible actuator-driver mismatch to
investigate separately; no focus-driver changes were made in this work.

`modules/camera/read-eeprom.c` reads only the first 256 bytes at Linux address
`0x54` (stock 8-bit address `0xa8`) with a two-byte address pointer and eight-byte
reads. The pinned Qualcomm CCI adapter limits reads to twelve bytes. It writes
no EEPROM data. It was built only on the build host using `read-eeprom.nix`, copied
to `/tmp` on the phone and run against `/dev/i2c-16` while the camera was powered.
The prefix includes a module identifier, so the raw data stays in excluded
local evidence, not the docs corpus. Prefix SHA-256:
`40da223c4ad7839bf03811e46df520232eda8cf2eb9ef62ef4e1d6ecab7d8cef`.

### Original native libraries

Read-only `debugfs` extraction from the reconstructed original vendor image:

| File under research `original/vendor/` | SHA-256 | Purpose |
| --- | --- | --- |
| `camera.qcom.so` | `0c0d7db395703421728f01fde68e28130a36faf84b18b16571a540ce3474f034` | ARM32 Qualcomm camera HAL; contains Chromatix parsers and CC13 interpolation/settings functions |
| `com.qti.chi.override.so` | `25643fc83f3a300e5901edb83cd20644bc360b23a87d5a647f7b9b6c6e5d2ad6` | Original CHI overrides, extracted but not exhaustively analysed |
| `com.qti.tuned.sunny_imx363.bin` | `554d5195774fec8b07166cf83ae37e9a0bca952b01a993f98e73c68e5e174480` | Matching Sunny Chromatix tuning container |

REA 6.7.0 with Ghidra 12.1.4 analysed `camera.qcom.so`. A persistent REA MCP
session avoided repeatedly importing the HAL. Ghidra pseudocode is a decompiler
interpretation, not recovered original source; inferred parameter types and
calling conventions need care, especially floating-point return registers.

| Function address | Evidence saved in research `evidence/` | Verified observation |
| --- | --- | --- |
| `0xe1870` | `hal-cc13-matrix-parser.json` | Matrix reader consumes nine consecutive floats |
| `0xe1964` | `hal-cc13-cct-parser.json` | CCT trigger precedes the region's matrix and offsets |
| `0xe21ac` | `hal-cc13-root-parser.json` | Root parser loads enable/version/control/private/reserve/core blocks in sequence |
| `0xe2440` | `hal-cc13-control-parser.json` | Two control-method enums are read |
| `0x196abc` | `hal-cc13-effect-code.json` | Matrix multiplication uses row-major indices `row*3+column` |
| `0x24da90` | `hal-cc13-interpolate-code.json` | Six-level tree setup includes DRC, HDR, LED, AEC and CCT inputs |
| `0x24e2b0` | `hal-cc13-aec-code.json` | AEC node selects and interpolates trigger regions |
| `0x2456ec` | `hal-aec-trigger-code.json` | AEC enum 0/1 chooses one of two inputs; this alone does not identify libcamera-equivalent lux/gain semantics |
| `0x1e413c` | `hal-cc13-setting-code.json` | Hardware coefficient packing permutes elements; its register order must not be copied as libcamera RGB matrix order |

REA MCP reproduction: `open_binary` with the absolute HAL path and
`provider_id: ghidra`, then `procedure_pseudo_code` with the addresses above.
`search_procedures` with literal `cc13` lists the parser/interpolation methods.
The saved `camera-hal-ghidra-snapshot.gzf` is REA's session/evidence snapshot;
it is not a claim that a complete reusable Ghidra project was exported.

### Reproducible Chromatix decoding

`infra/decode-camera-profile.py` decodes the original version 1.3 container's
section table, 64-byte symbol records, payload references and 16-byte mode
records. It traces CC13 root pointers through DRC (16-byte), HDR (24-byte), LED
(12-byte), AEC (24-byte) and CCT (56-byte) records. It validates bounds, child
counts/types, finite values, CCT ordering and matrix row sums. It also decodes
module `customInfo` names/values, including vendor IDs and offsets. This is a
format-specific research decoder, not a general parser for every Chromatix file.

On the build host, from the project directory:

```bash
python3 infra/decode-camera-profile.py \
  <camera-research-dir>/original/vendor/com.qti.tuned.sunny_imx363.bin
python3 infra/test-camera-profile.py \
  <camera-research-dir>/original/vendor
```

Full decoded trees stay in research `evidence/sunny-cc13-tree.json` and
`qtech-cc13-tree.json`; module records are in `sunny-module-descriptors.json`
and `qtech-module-descriptors.json`. Ten Sunny and twelve QTech CC13 roots were
found. The source-controlled `modules/camera/imx363-sunny-profile.json` records
the selected matrix's provenance:

- Sunny root symbol **43**, `cc13_ipe`, mode **3** under root mode **0**.
- Child symbols **568 → 569 → 570 → 571 → 572**: first DRC/HDR/no-flash/AEC branch.
- First matrix float at byte offset **471676** of the original Sunny binary.
- Both CCT records in that branch contain the same matrix; all three offsets
  are zero. Rows sum to one within floating-point precision.

```text
 1.68869996  -0.77240002   0.08370000
-0.19410001   1.39740002  -0.20330000
 0.04450000  -0.75260001   1.70809996
```

The profile uses this one constant base matrix. The single `ct: 5000` knot is a
libcamera lookup anchor, not a measurement that the scene is 5000 K. Qualcomm's
full runtime mode, exposure, HDR and LED selection has **not** been ported.
EEPROM AWB ratios, lens shading, denoising, autofocus and tone mapping are also
not claimed to match Xiaomi. Do not apply this Sunny profile to a QTech module
just because both use IMX363. See [phone validation](camera-colour.md#sunny-stock-colour-matrix-038).
