# Wisp remote_control protocol v1

One bidirectional connection to `$XDG_RUNTIME_DIR/wisp_remote_control.sock`, an
owner-only Unix socket. Forward through SSH to a local loopback port. No HTTP,
ADB, device TCP listener, root requirement or separate authentication service.
Only one controlling connection is accepted; additional connections are closed.

## Framing

Every message: 4-byte big-endian JSON length, 4-byte big-endian binary length,
UTF-8 JSON object, then binary bytes. Client JSON is limited to 4096 bytes and
client binary length must be zero. Server binary data is a PNG image or H.264 Annex B access unit, at most 32 MiB.
JSON responses have `id` matching the request, `ok`, and fields or `error`.
Stream frames use id 0, `event: "frame"`, `codec: "h264"` or `"png"`, a monotonic
`sequence` and `timestamp_us` relative to connection startup. Frames are pushed
after the stream command; clients do not poll status/screenshot for video.
Client IDs must be positive integers.

First request: `{"id":1,"type":"hello","version":1}`. The response includes
protocol version, physical `width`/`height`, `logical_width`/`logical_height`,
`scale`, `locked`, and `screen_on`. Status replies also report the numbers of
held remote `contacts` and `keys`, plus `unlock_prompt`, `indicator: "device_only"`,
`tap_feedback: true` and `stream_codec` and `stream_codecs: ["h264", "png"]`. Unsupported versions close the connection with an error.
The controller sends `heartbeat` every two seconds. Ten seconds without a command
ends the session. Heartbeats keep an already-on screen awake during remote work;
they never wake/unlock an explicitly sleeping phone. Disconnect/EOF/timeout
releases all remote touches and keys.

## Commands

```json
{"id":2,"type":"status"}
{"id":3,"type":"screenshot"}
{"id":4,"type":"stream","fps":15,"codec":"h264"}
{"id":5,"type":"stream","fps":0}
{"id":6,"type":"touch_down","slot":0,"x":120.0,"y":240.0}
{"id":7,"type":"touch_move","slot":0,"x":120.0,"y":180.0}
{"id":8,"type":"touch_up","slot":0}
{"id":9,"type":"touch_cancel"}
{"id":10,"type":"key","code":30,"down":true}
{"id":11,"type":"key","code":30,"down":false}
{"id":12,"type":"action","name":"back"}
{"id":13,"type":"action","name":"home"}
{"id":14,"type":"action","name":"overview"}
{"id":15,"type":"wake"}
{"id":16,"type":"sleep"}
{"id":17,"type":"heartbeat"}
{"id":18,"type":"disconnect"}
{"id":19,"type":"keyframe"}
```

Coordinates are finite logical display coordinates, origin top-left, bounded by
status dimensions. Slots are 0–9; down must precede move/up. Linux evdev keycodes
1–767 require paired down/up. Malformed framing, unknown types/fields and invalid JSON close the connection.
Invalid coordinates and invalid state transitions return error replies. Events run on the compositor thread, using its
monotonic clock and the same shell/application touch routing as physical input.
Commands are ordered. Success means dispatch, not application completion.

Screenshot replies always contain full-resolution PNG. Streaming accepts 0–30
FPS, with 0 stopping; new clients request H.264 at 15 FPS. `codec: "png"`
retains the legacy stream, and omitting codec selects PNG for old v1 clients.
ACKs report the selected codec and FPS. Unsupported codecs are rejected.

Each H.264 frame is one Annex B access unit, including start codes. Metadata has
`width`, `height` (encoded dimensions), `source_width`, `source_height` (physical
capture dimensions), `stream_id` (encoder epoch), `sequence` (contiguous emitted
access units), `timestamp_us` (capture time), `keyframe` and `dropped_raw_frames`.
Coordinates still use the logical display dimensions from status. H.264 scales
within 720×1600, preserving aspect ratio with even dimensions. The Poco produces
720×1496. Software OpenH264 targets 3 Mbit/s, with low complexity screen-content
encoding; this is a bitrate target, not a bandwidth guarantee.

The first access unit and every requested/periodic IDR include SPS/PPS. IDRs are
forced at least every two seconds during active capture. `keyframe` requests the
next IDR. Clients reset their decoder on a new epoch, missing sequence or decode
error, request a keyframe and discard deltas until an IDR. The Rust clients do
this automatically. Packet timestamps, rather than inferred raw-file frame
rates, define playback timing.

One capture is outstanding at a time. A dedicated encoder worker replaces stale
raw frames in one bounded slot; it never discards an already-encoded reference
frame. One encoded output slot bounds backpressure. Control ACKs are serviced
before video, and a two-second socket-write timeout disconnects slow readers.
Heartbeats and controls remain usable during streaming. Sleep clears cached video;
wake starts a new encoder epoch with an IDR. No stale pre-sleep picture is served.
Explicit wake reaches the lock screen without bypassing authentication.

## Local visibility and control

A persistent red REMOTE CONTROL banner is drawn into the physical shell UI for
the entire connection, including capture-only connections. It is deliberately
excluded from screenshot readback and streamed video. Blue touch rings are native
Smithay overlay elements above the shell, app and keyboard. Rings are captured
as well as physically displayed, and remain for 700 ms after touch release. Any
physical touch immediately disconnects remote control and takes over locally;
the first touch is consumed so stopping does not activate content beneath the
banner. Hardware keyboard presses also revoke control.
The remote controller cannot disable the banner. Session expiry, local takeover,
EOF and explicit disconnect cancel contacts and release held keys.

## Capture and development scope

Build with `development` (enables `compositor` and `remote_control`) and run with
`WISP_REMOTE_CONTROL=1`. Production compositor builds omit `remote_control`.
Full visible element composition is captured into an offscreen GLES buffer,
independent of DRM plane assignment. Readback occurs only on request; image/video encoding
and socket I/O occur off the compositor thread. H.264 currently uses software
OpenH264, not the phone’s Venus hardware codec. No audio, clipboard, file transfer or lock bypass.

## Stateful controller

`session.py` owns one connection and consumes stream events on a dedicated reader
thread while multiplexing command replies by ID. Heartbeats run independently.
A latest decoded-frame cache and a bounded encoded GOP cache (90 access units,
8 MiB total, 4 MiB per access unit) bound host video memory. The private `video`
IPC returns encoded access units with `client_packet_sequence`; joining or lagging
readers restart at an available IDR. A Rust decoder helper supplies PNG cache
images to Python clients; the native Rust viewer decodes the H.264 stream directly. The persistent CLI, viewer and
recorder share that connection through a private host Unix socket. They do not
open additional Wisp connections or request screenshots for video.

The host IPC uses the same JSON/binary length framing, with JSON capped at 128 KiB
and binary capped at 32 MiB. `type: "batch"` accepts a `commands` array. The whole
batch is preflighted before any UI action, then executed under the controller's
input lock. Runtime errors return `completed: false`, `failed_index`, `error`, and
successful-prefix `results`, then release held input. This is ordered execution,
not transactional rollback. Nested batches and arbitrary shell execution are not
supported. Status/stream reads may run concurrently with an input batch.

The host also provides `tap`, `swipe`, `text`, `wait`, `frame` and lifecycle helpers;
these translate into the existing v1 touch/key/stream commands. The phone wire
protocol remains v1. See [README.md](README.md) for CLI fields and examples.
