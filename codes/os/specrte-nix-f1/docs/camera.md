# Rear camera bring-up

Inspection and live package work: **2026-10-20**. Scope: the main rear Sony
IMX363 camera. Front/secondary/IR camera support is not established here.

## Findings

The running `7.1.0-rc1` kernel exposes CAMSS media/video/subdevice nodes, probes
IMX363 and binds BU64748. The existing kernel successfully streamed five
4032x3024 RAW10 frames through libmegapixels. No kernel rebuild or flash was
needed to establish capture.

Three userspace issues were identified:

1. Installed libmegapixels 0.2.3 had no `xiaomi,beryllium.conf`:
   `megapixels-findconfig` returned `No config found`.
2. Megapixels 2.1.0 used uninitialized `v4l2_plane` arrays in its query/initial
   queue paths; shutter testing reported `VIDIOC_QBUF: Invalid argument`.
3. The mode-switch blank-frame filter returned without releasing the V4L2
   buffer. Enough blank frames could exhaust the queue before the filter's
   twenty-frame fallback, leaving still capture without an image.

The source fixes are `modules/camera.nix`, the configuration in
`modules/camera/xiaomi,beryllium.conf`, and two
`patches/megapixels-*.patch` files. The imported camera config is based on the
[historical guide's upstream configuration](https://codeberg.org/magdesign/sxmop6/src/branch/main/configs/megapixels/xiaomi%2Cberyllium.conf),
reviewed against the live media entity names and pinned library parser.

## Configuration and build

`modules/camera.nix` is imported by `configuration.nix`. It installs the
device config into libmegapixels' own data directory and applies both app
patches. Using only `/etc/megapixels` was insufficient for the inspected
package: its compiled sysconfdir lookup was relative `etc/megapixels`.

The device config selects the IMX363 -> CSIPHY0 -> CSID0 -> VFE0 RDI0 pipeline,
4032x3024 packed RGGB10, with a 270-degree rotation.

Build on the server only, from the outer local workspace:

```bash
python3 codes/spectre-nix/infra/server.py build camera
```

The `camera` target cross-compiles the patched app and library on the build
server, reusing official cached ARM64 desktop dependencies via
`modules/camera/live.nix`. It creates `result-camera` on the server. It uses
the cross-aware wrapper hook: a host-only wrapper hook produced an x86 launcher
around the ARM64 executable during development and was corrected before the
working package test.

## Transfer and installation

```bash
python3 codes/spectre-nix/infra/export-camera.py --known-hosts /path/to/phone-known-hosts
python3 codes/spectre-nix/infra/install-camera.py codes/spectre-nix/artifacts/camera-live \
  --known-hosts /path/to/phone-known-hosts
```

Both helpers prompt for the phone password; automation can use their
`--password-env VARIABLE_NAME` option without storing a password in source.
Use a verified SSH known-hosts file. This session used `/tmp/poco-f1-known-hosts`.

The exporter fetches the build through the server, with only runtime store
paths missing on the particular connected phone. The NAR is therefore an
incremental artifact for that phone's existing store, not a standalone system
image or universal release. `closure.json` records required paths and checksums.

The installer verifies the local artifact and transferred copy, imports it
through the phone's Nix store, creates a persistent GC root at
`/var/lib/spectre-camera/gcroots/megapixels`, and installs a managed user launcher
at `~/.local/share/applications/me.gapixels.Megapixels.desktop`. It refuses to
overwrite an unrelated user launcher. `installed.json` records the deployment.
No boot, userdata or system-image partition is changed.

## Runtime evidence and limits

- Existing-kernel raw capture: 5 frames received; saved RAW10 frame was
  15,240,960 bytes (4032x3024, stride 5040).
- Missing config fixed: config discovery identifies one rear camera and the
  expected media/sensor/video paths.
- The rebuilt app starts OpenGL ES 3.2 and no longer reports the original
  queue error after the initialized-plane patch.
- Photo-save regression testing after the blank-frame fix is in progress.
- First diagnostic RAW frame was extremely dark, consistent with the lens
  facing an unlit/covered scene; image quality is not established by byte count.
- A screenshot showed the lock screen, not the app preview. The owner was
  asked to unlock and aim the rear lens at a lit scene for visual validation.
- Autofocus, image tuning, flash and other cameras are not claimed as working.
  The pinned board tree declares the actuator but lacks a sensor `lens-focus`
  reference; no actuator entity appeared in the inspected media graph. This
  is a separate focus-integration limitation, not evidence that a kernel change
  is needed to fix the already working raw streaming path.

Local build logs, RAW diagnostics, screenshots and retrieval-independent test
evidence live under `codes/spectre-nix/.code-sync/camera-tests/`. Transport
artifacts live under `codes/spectre-nix/artifacts/camera-live/`. Neither is
synchronized as source.

## Qt camera application

The separate [Spectre Camera Qt app](qt-camera.md) now uses libcamera
and its software ISP, with local IMX363 analogue/zero-digital-gain and first-frame startup fixes.
Native regressions and repeated live capture pass; actual scene quality is
still under investigation. The old custom RAW conversion has been superseded.

## Rollback

Close the test camera app, then remove the managed user desktop file to restore
the system launcher. If that file has a `.previous` backup, inspect it before
restoring it (it may point to an earlier test package). The original system
package is untouched. An initial user-local config was also placed at
`~/.config/megapixels/config/xiaomi,beryllium.conf`; retaining it lets the old
system app discover the rear camera. Delete the GC-root symlink only if the
test package should become eligible for normal Nix garbage collection.
