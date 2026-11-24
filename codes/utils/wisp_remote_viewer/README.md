# Wisp remote viewer

Native Rust viewer for Wisp's [remote_control protocol](../../apps/wisp/docs/remote_control/protocol.md).
H.264 frames are decoded with OpenH264 and displayed with winit and softbuffer on Linux X11 or Wayland.
There is no Python/Tk dependency in the viewer. All viewer source, dependency
lockfile and tests live in this directory.

## Run

From the repository root, attach to the already-running persistent CLI session:

```sh
cargo run --release --locked --manifest-path codes/utils/wisp_remote_viewer/Cargo.toml -- --session poco
```

The existing launcher now runs this Rust viewer too:

```sh
codes/apps/wisp/src/remote_control/view.sh --session poco
```

`--session poco` is the default. It shares the CLI's stream and commands, so its
ordered batches remain usable while the viewer is open. **Detach** closes only
the viewer; it releases the viewer's held input and leaves the CLI connected.
The existing controller uses an owner-only local Unix socket. Use
`--control-socket PATH` to select that socket explicitly, for example when the
controller was started with a different runtime environment.

For a completely native viewer connection, forward the phone socket through SSH
and connect directly (first stop another controller if it already owns Wisp):

```sh
ssh -N -i ~/.ssh/<device-key> -o IdentitiesOnly=yes -o ExitOnForwardFailure=yes \
  -L 127.0.0.1:8765:/run/user/1000/wisp_remote_control.sock <user>@<device-ip>
cargo run --release --locked --manifest-path codes/utils/wisp_remote_viewer/Cargo.toml -- --connect 127.0.0.1:8765
```

Direct mode implements hello, stream reception, command acknowledgements and
heartbeats in Rust. **Stop** closes its Wisp connection. `--socket PATH` connects
directly to a local Wisp Unix socket instead of an SSH-forwarded TCP port.
No connection is automatically re-established after local takeover or failure.

## Controls

- Left mouse drag: first touch contact. Right drag: second contact.
- Native touchscreen: up to ten simultaneous contacts, sharing the slot allocator
  with the mouse so contacts cannot collide.
- Physical keyboard: evdev key down/up, including modifiers, navigation keys,
  function keys, keypad and common text-entry keys. The phone's XKB layout applies.
- Toolbar: Wake, Sleep, Back, Home, Recents, and Detach/Stop.
- Losing window focus releases all viewer-held keys and touch contacts. Normal
  close also releases held input in attached mode, without stopping the CLI.

Window scaling preserves the phone's aspect ratio. Black letterboxes and toolbar
clicks never become phone taps; dragging outside the image clamps the contact to
the phone edge. Consecutive moves for one contact are coalesced, while key and
press/release ordering is preserved. An overflowing command queue stops the
viewer worker rather than silently dropping a release event.

Phone-side blue tap rings arrive in the stream. The red remote-control indicator
remains visible only on the physical phone and is not overlaid by this viewer.
Local phone input revokes control as before. The viewer requests H.264 at 15 FPS and supports legacy PNG. It decodes encoded
access units directly in both attached and direct modes. Decoder resets, missing
sequences and new epochs trigger IDR recovery; cached deltas are never treated
as independent images.
No audio, clipboard, recording encoder or batch executor is duplicated here;
those existing CLI capabilities remain available while attached.

## Validation

```sh
cargo test --locked --manifest-path codes/utils/wisp_remote_viewer/Cargo.toml
cargo run --release --locked --manifest-path codes/utils/wisp_remote_viewer/Cargo.toml -- \
  --session poco --headless --frames 5 --output /tmp/rust_stream.png
```

Headless mode decodes actual stream frames without a display. The optional
`--exit-after-frames N` graphical flag enables bounded GUI smoke checks.
[H.264 validation](../../apps/wisp/docs/remote_control/h264_validation.md) records current checks;
[validation.md](validation.md) preserves the earlier PNG input-lifecycle checks.
`tests/gui_smoke.py` is opt-in test instrumentation requiring a private Xvfb
display, XTest, FFmpeg and the connected/unlocked Poco; it is not used at runtime.

## Dependencies and license

Original source is GPL-3.0-only, matching the repository. `Cargo.lock` pins the
complete dependency graph. Direct dependencies in the validated build:

| Dependency | Version | License | Purpose |
| --- | --- | --- | --- |
| winit | 0.30.13 | Apache-2.0 | Window, events, physical input |
| softbuffer | 0.4.8 | MIT OR Apache-2.0 | Native CPU pixel presentation |
| png | 0.17.16 | MIT OR Apache-2.0 | Legacy PNG decoding and snapshot encoding |
| openh264 | 0.9.10 | BSD-2-Clause | H.264 decoder, bundled Cisco OpenH264 2.7 |
| serde_json | 1.0.154 | MIT OR Apache-2.0 | Protocol metadata |
| font8x8 | 0.3.1 | MIT | Toolbar bitmap glyphs |

PNG sizes and frame lengths are bounded before allocation. A single latest-frame
slot bounds display memory. GPU readback and the physical-only indicator remain
owned by Wisp; this utility changes no phone compositor configuration.

`--decode-video` is an internal framed stdin/stdout H.264-to-PNG helper for the
stateful Python controller. The native viewer does not need Python or this subprocess.
