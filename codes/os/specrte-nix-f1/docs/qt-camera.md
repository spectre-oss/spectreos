# Spectre Camera (Qt / libcamera)

**Current deployed version: 0.4.4.** Adds ARM-accelerated module-specific colour-shading correction
validated against the owner's wall captures. Includes 0.4.2 manual exposure/WB
support and 0.4.1 correction of 149 measured bright defects. See
[current implementation, live measurements and limitations](camera-pipeline-progress.md).
Version 0.4.0 introduced measured
gain-dependent RAW filtering, motion-gated burst merging and separate preview
and still chroma processing. See [implementation, calibration limits and phone
measurements](camera-noise.md). The 0.3.9 shutter/keyboard fix remains included.

The earlier **0.3.8 colour baseline**: this phone's EEPROM identifies a Sunny
IMX363 module. Its original Xiaomi base colour matrix is now enabled, with
bounded white-balanced inputs to avoid pink saturated highlights. Measured
black subtraction, unity digital gain and the exposure highlight guard remain.
Warm first presentation was 305 ms; steady visible preview was about 21.5 fps
with the extra matrix processing (previously about 24 fps without it).
See [colour diagnosis and live validation](camera-colour.md). Full sensor
colour calibration remains incomplete. Earlier zero-gain restoration-to-1024
and colour-rendering measurements below are historical.

Colour investigation and the current preview-mode fix:
[IMX363 colour diagnosis and calibration](camera-colour.md). The 0.3.4 change
avoids the broken binned sensor mode and adds Bayer-preserving preview
reduction; earlier 1024×768 measurements below describe the previous path.

Created **2026-10-20**, migrated to **libcamera in version 0.2.0** at the owner's
request to prioritize picture quality. Sources: `apps/spectre-camera/`.
The previous 0.1.0 libmegapixels/custom-demosaic implementation is superseded.

## Rotation/cache correction — current status

**0.3.2 fixes a stale QML disk-cache problem.** Earlier rotation confirmations
proved settings changes and saved-photo dimensions, but did not prove that the
phone's visible preview had changed. A compositor screenshot showed the old
landscape UI without the new rotate button. Those preview claims are superseded
by the evidence in the 0.3.2 section below.

## Processing and quality

Version **0.3.2** requests a **1024x768 processed preview** from libcamera's
simple pipeline and software ISP. The negotiated size is printed in the app log;
the phone validated the requested 1024x768 mode. The app switches to the
native-size still configuration only when the shutter is pressed, waits six
frames for initial AE/WB statistics, copies the full-resolution photo, and
restarts preview before encoding the file on a separate background task. Six
frames are not a claim of exposure convergence. Stream switching temporarily
holds the last preview image; this is not a zero-shutter-lag implementation.

The previously tested full-resolution stream negotiates **4024x3024 BGR888/sRGB**,
previously saved at 3024x4024 using the reported 270-degree mounting rotation.
The owner reported that the 0.3.0 correction still displayed landscape (`—`)
when portrait (`|`) was wanted. The old zero-degree result is historical evidence,
not a verified orientation fix. Version 0.3.1 adds a further quarter turn by
default, producing 90-degree displayed/saved pixels on this phone. The ↻ header
button cycles image orientation in 90-degree steps, updates the scene-graph
preview immediately, and stores the choice in Qt settings under
`camera/rotationOffset`. It applies to subsequent saved photos as well. Rotation
is locked during an in-flight capture so preview and photo cannot disagree due
to changing the setting mid-shot. `imageRotationDegrees` records the actual
saved-pixel rotation in each sidecar. Existing photos are not rewritten. No
upscaling is used to claim a larger resolution. Version 0.2.0 continuously
processed that full-size stream, rotated/scaled it on the CPU, and limited UI
refresh to eight updates per second; that implementation is superseded.

The backend selects IMX363 by its model property and uses FrameBufferAllocator /
Request capture with mapped processed output, validated stride and RGB byte
order. Preview is copied at its small negotiated size without CPU rotation or
resizing. Qt Quick rotates and fits it in the scene graph. Hardware acceleration
depends on Qt's active renderer; the offscreen software test does not prove GPU
rendering on the phone. RAW processing remains inside libcamera.

A bounded set of requests and a latest-frame image provider avoid an unbounded
frame queue. Coalesced frame notifications replace the 125 ms polling timer and
only notify the QML preview property, rather than reevaluating all controls on
every frame. Camera startup overlaps QML loading and the first received preview
frame enables the shutter without the previous twelve-frame startup delay.
File encoding and atomic writes run separately from real-camera preview capture.
The thumbnail decodes asynchronously at a bounded size. The photo viewer loads
only when opened, also asynchronously with a display-sized decode; the saved
full-resolution file is unchanged.

Libcamera performs automatic exposure/gain, white balance, black-level handling,
gamma and demosaicing. This is a software ISP; this work does not establish use
of Qualcomm's proprietary Android ISP or reproduce Google's/MIUI's computational
photography pipeline. Full colour calibration, lens shading, HDR and autofocus
integration remain unverified or unavailable. Version 0.4.0 adds the software
noise-reduction stages described in [camera-noise.md](camera-noise.md).
Merely using libcamera does not establish acceptable picture quality.

The initial 64/1024 pedestal came from a published IMX363 helper. Later covered-
lens measurements confirmed approximately 64 RAW10 counts on this phone, and
the current sensor YAML uses that measured black level. Version 0.3.8 adds one
verified Sunny stock colour matrix; generic AWB and unverified sensor delays
remain. Metadata explicitly says this is not full factory calibration. A
neutral reference and colour chart under known lighting are needed before
claiming calibrated colour accuracy. See the [native research](camera-xiaomi-research.md#sunny-module-and-native-colour-processing)
for provenance and limitations.

## Interface and output

- Portrait viewfinder, large shutter, composition grid and three-second timer.
- Last-photo thumbnail and an in-app photo viewer.
- JPEG quality 100, or **lossless PNG** through the information panel.
- Photos saved atomically to `~/Pictures/Spectre Camera/`, with a UUID suffix
  to avoid filename collisions if the phone clock jumps.
- A `.jpg.json` / `.png.json` sidecar records actual exposure, analogue gain,
  white-balance gains, gamma, black level, negotiated dimensions and the
  explicit uncalibrated status when that metadata is available.
- Camera release on application inactivity, followed by reopen on return.
- Clearly marked synthetic demo mode for interface tests.

The old digital-brightness slider has been removed from the processing path.
Exposure compensation is enabled only if libcamera advertises `ExposureValue`.
The inspected simple IPA exposes contrast/gamma but not that control, so the
panel reports automatic exposure rather than pretending to change sensor EV.
No front-camera switch, video, flash or autofocus is presented as working.

## Local libcamera fixes

`modules/camera/libcamera.nix` builds the pinned libcamera 0.7.0 simple pipeline,
software IPA and `cam` diagnostic tool. It retains Nixpkgs' IPA signing hook.
The app wrapper points its IPA module, proxy and config paths at the matching
library output, preventing an inherited environment from loading an old IPA.

Three source patches are applied:

1. `patches/libcamera-imx363-helper.patch`: local backport of the helper part of
   [Samuel LEGROS' IMX363 submission](https://patchwork.libcamera.org/patch/28201/).
   The pinned driver writes gain codes 0..480 to register 0x0204. The helper uses
   gain = 512/(512-code), correctly exposing 1x..16x gain instead of treating
   the register codes as linear gain. The upstream submission was withdrawn
   pending upstream kernel-driver support; it is not an accepted upstream
   calibration. Unmeasured delay values from that submission are not copied.
2. `patches/libcamera-softisp-initialize-params.patch`: first-frame startup fix.
   Live `cam` tests intermittently crashed in `DebayerCpu::updateGammaTable`.
   A debugger backtrace and source inspection showed `SoftwareIsp::debayerParams_`
   could be passed to the worker before the asynchronous IPA update initialized
   it. The patch initializes the shared identity matrix and seeds the local
   parameter copy from synchronous IPA defaults before streaming begins.

3. `patches/libcamera-imx363-digital-gain.patch`: recover zero digital gain at
   configure time. Live inspection found `digital_gain=0` with a driver-declared
   default of 1024. Software IPA controls exposure and analogue gain, so it did
   not recover that state. The patch applies only to IMX363, restores only zero
   to the driver's own positive default, and preserves other positive values.
   The cause of the earlier zero setting was not established. A deliberate
   zero-gain regression test logged the restoration and finished at 1024.

A separate `spectre-sensor-controls` utility queries V4L2 sensor controls and
allows explicit bounded control changes for diagnostics. The camera app itself
uses libcamera exclusively. If enabling the sensor colour-bar test, always
record the old value and restore `test_pattern=0` after the diagnostic.

## Build and test — build server only

```bash
python3 codes/spectre-nix/infra/server.py build qt-camera-test
python3 codes/spectre-nix/infra/server.py build qt-camera
```

The first builds/runs the native x86 test app on the server. The second
cross-compiles ARM64 there with cached ARM64 Qt/system libraries and native
server moc/rcc tools. No compilation is performed on this computer.
`modules/camera.nix` includes the app for future full image builds. Kernel
rebuilding/flashing is not required for this migration.

Native regressions cover processed RGB channel order, row padding, rotation,
truncation rejection and sRGB tagging; main/expanded QML views; repeated JPEG
and PNG saves with stop/reopen. These tests do not claim real image quality.

Live evidence on 2026-10-20:

- Installed libcamera 0.7.0 detected the main rear IMX363 through `qcom-camss`.
- Unpatched library: multiple first-frame software-ISP crashes, plus one
  successful two-frame full-resolution run. A smaller preview also crashed;
  this was not proven to be a resolution-specific defect.
- Patched library: **120 consecutive 4024x3024 processed frames completed** with
  no crash marker, and the Qt hardware test saved two full-resolution JPEGs
  with a camera stop/reopen between captures.
- Metadata reports 1x..16x supported gain and real exposure/AWB fields.
- The first saved photos were **entirely black**, not acceptable camera-quality
  evidence. The owner was asked to uncover/aim the lens at a lit scene, and a
  sensor colour-bar diagnostic was attempted.
- After restoring digital gain to 1024, a lossless real frame contained nonzero
  pixels (8-bit RGB means approximately 10.8, 13.3, 10.0), but visual inspection
  showed very dark noise without an identifiable scene. That is not evidence of
  acceptable quality or a successful colour calibration.
- Setting `test_pattern=2` before and repeatedly during capture still produced
  black images, not recognizable bars. This diagnostic is unresolved and must
  not be cited as proof of sensor/ISP correctness or blamed only on lighting.
  The control was restored to 0 afterward. No kernel change was made.
- The final installed build passed two real lossless PNG captures and camera
  reopen, and explicitly recovered a deliberately zero digital gain to 1024.
  Its logs no longer contained the earlier media-device teardown warning.
- Camera configuration ownership was corrected so its retained camera reference
  is released before CameraManager teardown.

Logs, backtraces, local diagnostic images and test screenshots are under
`.code-sync/camera-tests/libcamera-*`. They are not source-synchronized.

Runtime options:

```bash
spectre-camera
spectre-camera --demo
spectre-camera --self-test
spectre-camera --demo --capture-test
spectre-camera --demo --lossless --capture-test
spectre-camera --hardware-test             # explicitly takes two actual photos
spectre-camera --lossless --hardware-test  # same, lossless PNG
```

The initial twelve processed frames are allowed to provide AE/AWB statistics
before enabling the shutter. This warm-up does not prove those algorithms have
converged; inspect scene content and metadata.

## Transfer and installation

For an incremental update to the connected phone:

```bash
python3 codes/spectre-nix/infra/export-camera.py --app qt \
  --known-hosts /path/to/verified-phone-known-hosts
python3 codes/spectre-nix/infra/install-camera.py \
  codes/spectre-nix/artifacts/qt-camera \
  --known-hosts /path/to/verified-phone-known-hosts
```

`--full` exports the complete closure without contacting the phone. Otherwise,
the NAR contains only store paths missing from that phone; it is not a universal
standalone installation image. SHA-256, size and all required store paths are
recorded in `closure.json`. SSH transport compresses large exports.

The installer verifies local/transferred SHA-256, imports the store paths,
roots the app at `/var/lib/spectre-camera/gcroots/spectre-camera`, and installs
`~/.local/share/applications/org.spectre.Camera.desktop` plus its icon. The icon
is installed with user-writable permissions so later upgrades can replace it.
Credentials are prompted or supplied through an explicitly named environment
variable. No system image or boot partition is changed; Megapixels remains
available separately.

For rollback, close the app and remove its managed user launcher and icon.
Removing its GC-root symlink permits normal later Nix garbage collection.
Saved photos are independent of the application package.

## Performance verification (0.3.0)

Launch with `--benchmark` to close after twelve seconds. Logs report first
processed preview availability, first Qt frame-swap presentation since process
entry, processed preview frames/s and distinct presented preview updates/s.
The presentation counter observes image-provider revisions at frame swaps; it is
an application metric, not a hardware display latency measurement. Shutter logs
include full-resolution capture through the preview configuration restart;
they do not include JPEG/PNG encoding or guarantee first resumed-frame timing.

Native server tests of 0.3.0 passed RGB layout, main/expanded QML rendering,
and repeated JPEG/PNG save/reopen. ARM compilation completed on the build server;
the app was installed through Nix store import and its managed user launcher.
No system image or partition was changed.

Initial live 0.3.0 measurements on 2026-10-20, before the final thumbnail and
orientation adjustments:

- 1024x768 negotiated processed preview.
- First processed preview: 169–233 ms from worker entry.
- First Qt frame-swap observation: 294–297 ms from process entry. Filesystem
  cache state was not controlled, so these are not proven cold-boot timings.
- Steady processed and presented preview: approximately **22.8 updates/s**, versus
  the previous application's explicit eight-update/s cap. An early three-second
  window showed 9.3 displayed updates/s before stabilizing; one earlier run only
  reported its first display callback. The owner then confirmed the preview was
  visible and smooth. These are not 30/60 fps measurements.
- Two real full-resolution JPEGs plus stop/reopen passed. Capture through preview
  reconfiguration took 759 and 789 ms. The last preview image remains on screen
  during that switch. JPEG encoding finishes later, independently.

Logs and deployment artifacts are local under `.code-sync/camera-tests/fast-*`
and `artifacts/qt-camera-fast/`, excluded from source sync. Picture-quality and
sensor-calibration limitations above still apply.

Final 0.3.0 package validation after the 90-degree correction:

- Installed app: `/nix/store/gw0jv4jzp5k8hdv8j0xhb8gcbrvwh3fy-spectre-camera-aarch64-unknown-linux-gnu-0.3.0`.
- Native QML/layout and repeated JPEG/PNG save/reopen tests passed again.
- Actual Wayland rendering used OpenGL, Mesa freedreno **FD630 (Adreno 630)**.
- First preview presentation: 280 ms during the hardware test and 270 ms on
  the subsequent ordinary launch; these retain the cache-state caveat above.
- Two full-resolution real JPEG captures and camera reopen passed on Wayland.
  Preview reconfiguration took 799/765 ms; the recorded first resumed preview
  was 856 ms from shutter. JPEG encoding/atomic save took 468/461 ms separately.
- The saved sidecar verifies 4024x3024 pixels and `imageRotationDegrees: 0`,
  corresponding to the requested extra 90 degrees clockwise relative to the
  previous 270-degree presentation. Metadata still reports uncalibrated colour.
- A normal camera process was left running. Displayed FPS must be measured while
  its surface is visible: covered/locked runs reported few frame swaps despite
  ongoing 22.8 fps sensor output. Steady visible 22.8 fps was established in the
  preceding benchmark, and the owner confirmed smooth preview before rotation.

## Portrait correction follow-up (0.3.1)

The owner rejected the 0.3.0 rotation result: the image was still landscape
instead of portrait. The installed launcher was verified to point at the latest
0.3.0 package, so this was not established to be an old-launcher problem.
Version 0.3.1 adds the persistent rotate control described above and defaults
to one additional clockwise quarter turn. Build and deployment results follow; these tests establish portrait pixel
dimensions, while the owner must visually confirm the scene is upright.


0.3.1 validation:

- Server ARM build and native RGB/QML/repeated JPEG/PNG tests passed.
- Installed `/nix/store/qhxs84ssng3acjwpj2p4si9nsi7bj515-spectre-camera-aarch64-unknown-linux-gnu-0.3.1`
  through a Nix store import and updated managed launcher; no flashing.
- Two real captures with stop/reopen passed using Wayland. Latest saved sidecar
  verifies **3024x4024 portrait**, `imageRotationDegrees: 90`.
- Recorded first resumed preview: 820 ms after shutter; JPEG save: 502 ms.
- Corrected app left running. Local evidence: `.code-sync/camera-tests/rotation-fix-*`;
  artifact: `artifacts/qt-camera-rotation/`. The ↻ control updates preview in the
  scene graph and persists its offset via QSettings without restarting capture.

Owner orientation preference follow-up: the owner requested the opposite
direction to the 90-degree clockwise default. On the phone,
`~/.config/Spectre/Spectre Camera.conf` now contains `[camera]` with
`rotationOffset=270` (90 degrees counterclockwise). The installed 0.3.1 app was
relaunched with this persistent preference; preview and subsequent saved photos
share that rotation. The compiled fallback remains 90 for an unset preference.

Subsequent owner request: rotate another 90 degrees. Live inspection verified
the installed and running app was 0.3.1 with `rotationOffset=270`. The camera
was stopped before changing settings, then the offset was advanced clockwise
to **0** (270 + 90 modulo 360), and 0.3.1 was relaunched. This is the latest
phone preference, superseding the 270 setting above. It is an additional
quarter turn relative to the previous view, not a new binary deployment.

Latest explicit owner request: rotate **-90 degrees** relative to the current
view. Live setting was 0; changed to **270** (0 - 90 modulo 360), saved in the
phone camera configuration, and relaunched 0.3.1. This supersedes the 0 setting
in the preceding entry.

Next explicit owner request: rotate **+90 degrees** from the current view.
Verified live offset 270, advanced to **0**, persisted, and relaunched 0.3.1.
This is the latest phone orientation preference.

## 0.3.2: actual rendered rotation and stale QML cache

Diagnosis on 2026-10-20:

- Phone binary and managed launcher really pointed at 0.3.1. The user correctly
  reported that the image did not change despite repeated settings updates.
- A screenshot from Phosh's `org.gnome.Shell.Screenshot` D-Bus service showed
  the old landscape UI with three header controls and no rotate button.
- The new native integration test could not find the preview's new object name
  with ordinary caching. The exact same build passed with
  `QML_DISABLE_DISK_CACHE=1`, including a rendered angle change from 0 to 90.
- Temporarily bypassing the cache on the phone immediately showed the rotate
  button and a visibly rotated portrait preview, verified from a second actual
  compositor screenshot. This establishes stale compiled QML as the cause.

The app previously always loaded `qrc:/Main.qml`. Nix normalizes source times,
so reusing that stable URL allowed an older compiled screen to survive upgrades.
CMake now computes SHA-256 of Main.qml and embeds/loads it at
`qrc:/spectre-camera/<content-sha256>/Main.qml`. Changed QML gets a fresh cache
address; unchanged QML retains normal disk-cache startup benefits. No global Qt
cache removal or permanently disabled caching is required. Qt documents the
cache controls at <https://doc.qt.io/qt-6/qmldiskcache.html>.

The app logs its exact QSettings filename, loaded rotation and QML URL.
`--rotation N` applies and persists a multiple-of-90 correction for diagnostics.
`--rotation-test --screenshot PREFIX` checks the actual QQuickItem rotation
against the controller, verifies a quarter turn and changed rendered pixels,
and writes PREFIX-before.png/PREFIX-after.png. It intentionally changes the
saved rotation by +90, so choose the initial diagnostic rotation explicitly.
The native test script runs this against the normal cache, after the preceding
smoke tests have already warmed it.

Server native RGB, QML rendered rotation, repeated JPEG/PNG and reopen tests
passed with normal caching enabled. ARM 0.3.2 was built on the server and
installed with its managed launcher, without flashing. Installed app path:
`/nix/store/iivca92pzb5brk41ixjw6mjia7blcaqp-spectre-camera-aarch64-unknown-linux-gnu-0.3.2`.
Local evidence is under `.code-sync/camera-tests/rotation-*`, with build and
native test logs copied there; NAR artifact is `artifacts/qt-camera-cachefix/`.

Live final 0.3.2 verification with normal caching enabled:

- `--rotation 0 --rotation-test` printed rendered/controller angles 0 then 90
  and **Rendered quarter-turn: PASS** on the actual Wayland phone window.
- Before/after screenshots were captured; the after image visibly contains the
  new rotate button and portrait scene. The selected rotation persisted at 90.
- Two real full-resolution JPEGs with camera reopen passed on the cache-safe UI.
  A recorded first resumed preview was 884 ms from shutter and save took 483 ms.
- Normal launch afterward uses the content-addressed URL with caching enabled;
  no QML_DISABLE_DISK_CACHE workaround is installed in the launcher.

## 0.3.9: shutter opening the on-screen keyboard

Reproduced on 2026-10-20 and verified fixed on 2026-10-21. The desktop exports
`QT_IM_MODULE=ibus` and `QT_IM_MODULES=wayland;ibus`. With these settings, the old
camera enabled a Wayland text-input session on focus; tapping the shutter
committed it and Stevia became visible. The camera UI has no text-entry fields.
An SSH launch without the desktop's plural variable initially hid the problem.

Before constructing QGuiApplication, `apps/spectre-camera/main.cpp` now selects
Qt's `compose` input context using both variables. This is confined to the camera
process and avoids requesting a Wayland text session. The compose context keeps
physical-key input available; Stevia and the desktop environment are unchanged.
If text-entry controls are added later, revisit this deliberate input choice.

Built and tested only on the build host. Native RGB/stride/orientation/sRGB checks,
QML rendering and controls, rendered rotation, and repeated JPEG/PNG save/reopen
tests passed. ARM package installed through the normal NAR installer:

```text
/nix/store/f6pc7lzxmzhxhrxxfz6hn2875zbf52fs-spectre-camera-aarch64-unknown-linux-gnu-0.3.9
```

Live phone verification with the desktop's input settings:

- Before: shutter tap changed Stevia visibility to true; trace contains
  `zwp_text_input_v3.enable` followed by a commit.
- After: multiple completed shutter captures saved JPEGs while Stevia visibility
  stayed false. The fixed trace has no text-input enable requests.
- Stevia PID 1212 remained running. The managed desktop launcher points at 0.3.9.
- Downloaded `IMG_20261021_055739_851_9c7cb412.jpg` decoded successfully as a
  3024×4024 RGB JPEG. Phone filenames use its UTC clock.
- Relaunched the installed app without Wayland tracing after verification.

Local screenshots, traces and a saved photo are under
`.code-sync/camera-tests/keyboard/`; install manifest and NAR are under
`artifacts/qt-camera-keyboard/`. These are local evidence, excluded from source
sync. No system image or kernel was flashed.

This release changes input handling, not image processing. Saved photos still
use one libcamera-processed full-resolution frame and JPEG quality 100 (or PNG).
The [Xiaomi HDR helper](camera-xiaomi-compat.md) is installed and tested separately,
but the app does not invoke it. Real exposure brackets, validated YUV/AE data and
asynchronous integration remain necessary before it can affect saved pictures.
The keyboard fix does not establish stock-camera image quality.
