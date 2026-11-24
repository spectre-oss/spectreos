> Historical PNG validation. Current encoded-video implementation and evidence: [H.264 validation](../../apps/wisp/docs/remote_control/h264_validation.md).

# Rust viewer validation — 2026-11-23

`codes/test-utils/` contained only `.gitkeep` and had no source/documentation
references. It was renamed to `codes/utils/`; the native viewer was added under
`codes/utils/wisp_remote_viewer/`. The repository utilities index and Wisp viewer
launcher/documentation now point to this crate.

## Build and tests

- `cargo check --locked`: passed.
- `cargo test --locked`: seven tests passed. [Output](evidence/rust_tests.txt).
- Debug and optimized release compilation passed. [Release build output](evidence/release_build.txt) and [machine-readable checks](evidence/checks.json) record the result.
- Tests cover fragmented frame parsing with binary payloads, oversized frame
  rejection, PNG colors, aspect-ratio coordinate mapping, contact/keyboard focus
  cleanup, motion coalescing without reordering releases, and detaching while a
  key is held without stopping the persistent controller.

## Live Poco stream and graphical checks

Headless attachment to the existing `poco` CLI session decoded five full-resolution
1080×2246 frames. [Captured streamed frame](evidence/attached_stream.png).

The Rust window ran on a private Xvfb X11 display and displayed the actual Poco
stream. [Native window capture](evidence/native_window.png). The window has its
own native toolbar, aspect-preserving image presentation, and no mirrored red
remote-control badge. The blue ring in the capture came from Wisp's live stream.

The opt-in `tests/gui_smoke.py` test used XTest to send a real window mouse press:
the phone controller reported one held touch, then zero after release. It also
sent a physical Shift key-down through the Rust window; the controller reported
one held key. Closing the Rust window while Shift remained held returned the
controller to zero held keys/contacts and left the CLI session connected.
A separate graphical smoke run exited successfully after five received frames.

Direct SSH-forwarded native mode decoded five 1080×2246 frames without a Python
viewer. [Direct stream capture](evidence/direct_stream.png). The updated `view.sh`
launcher also decoded five frames with the optimized release binary and completed
a twelve-frame graphical release smoke run. [Release stream capture](evidence/release_stream.png).
A local phone takeover interrupted an initial direct attempt; it correctly
reported disconnect and did not reconnect automatically. The active persistent
controller is restored after direct-mode checks.

## Scope

The viewer runs on a desktop host, so no phone-side deployment is required.
Wisp's stream stays capped at five PNG frames per second. The Rust viewer receives
and decodes stream events; it never polls Wisp screenshots for video. In attached
mode it reads the existing controller's latest-frame cache through local IPC.
Normal close/focus loss releases viewer input. A forcibly killed attached viewer
cannot execute cleanup; its owning CLI session remains the place to inspect or
release held input. IME text composition, audio and clipboard are not implemented.
The actual Wayland backend is compiled; the graphical integration check used X11.
