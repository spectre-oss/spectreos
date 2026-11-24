# Camera pipeline implementation — 2026-10-21

This is the current implementation checkpoint for the owner's **Sunny IMX363**
module. The goal is still open. Recovered tables and successful compilation
must not be described as validated factory-quality processing.

## Verified on the phone: defect correction (introduced in 0.4.1)

Current installed application: **0.4.4**,
`/nix/store/jqyydjqykllfcxlhjcpp8xwbkqjhp0n3-spectre-camera-aarch64-unknown-linux-gnu-0.4.4`.
The defect-only measurements below were collected with 0.4.1.

The four retained covered-lens RAW frames at gains 4 and 16 identify **149
repeatable isolated bright defects**. `modules/camera/calibrate-defects.py`
requires agreement across all four frames, using eight same-CFA neighbours
and a noise-dependent threshold. The generated `imx363-sunny-defects.json`
records source hashes and coordinates; `defect-profile.h` is the runtime map.
This is calibration for this physical module, not all IMX363 units.

`raw10-defects.h` corrects only those coordinates from immutable input before
preview reduction and still filtering. The replacement is the median of eight
same-colour neighbours. Padding is preserved; format, dimensions and buffer
bounds are checked. It does not classify ordinary scene highlights as defects.
Only full 4032×3024 SRGGB10 CSI2 input and the explicit
`SPECTRE_IMX363_SUNNY_DEFECTS=1` setting enable it. The owner-specific launcher
sets that default. Cold/dead defects and temperature-dependent defects are not
yet calibrated.

Native ASan/UBSan RAW tests and Qt regression tests passed on the build host, as did
the ARM build. On the phone, first preview availability/presentation were
300/318 ms; visible preview was approximately 20.1 fps. A real photograph,
`IMG_20261021_071107_885_f0bc485b.jpg`, saved at 3024×4024 and 90° rotation.
Metadata recorded gain 3.737226, exposure 54,262 µs, WB 2.367807/1.396202, three
merged frames and 6,048/6,048 accepted alignment tiles. Preview restarted at
3496 ms and first resumed frame arrived at 3614 ms; denoise took 4483 ms and
encoding/save 282 ms. The keyboard remained hidden. These are observed timings,
not a cold-start guarantee. Changed framing/exposure prevents a controlled
percentage comparison with earlier photographs.

Local evidence: `.code-sync/pipeline/defects-live.jpg`, its JSON sidecar,
`defects-preview.png`, `defects-capture-screen.png`, and build logs. The previous
0.4.0 noise stages and their limitations remain in [camera-noise.md](camera-noise.md).

## Per-module EEPROM and autofocus

The default EEPROM reader still reads 256 bytes. Its explicit `--full` option
reads the stock descriptor's 8192-byte extent at address 0x54 in eight-byte
transactions. It writes address pointers only, never EEPROM data. Two full
reads were identical, SHA-256
`80ba43461c6259aa823c71461bc25be7998c49ad3e557ec1a1ca8dd72172c696`.
No EEPROM checksum scheme has been recovered; repeated reads alone do not
prove checksum validity. Raw unit data stays in excluded research storage.

`infra/decode-camera-eeprom.py` validates the pinned Sunny descriptor and its
packed field layout, verified through REA/Ghidra on the original `camera.qcom.so`.
Decoder tests pass against the actual files and malformed input cases.

| Field | Observed value / meaning |
| --- | --- |
| Module vendor | 1, Sunny |
| Actuator | `beryllium_imx363_ak7374_sunny`, I²C 0x0c |
| Position encoding | Register 0, 10-bit code shifted left six bits |
| Active command | Register 2, value 0 |
| Focus endpoints | Infinity 370, macro 859 |
| AF margins | Stock 0.3 / -0.15; not applied as new physical limits |
| WB ratios | R/G 548/1024, B/G 682/1024, Gr/Gb 1026/1024 |
| LSC | 17×13 measured-response samples for each of R, Gr, Gb, B |
| Generic stock EEPROM LSC enable | False |

WB ratios are module calibration, not scene gains. Mapping their illuminant
enum through the stock AWB runtime is still unresolved.

The running phone binds the actuator address to **BU64748**, and exposes no lens
V4L2 subdevice. This conflicts with the verified Sunny descriptor.
`kernel/patches/0003-beryllium-sunny-ak7374-focus.patch` adds the AK7374 variant
to the AK7375-family driver, changes the owner's device tree, corrects the
regulator name and supplies `lens-focus`. The config enables `VIDEO_AK7375`.
The unverified standby command is deliberately not used. This change is
Sunny-specific; it must not be applied blindly to a QTech module.

The kernel built successfully on the build host as
`/nix/store/67mzws0bhsycgayij5p7pxmrx3bq8h3s-linux-aarch64-unknown-linux-gnu-7.1.0-rc1`.
`infra/prepare-camera-boot.py` packages a RAM-only test image preserving the
phone's current initrd, boot arguments and header addresses; config comparison
allows exactly the AK7375 enable change. It retains the exact previous boot
payload as a fallback. Local artifacts are under `artifacts/sunny-focus-boot/`.

**No kernel was installed, transferred with fastboot, or flashed; autofocus is
not operational.** `reboot bootloader` returned to ordinary Linux instead of a
usable fastboot session. USB ID 18d1:d001 is misleading: live USB descriptors
identify Mobile NixOS CDC networking, not fastboot. The phone still reports
`CONFIG_VIDEO_AK7375` disabled. See [reboot diagnosis](reboot-2026-10-21.md).
The runtime driver, media link, calibrated sweep and focus algorithm remain
untested. Do not describe compilation as successful autofocus.

## Lens shading: colour-only correction deployed in 0.4.3

`infra/decode-camera-lsc.py` preserves all four BPS/IFE LSC34 roots and their
lens-position, DRC, HDR, LED, exposure and CCT branches. It validates the original
Sunny tuning hash
`554d5195774fec8b07166cf83ae37e9a0bca952b01a993f98e73c68e5e174480`.
The default candidate follows root 9, symbols 432→433→434→435→436→437 and golden
symbol 441. REA addresses include Ghidra's 0x10000 image base:

- `0xf6df8`: golden table channel order R, Gr, Gb, B.
- `0x25a1f4`: calibration divides golden measured response by module response.
- `0x259d70`: multiply interpolated base gains by that ratio, then average greens.

The candidate computes `base gain × golden response / module response`.
Resulting channel ranges are R 1.008–3.823, Gr/Gb 1.005–3.384, B 1.005–3.247.
These are candidate correction gains; raw EEPROM samples themselves are not
gains. `infra/test-camera-lsc.py` passes independent arithmetic, green averaging,
invalid-value rejection and pinned source-tree checks on the build host.

The source candidate's `enabled: false` records the recovered generic stock
LSC path, not the app's runtime setting. The owner-specific launcher now opts
into `SPECTRE_IMX363_SHADING=colour`. This deliberate software-ISP correction
uses channel gain divided by mean green gain, normalized at the mesh centre;
it corrects colour variation without enabling full brightness compensation.
`full` is experimental; `off` disables correction.

The first purported wall capture still contained the desk and was rejected.
After the owner moved the phone, four retained frames show only a textured
light wall, with a visible illumination gradient. Sensor controls were fixed:
exposure 1383 lines, analogue gain code 256 (2×), digital gain 256 (unity).
Frame means were 235.438, 235.424, 235.334 and 235.332 RAW10 counts, with no
clipped pixels. `modules/camera/validate-shading.py` independently samples
13×17 regions, subtracts measured black 64 and compares centre-normalized
channel ratios using the full-resolution CFA coordinates.

| RMS spatial colour-ratio deviation | Before | Colour-only candidate |
| --- | --- | --- |
| R/G | 11.14% | 1.91% |
| B/G | 2.94% | 1.74% |

This is evidence for this module and scene, not calibrated absolute colour,
certified flat illumination or validation of every stock lighting trigger.
Raw pair hashes and reproducible metrics are in local
`.code-sync/pipeline/flat-validation.json`. Full brightness compensation, CA
model and dynamic trigger selection remain unvalidated.

`raw10-shading.h` precomputes a Q12 mesh and applies it after defect correction,
preview reduction/still denoise and before debayer/statistics. The reduced
preview uses the centre of each same-CFA averaging footprint. Signed samples
around pedestal 64, clipping, padding and immutable input are preserved. Native
ASan/UBSan and Qt regressions passed; ARM build passed. Phone logs confirm
`Sunny module lens shading: colour` in both preview and full-resolution modes.

A real 3024×4024 JPEG (`IMG_20261021_074627_539_8766c00a.jpg`) saved successfully
with three merged frames, and the keyboard stayed hidden. Preview restart was
5195 ms, first resumed frame 5356 ms; background denoise 4331 ms and save 250 ms.
First preview availability/presentation were 360/632 ms. Steady visible preview
was about 17.8 fps, a cost compared with the earlier ~20 fps observation.
An ARM NEON implementation passes byte-for-byte comparison with the scalar
path on the actual phone, including packed tails, padding, clipping and
below-black values. A 30-frame 1008×756 processing benchmark measured scalar
6.14 ms/frame versus NEON 1.46 ms/frame. Native sanitizer/Qt tests also passed.
The optimized app is deployed as **0.4.4**. Live preview measured about 19.5 fps
versus 17.8 fps for 0.4.3; both sensor snapshots showed exposure 3126 lines,
analogue code 259, digital gain 256 and vblank 112. First preview was available
at 336 ms and presented at 383 ms. This is one warm launch, not a startup SLA.

A second real photograph, `IMG_20261021_075226_647_286d3497.jpg`, validated the
accelerated full-resolution path. Preview restarted at 4332 ms and resumed at
4449 ms; three-frame denoise took 4355 ms and encoding/save 243 ms. The keyboard
remained hidden and visible preview returned to about 19.5 fps. The remaining
several-second shutter interruption is not fixed by this shading optimization.
Evidence: `.code-sync/pipeline/shading-vector-live.log`,
`shading-vector-phone-test.log`, `shading-vector-live.jpg.json`, screenshots and
server build logs. The scalar path remains available in the unit-test API for
byte-exact comparisons; ordinary ARM camera processing uses NEON automatically.

## Manual exposure/WB prerequisite for HDR — verified in 0.4.2

The original Simple IPA always adjusted exposure automatically. The patch,
`libcamera-simple-manual-capture.patch`, adds request state for independent
exposure/gain modes, legacy AeEnable precedence, bounded manual values, WB lock
and manual colour gains. Automatic requests ignore manual values; transition
to manual without values freezes the last computed values. Frame contexts keep
their own snapshots. Metadata continues to report observed sensor exposure and
gain, rather than substituting requested values.

`manual-capture.h` shares state/conversion helpers with sanitizer tests.
`spectre-capture-probe --output NEW_DIRECTORY [--full]` is an explicit hardware
diagnostic: auto warmup, four manual targets, four consecutive matching metadata
frames per target, bounded wait, saved PPM evidence and automatic-mode recovery.
The pipeline's sensor delay still requires measurement; callers must wait for
actual metadata, not assume the first request has applied its exposure.

Native sanitizer tests, Qt regressions and the ARM build passed. Live tests
passed at both 1000×750 preview and 4024×3024 full resolution. Requested/observed
exposures were 12000/11994, 24000/24006, 6000/6006 and 12000/11994 µs, with
requested gains 2, 2, 2 and 4 observed exactly. Each target had at least four
consecutive matching metadata frames; WB stayed fixed and automatic exposure
resumed. Local evidence: `manual-{preview,full}-report.json` and probe logs in
`.code-sync/pipeline/`; PPM captures remain on the phone. This verifies manual
control and observed convergence, not a calibrated request-delay model.
The active Qt shutter does not yet make HDR brackets or call the original engine. Control semantics
were checked against the [official libcamera controls documentation](https://docs.libcamera.org/master/internal-api/namespacelibcamera_1_1controls.html).

## Delegated review

With the owner's explicit authorization, two prepared source packets were sent
through the installed Grok CLI: the HDR helper boundary and burst/noise code.
No photographs, raw EEPROM or proprietary binaries were sent. Responses are
retained under `.code-sync/grok/`. The configured preferred model was unavailable
and the CLI fell back; an exact model identity is not established here.

Useful review items include exact output-size validation, subprocess cleanup,
real AE metadata, explicit YUV range, and retaining a reference photo on failure.
The reported overlapping shutter/save bug does not reproduce through the UI:
`CameraController::capture()` refuses a second capture while `pending` is true.
Review findings are hypotheses to check, not accepted defects or validation.

## Remaining implementation and evidence gates

| Area | Current state / next evidence |
| --- | --- |
| Black level / base colour | Measured black, unity digital gain, one Sunny matrix; scene-dependent matrices and calibrated AWB remain. |
| Defects | 149 bright points deployed; other defect classes/temperature coverage remain. |
| Noise / multi-frame | Existing measured RAW/chroma filters and three-frame merge; stock MFNR parity not established. |
| Lens corrections | Colour-only stock/module mesh validated on one wall and deployed; full brightness, triggers and CA remain. |
| Autofocus | Correct actuator identified and kernel built; temporary boot, runtime test and focus algorithm remain. |
| Tone/detail | Highlight guard exists; stock local tone mapping, sharpening and texture preservation remain. |
| HDR | Original ARM32 worker works on fixtures; manual capture verified; real AE mapping, frame order, YUV range and UI integration remain. |
| Night | Original libraries recovered; ABI/lifecycle and real multi-frame integration remain. |
| Latency / acceleration | Fast small preview, asynchronous save and ARM NEON shading; full-resolution ZSL, GPU/DSP processing and zero-copy processing remain. |

Full Xiaomi CamX/CHI hardware processing is not available merely because its
userspace files exist. No claim of complete stock pipeline equivalence is made.
