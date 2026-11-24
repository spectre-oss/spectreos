# remote_control

Wisp's own development protocol: one framed Unix socket connection through SSH
for a continuous H.264 video stream, screenshots and touch/keyboard control. No HTTP
or ADB. See [protocol.md](protocol.md) for the wire format and
[h264_validation.md](h264_validation.md) for current device evidence.

Build with `make build_development`; launch with `WISP_REMOTE_CONTROL=1`.
The ordinary production `build-compositor` target omits remote_control.

## Stateful CLI

Run these examples from the repository root. `session.py` starts a background
controller, maintains one Wisp connection, receives stream events continuously,
and sends heartbeats. Commands in later invocations reuse that same session.
The controller can own its SSH tunnel:

```sh
python3 codes/apps/wisp/src/remote_control/session.py start \
  --ssh <user>@<device-ip> --identity ~/.ssh/<device-key> --port 8795
python3 codes/apps/wisp/src/remote_control/session.py status
python3 codes/apps/wisp/src/remote_control/session.py wake
python3 codes/apps/wisp/src/remote_control/session.py tap 200 710
python3 codes/apps/wisp/src/remote_control/session.py swipe 200 700 200 200
python3 codes/apps/wisp/src/remote_control/session.py action home
python3 codes/apps/wisp/src/remote_control/session.py frame /tmp/poco.png
python3 codes/apps/wisp/src/remote_control/session.py record /tmp/poco.mp4 --seconds 10
codes/apps/wisp/src/remote_control/view.sh --session poco
python3 codes/apps/wisp/src/remote_control/session.py stop
```

`frame` saves a fresh frame received from the stream, without requesting a Wisp
screenshot. `record` consumes that same stream and uses local FFmpeg/libx264 to
write MP4. Output-clock duplication preserves real-time duration on slower links.
`status` reports frame count/age, lock state, held input and command/batch counts.
The phone encodes H.264 in software; new clients default to 15 FPS with a
720×1600 bounding box (720×1496 on Poco). `start --codec png` selects the legacy
full-resolution PNG stream. `screenshot /tmp/full.png` captures a full-resolution
PNG independently; `frame` saves the decoded video resolution. `keyframe` requests
an IDR. MP4 recording uses host-side FFmpeg sampling and preserves wall-clock duration.

The default session is `poco`; use `--name another_name` before the subcommand to
manage a different session. If a tunnel already exists, `start --port 8765`
connects through it. Occupied ports are rejected when the session owns the tunnel.
`stop` releases the Wisp connection, waits for tunnel cleanup and removes the
private local control socket. A stopped or physically revoked connection is never
automatically reconnected: explicitly `stop`, then `start` again.

The viewer attaches to the session; other CLI commands and batches remain usable.
Closing it detaches the viewer and keeps the session alive. `view.sh` launches the
[native Rust viewer](../../../../utils/wisp_remote_viewer/README.md), which uses
winit and softbuffer on X11 or Wayland; it needs no Python/Tk runtime. It accepts
`--session poco` (the default), `--connect HOST:PORT`, or `--socket PATH` for direct
connections. The original Tk viewer remains available through `client.py view`
as a fallback. The Python CLI uses the native Rust binary as an H.264 decoder helper; build it
with `cargo build --release --locked --manifest-path codes/utils/wisp_remote_viewer/Cargo.toml`.
`WISP_REMOTE_CODEC` can select that executable explicitly. PNG mode needs only
Python's standard library; `record` additionally needs FFmpeg with libx264.

## Remote keyboard and password entry

Open the phone's unlock prompt with a swipe, then enter a supplied credential
through actual remote key down/up events:

```sh
python3 codes/apps/wisp/src/remote_control/session.py swipe 200 750 200 260
python3 codes/apps/wisp/src/remote_control/session.py type --password --submit
```

The password prompt does not echo. The CLI neither writes entered text to files
nor includes it in results; a submitted text operation reports the resulting lock
state. `type --stdin` supports input from a secure source. Use `type 'ordinary
text'` for non-sensitive text only. Do not put credentials in argv or batch files.
Text helpers map US keyboard characters to evdev events; `key CODE`, `key CODE
--down` and `key CODE --up` expose individual keys. Existing phone lock checking
is used; there is no remote unlock bypass or password configuration change.

## Batching

```sh
python3 codes/apps/wisp/src/remote_control/session.py batch \
  codes/apps/wisp/docs/remote_control/example_batch.json
```

A batch is a JSON array, also accepted from stdin with `batch -`. For example:

```json
[
  {"type":"action","name":"home"},
  {"type":"tap","x":200,"y":710},
  {"type":"frame","output":"/tmp/poco_drawer.png"},
  {"type":"wait","seconds":0.4},
  {"type":"status"}
]
```

Input batches execute in order and cannot interleave UI input with other batches
or individual commands. Read-only status and live-frame viewing remain available.
All command shapes/ranges are validated before execution. A runtime failure stops
at `failed_index`, returns the completed prefix and releases held contacts/keys.
Completed UI actions are not rolled back. The CLI exits nonzero on a failed batch.
At most 128 steps and 60 seconds of estimated work are accepted per batch.

Steps support `tap`, `swipe`, `key`, `text`, `action`, `wake`, `sleep`, `wait`,
`status`, `frame`, and raw `touch_down`/`touch_move`/`touch_up`/`touch_cancel` events.
Batch frames require an explicit output path; no large binary frames are embedded
in batch JSON results. Coordinates are logical pixels (see status); PNG/video
retain full physical resolution. The example coordinates are for this Poco.

## On-phone visibility

The red REMOTE CONTROL banner is part of the physical phone UI only. It is
excluded from captured frames and recordings. Any physical touch revokes control;
the first touch is consumed to avoid activating underlying content. Hardware keys
also revoke control. Blue touch rings are visible on the phone and in the stream,
including over the keyboard; they last for the contact and 700 ms after release.

Active heartbeats keep an already-on display awake for remote work. Explicit
sleep still works; heartbeats never wake or unlock a sleeping phone. Once the
session ends, ordinary inactivity behavior resumes. Raw connections expire after
ten seconds without a command, while the stateful client sends heartbeats every
two seconds. Disconnect, expiry and local takeover release held input.

## One-off client and manual forwarding

The small original client remains available when no persistent session owns Wisp:

```sh
ssh -N -i ~/.ssh/<device-key> -o IdentitiesOnly=yes -o ExitOnForwardFailure=yes \
  -L 127.0.0.1:8765:/run/user/1000/wisp_remote_control.sock <user>@<device-ip>
python3 codes/apps/wisp/src/remote_control/client.py screenshot /tmp/poco.png
codes/apps/wisp/src/remote_control/view.sh --connect 127.0.0.1:8765
```

Only one connection may control Wisp; detach/stop the persistent session before
using a separate one-off connection. Stream-attached viewers do not need a second
Wisp connection.

## Source and checks

All implementation is under `src/remote_control/`: `protocol.rs`, `server.rs`,
`capture.rs`, `input.rs`, `integration.rs`, `client.py`, `streaming.py`, `session.py`
and `view.sh`. The native viewer lives in `codes/utils/wisp_remote_viewer/`. The private host IPC uses the same two-length framing with a
128 KiB JSON request cap and owner-only Unix socket; it is not a network service.

Run these checks from `codes/apps/wisp/`:

```sh
cargo test --locked --features development remote_control
python3 -m unittest discover -s src/remote_control -p 'test_*.py'
# Explicit device integration check; no stateful session may be active:
python3 src/remote_control/check_device.py --output /tmp/device_checks.json
```

The explicit device checker sleeps/wakes the phone and leaves it locked. There
is no audio, clipboard, file transfer or device-side hardware video encoding.
