# Xiaomi HDR compatibility helper for the Poco F1

Implemented and tested on 2026-10-20. This is a narrowly scoped adapter for the
owner's **beryllium / IMX363 Sunny camera**. It executes Xiaomi's original
`libarcsoft_high_dynamic_range.so` in a separate ARM32 process on the current
NixOS kernel. It does not replace libcamera, Qt, Android system services or the
kernel. It does not implement the full CamX/CHI hardware ISP pipeline.

## Implemented boundary

```text
Caller supplies 3 processed YUV frames + explicit AE data
    -> validate format, dimensions, sizes and pinned bundle
    -> private PID namespace
    -> original Android linker + Bionic runtime
    -> ARM32 camera helper
    -> original ArcSoft HDR Init / PreProcess / Process / Uninit
    -> tightly packed YUV output + JSON result
```

The source is `modules/camera/xiaomi-compat/`:

| File | Purpose |
| --- | --- |
| `arcsoft-abi.h` | Recovered, size/offset-checked ARM32 image and parameter structures. |
| `helper.c` | Native worker; validates and copies image buffers, calls the original engine, emits JSON. |
| `run.sh` | Phone launcher; verifies checksums, creates an unprivileged PID namespace, invokes the bundled linker. |
| `client.py` | Optional Python 3.11+ job API with validation, private input copies, deadline and atomic output publication. |
| `blobs.json` | Exact SHA-256 pins for this ROM's engine, helper library and runtime. |
| `extract.py` | Read-only `debugfs` extraction from the already reconstructed system/vendor images. |
| `build.py` | Build-host-only Android cross-build; creates a versioned bundle with source/compiler/file manifest. |
| `test.py` | Invalid-job tests and actual-engine ARM-emulated integration/failure tests. |
| `phone-test.sh` | Native processing, repeatability, size and input-preservation checks using artificial fixtures. |

Only the HDR engine is currently enabled. Low-light HDR and SuperNight remain
unimplemented. The active Qt app does **not** call this worker yet: it needs
real exposure-bracket capture, correct YUV conversion/colour range, AE mapping,
frame-order validation and asynchronous integration first. Synthetic fixtures
do not establish the appropriate exposure sequence for real photos.

## Exact ABI and evidence

The engine hash is
`3e2a5db30a41b17e28d79ace12b60c5df0f7bc4c6b8688a403f624d18c65df0a`.
The original node hash is
`8c9c7eea9f450e9ac6ffb6a7be315ad16c152f59af55d05301fbe5298293fbd6`.
No proprietary executable was patched. The helper is new source code calling
the exported API; Bionic and its dependencies come from the same ROM.

REA/Ghidra evidence is under the build host research directory
`<camera-research-dir>/evidence/`:

| Evidence | Finding |
| --- | --- |
| `compat-offscreen.json` | 44-byte image: format/width/height, four 32-bit plane pointers at offset 12, four strides at offset 28. NV12 `0x801`, NV21 `0x802`. |
| `compat-hdr-process.json` | Original node allocates 136-byte input structure (count + three images); rear Init mode 0, parameter 3 = 0; copies 24 AE bytes; preprocesses each index; writes output into final input buffer. |
| `compat-ae-info.json` | First four AE float words are lux index, ISP gain, sensor/BPS gain, ADRC gain; remaining two are vendor constants read from the node's data. |
| `compat-engine-init.json` | Three Init arguments, 120 MiB memory-pool reservation and camera-mode state. |
| `compat-engine-default.json` | Four integer tone/brightness/saturation/contrast parameters; GetDefaultParam depends on Init state. |
| `compat-engine-preprocess.json` | Input and AE pointers are retained; their storage must remain valid until processing completes. |
| `compat-engine-process.json` | Actual original engine processing; includes thread-affinity operations, so running outside the camera UI thread matters. |
| `compat-hdr-node-snapshot.json`, `compat-hdr-engine-snapshot.json` | Saved REA evidence sessions. |

Ghidra node addresses include its `0x10000` image base. Raw ELF addresses
`0x8020` and `0x8024` contain float 4 and float 8; GetAEInfo writes those to
AE word 5 and word 4 respectively. The test's `[50,1,1,1,8,4]` matches the
node's relevant fallbacks/constants, **not measured exposure metadata**. The
last two fields are kept as raw ABI words instead of assigning unverified
semantic names. The native default parameters `[20,-15,5,50]` also match the
rear-camera parameter block at ELF `0x80c8`.

The worker requires exactly three input frames and forwards their order. It
accepts even dimensions between 64 and 4032, capped at 4032×3024 pixels. It
converts tight file rows to 64-byte-aligned strides internally. Files contain
8-bit Y followed by interleaved UV (NV12) or VU (NV21), without a header. HDR
requires consistently processed inputs; arbitrary JPEG-to-YUV conversion is
not a calibrated substitute for suitable camera frames.

## Runtime compatibility and lifecycle

The dependency chain includes original `libmpbase`, Bionic `libc/libm/libdl`,
`liblog`, and `libc++` (needed by liblog). The Android linker is invoked directly,
so no `/system` or `/apex` installation, root, chroot or service is required.
The phone's observed kernel has `CONFIG_COMPAT=y`.

The first server-emulation probe exposed an actual Bionic constraint: its
ARM32 mutex owner encoding rejects PID/TID above 65535. The final launcher
therefore **always** uses a private PID namespace via unprivileged user
namespaces. `unshare --user --map-root-user --pid --fork --kill-child` was
tested on the phone. Do not remove this wrapper merely because a low-PID test
works without it. No system PID limit was changed.

The worker has a 90-second alarm and 1 GiB virtual-address-space ceiling; core
dumps are disabled. Python callers can set a shorter deadline. Killing the
namespace supervisor also kills its child. The worker refuses to replace
existing output; the Python API publishes staged output atomically only on
success. Nonzero engine status is surfaced as an error, including `0x1a`, which
the stock wrapper treats as acceptable but whose meaning we have not verified.
The boundary handles process/ABI failures; it is not a filesystem sandbox.

## Build and tests

All compilation ran on the build host. The official NDK r30
Linux archive was verified against the published SHA-1
`5107f898313790e449e87eee2183d9a20602dee9`; the build manifest captures the
actual compiler identity. Toolchain source: [official NDK downloads](https://developer.android.com/ndk/downloads).
Build/extraction commands and the JSON job schema are in
[`modules/camera/xiaomi-compat/README.md`](../modules/camera/xiaomi-compat/README.md).

The server's existing QEMU ARM interpreter resides in its private Nix store.
The reproducible integration command is:

```sh
cd <repo-root>
bwrap --unshare-pid --ro-bind / / --bind /tmp /tmp \
  --tmpfs /nix --dir /nix/store \
  --ro-bind ../nix-store/nix/store /nix/store \
  --dev-bind /dev /dev --proc /proc \
  /usr/bin/python3 modules/camera/xiaomi-compat/test.py \
  --bundle <camera-research-dir>/build/compat-0.1.1 \
  --emulator /nix/store/r1p7gnaj9dls61pfdn911fg0c1gvxmvf-qemu-user-11.0.1/bin/qemu-arm
```

Passed: four boundary test methods (including malformed schema/camera/format,
size, nonfinite AE, truncated input and existing-output subcases), actual HDR
NV12/NV21 processing at 640×480, padded stride at 642×480, native wrong-format
rejection, corrupted-engine rejection, and a 10 ms timeout that leaves no
published output or staging directory. The initial one-second timeout test
was too generous: the synthetic job completed, so the test was tightened.

Native phone validation initially measured 28.973 ms for 640×480 and
346.040 / 338.840 ms for full 4032×3024 synthetic brackets. These worker timings
include Init, processing, Uninit and output write; exclude input loading,
launcher/checksum verification and acquisition of real frames. They are not
app shutter latency or a performance guarantee.

The 640×480 output hash exactly matched server emulation:
`998f784788a4829cbd43866678d477ab50a99c2648c3ef23a88bf2a2d37a0326`.
Full-resolution output is 18,289,152 bytes with hash
`fb755aa601625ffd79572d0d9765ca82ffdf9edd67e94fa20230fcfdad542072`;
repeated runs were identical. Input files were preserved. The camera app was
still running as PID 4039 after the initial native checks; no app restart,
kernel replacement or flash was performed.

The final installed `compat-0.1.1` bundle passed the same full-resolution native
test at 353.387 / 335.226 ms, with the same output hash and all input checksums
unchanged. Report: `artifacts/xiaomi-compat/compat-0.1.1-phone-test.txt` in the
local source checkout and `evidence/compat-0.1.1-phone-test.txt` on the build host.

Server report: `evidence/compat-0.1.1-server-tests.json`. Local bundles/reports
are under `codes/spectre-nix/artifacts/xiaomi-compat/`, excluded from code sync.
The standalone phone bundle is
`/home/spectre/.local/lib/spectre-camera/compat-0.1.1/`; invoke its `run probe`
to verify it. No launcher/settings change connects it to the Qt app yet.

## Remaining quality work

This proves that the original HDR algorithm executes and processes buffers on
our current system. It does **not** prove stock-quality photographs. Correct
RAW colour, white balance, lens shading, focus and suitable exposure brackets
remain prerequisites. Low-light HDR, SuperNight and the proprietary BPS/IPE
pipeline are outside this first helper. See [the recovered stock pipeline](camera-pipeline.md).
