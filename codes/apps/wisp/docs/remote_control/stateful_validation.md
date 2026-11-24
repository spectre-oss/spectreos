> Historical PNG validation. Current encoded-video implementation and evidence: [h264_validation.md](h264_validation.md).

# Stateful remote_control validation — 2026-11-23

This report supersedes the initial banner/capture behavior in
[validation.md](validation.md). [Machine-readable checks](evidence/stateful/checks.json)
and [completed batch results](evidence/stateful/batch_result.json) retain the
current device evidence. No credential is stored in these artifacts.

## Deployment

The Poco runs an optimized `development` aarch64 build with
`WISP_REMOTE_CONTROL=1`. The active service is `wisp-boot.service`; its binary at
`~/wisp` matches the local build's SHA-256:

```text
5c4e4967fcbc1b3d2442f65957b793ef1e70d451eb95f47db04e1637bb9f4888
```

The preceding binary is retained as `~/wisp.before_stateful_remote_control`;
initial deployment backups also remain. Source, protocol, CLI and tests are under
`codes/apps/wisp/`. The cross-build environment is recorded locally in
`evidence/cross_build_environment.json` (gitignored).

## Phone versus captured UI

The user confirmed the red REMOTE CONTROL banner is visible on the physical phone.
It is appended to the main shell draw commands after capture readback, rather
than being a second full-screen Bhoo render element. The stream and recordings
therefore omit it. Local touch/hardware input still revokes the session.

Blue tap rings are native Smithay memory elements above app and keyboard surfaces.
[The streamed tap frame](evidence/stateful/tap_marker.png) shows a complete circle
with no red banner. Decoded-pixel checks confirmed cyan ring pixels on all four
sides of the requested touch position, and no red banner at its reserved screen
position. Explicit full-source texture coordinates prevent clipping the ring
when scaling its texture down to a 24-logical-pixel marker.

## Streaming and persistent client

The background CLI owns one Wisp connection and an SSH forward on localhost:8795.
A dedicated thread receives `event: frame` messages independently of request
replies. Commands are multiplexed by ID; no status/screenshot polling drives video.
The session retains only the latest frame and sends heartbeats every two seconds.
An active controller keeps an already-on display awake; explicit sleep is honored.

[The eight-second recording](evidence/stateful/stream_taps.mp4) was produced from
live stream frames while the ordered UI batch executed over the same session.
It contains forty output frames at five FPS, using twenty-six unique incoming
frames. Repeating cached stream frames maintains correct playback duration when
the phone delivers fewer than five frames per second. FFprobe verified H.264,
1080×2246, and duration 8.000 seconds; [metadata](evidence/stateful/video_metadata.json).
Network payloads remain PNG; H.264 encoding occurs locally through FFmpeg.

The attached Tk viewer received twelve frames during a three-second Xvfb check.
Closing the viewer preserved the connected CLI session. Independent CLI status,
tap, navigation, frame, batch and record commands reused that same stateful
controller. Invalid local port selection was corrected to reject occupied ports;
stop waits for owned-tunnel cleanup before allowing restart.

## Remote keyboard unlock

The lock screen was opened using remote touch input. The user-supplied credential
was entered via the CLI's non-echoing password prompt and actual remote evdev
key-down/key-up events, followed by Enter. The submitted text result reported
`locked: false`. Neither a lock bypass nor configuration modification was used.
The credential was not written into source, batch files, results or recordings.
The persistent session remains unlocked and connected after validation.

## Checks

- Twelve Python tests passed, including continuous frames without polling,
  reply multiplexing, independent heartbeat, batch preflight, serialization,
  release on runtime failure, sensitive-text result omission, work limits and
  navigation/session-name argument separation. [Output](evidence/stateful/python_tests.txt).
- Six Rust protocol/server/input tests passed. [Output](evidence/stateful/rust_tests.txt).
- Production compositor compilation without remote_control passed.
- Python compilation, viewer shell syntax, current documentation links and
  local/deployed binary digest checks passed.
- The two existing gesture-suite failures from the initial implementation remain
  recorded in [the historical report](validation.md); this update does not change
  their shell gesture implementations.

## Limits

Video is currently the existing capped five-FPS PNG stream, with synchronous GPU
readback; it is not a hardware-encoded high-frame-rate stream. The recorder needs
local FFmpeg/libx264, and a GUI viewer needs Tk and a desktop display. Text helpers
map supported US keyboard characters to evdev events; raw key events remain
available. Batches are ordered, stop on runtime failure and release held input;
completed UI effects are not rolled back. A physically revoked session never
silently reconnects. The private controller socket persists for status/stop, and
an explicit stop/start is required before reconnecting.
