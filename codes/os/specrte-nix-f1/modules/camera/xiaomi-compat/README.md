# Poco F1 Xiaomi camera compatibility helper

This is a camera-specific, out-of-process adapter to the **original ARM32
ArcSoft HDR engine** extracted from the owner's Poco F1 MIUI ROM. It keeps the
existing NixOS, Qt and libcamera stack. It is not a general Android layer or a
replacement Qualcomm camera HAL.

It currently implements three-frame YUV HDR. It does not enable low-light HDR,
SuperNight, autofocus, RAW ISP processing, or automatic bracketing in the app.
The live app does not invoke it yet. The complete integration/ABI evidence and
observed results are in [camera-xiaomi-compat.md](../../../docs/camera-xiaomi-compat.md).

## Build on the build host only

The original images must already be reconstructed. Extraction uses read-only
`debugfs`, never mounts them, and checks exact SHA-256 hashes. Proprietary files
and generated bundles stay outside the source tree.

```sh
cd <repo-root>
python3 modules/camera/xiaomi-compat/extract.py
python3 modules/camera/xiaomi-compat/test.py
python3 modules/camera/xiaomi-compat/build.py \
  --out <camera-research-dir>/build/compat-0.1.1
```

`build.py` defaults to the server's `camera-research/tools/android-ndk-r30`.
Pass `--ndk` to select an existing toolchain. API 29, ARMv7 soft-float ABI,
PIE and the original Android 10 Bionic runtime match the inspected ROM.
The output directory must not exist. The manifest records compiler identity,
source hashes and every packaged file; no Nix store import is needed.

## Native phone API (no Python required)

```sh
/path/to/bundle/run probe
/path/to/bundle/run hdr nv12 4032 3024 \
  /path/frame-0.yuv /path/frame-1.yuv /path/frame-2.yuv /path/new-output.yuv \
  50 1 1 1 8 4
```

Those six AE numbers are **synthetic-test / stock fallback values**, not a
calibration for arbitrary real photos. The caller must supply suitable AE data
for a real capture. Output must be a new file. Input is tight 8-bit NV12 or
NV21; the worker copies it into aligned/padded memory and writes tight output.
The worker's single JSON stdout record reports stage/error or success/timing.

The `run` launcher verifies the bundled checksums and uses unprivileged
`unshare --user --map-root-user --pid --fork --kill-child`. This is required
because the original 32-bit Bionic mutex owner field cannot handle large PIDs.
The worker has a 90-second alarm, a 1 GiB address-space ceiling and no core
dumps. This process boundary isolates ABI/lifetime failures; it is **not a
filesystem security sandbox**. Nothing starts a camera service or accesses the
sensor. Do not put this processing in the preview thread.

## Python job API (host with Python 3.11+)

```json
{
  "schema": 1,
  "camera": "beryllium-imx363-sunny",
  "format": "nv12",
  "width": 4032,
  "height": 3024,
  "frames": ["frame-0.yuv", "frame-1.yuv", "frame-2.yuv"],
  "ae_info": [50, 1, 1, 1, 8, 4],
  "output": "new-output.yuv"
}
```

```sh
python3 client.py hdr --bundle /path/to/bundle --job /path/job.json --timeout 60
```

Paths are relative to the job file. `client.process_job()` provides the same
API for callers. It validates the camera ID, format, dimensions, file sizes and
AE data; verifies the bundle; copies inputs to a private staging directory;
runs a killable child; and atomically publishes a new output only on success.
The phone currently has no Python in PATH, so use its native `run` launcher.
The C worker independently checks the image contract, even without Python.

## Tests

`test.py` runs contract validation tests. `--bundle PATH --emulator QEMU_ARM`
additionally runs the original engine on artificial brackets in both YUV
formats and with padding, rejects a corrupted blob and invalid native input,
and tests timeout cleanup. Run this only on the build server in its Nix store
namespace; see the documentation for the exact command.

Generate fixtures on the build host:

```sh
python3 modules/camera/xiaomi-compat/test.py --fixtures /tmp/hdr-fixture \
  --width 4032 --height 3024
```

After transferring the fixtures and bundle, run on the phone:

```sh
sh phone-test.sh /path/to/bundle /path/to/hdr-fixture 4032 3024
```

These tests prove executable compatibility, output shape and determinism.
They do not establish photo quality or the correct real-camera exposure
sequence. Real bracketing/AE mapping and comparison photographs are required
before enabling this backend for normal photo capture.
