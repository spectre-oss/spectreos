# Initial remote_control validation — 2026-11-23

This is the historical first-delivery report. Current deployment, device-only
indicator semantics, persistent client and stream evidence are documented in
[stateful_validation.md](stateful_validation.md).

## Deployed build

Poco F1 (`<user>@<device-ip>`), Wisp `development` feature, optimized aarch64
binary. `wisp-boot.service` is active and `~/run-wisp.sh` exports
`WISP_REMOTE_CONTROL=1`. The socket is owned by the device user with mode 0600 under the
0700 runtime directory. The local build and deployed binary have matching SHA-256:

```text
a760fea0d3b593535605a4c60c3b4639aaf9ab53063ed73ef32ef6360f6c3236
```

Backups: `~/wisp.before_remote_control` and
`~/run-wisp.sh.before_remote_control`. Deployment used an uploaded
replacement and atomic rename, then SIGTERM of Wisp's user-owned main process;
systemd restarted it. No NixOS rebuild or privileged service operation was needed.
The existing runtime linker/library setup was preserved.

The standard build command is `make build_development` in `codes/apps/wisp`.
This host cross-compiled with its existing extracted Poco Nix closure and
`aarch64-linux-gnu-gcc`. Exact target environment values are saved locally in
`evidence/cross_build_environment.json` (gitignored).
These paths are specific to the build host. To reproduce:

```python
import json, os, subprocess
from pathlib import Path
wisp = Path("codes/apps/wisp")
env = os.environ.copy()
env.update(json.loads((wisp / "docs/remote_control/evidence/cross_build_environment.json").read_text()))
subprocess.run(["cargo", "build", "--locked", "--release", "--target",
                "aarch64-unknown-linux-gnu", "--features", "development", "--bin", "wisp"],
               cwd=wisp, env=env, check=True)
```

## Automated checks

- `cargo check --locked --features development`: passed.
- `cargo check --locked --features compositor`: passed, verifying the production
  compositor still compiles without remote_control.
- Six Rust tests passed: frame bounds, command parsing/unknown fields, touch and
  key transitions, socket handshake/status/disconnect, unsupported protocol version.
  [Output](evidence/rust_tests.txt).
- Five Python client tests passed: fragmented frames, interleaved live frames and
  command replies, oversized frame rejection, EOF, and server errors.
- Python compilation and viewer shell syntax checks passed.
- Full Wisp tests exposed two existing failures:
  `slow_mo_notification_bar_scrub_and_collapse` and
  `overview_drag_right_moves_older_cards_into_center_and_swipe_up_dismisses`.
  Both also fail with remote_control disabled (26 passed, two failed).
  [Default-suite output](evidence/default_suite.txt). No unrelated gesture behavior
  was changed to make these tests pass.

## Device checks

[Machine-readable results](evidence/device_checks.json) were collected over the
SSH-forwarded connection using `src/remote_control/check_device.py`:

- Full 1080×2246 PNG capture, valid RGBA pixels, upright red banner at the top.
- Second controller rejected without replacing the current controller.
- Invalid touch slots, coordinates, transitions and keycodes rejected.
- Two simultaneous contact states, cancellation, held-key tracking and release.
- Stream produced eleven full-resolution frames in 3.173 seconds, about 3.5 FPS,
  with heartbeats and command replies on the same connection.
- Abrupt socket closure released held contacts and modifier keys.
- An independent no-input test measured idle disconnect at 9.96 seconds and
  confirmed that the next session had zero held keys/contacts.
- Sleep rejected capture; wake retained the lock screen. No credentials were read
  or entered and the protocol does not unlock the device.
- The native Tk viewer decoded twelve live PNG frames during a four-second Xvfb
  smoke test. The `view.sh` wrapper was also exercised. System Tk libraries were
  missing on this host; an unprivileged runtime was prepared under
  `~/.local/share/wisp_remote_control/viewer_runtime/`. No system packages changed.

## Visual and interactive checks

- [Launcher](evidence/poco_home.png) and [app drawer](evidence/poco_drawer.png):
  remote taps opened the drawer; launcher navigation worked.
- [Lock screen](evidence/poco_lock.png): upright capture with visible session banner.
- [Native Wayland app](evidence/poco_wayland.png): Nautilus displays an empty
  temporary validation folder with Wisp chrome and the remote banner.
- [Native app, popup and keyboard](evidence/poco_wayland_keyboard.png): remote
  Ctrl+L and typed `/tmp` reached Nautilus; the location popup and Stevia OSK are
  captured upright in the same frame. Escape cancelled location editing.
- The user confirmed that touching the phone removes the banner. The compositor
  also logged `remote_control ended by local input`.

The initial vertical readback flip was corrected before these final captures.
Both Bhoo chrome and native Wayland/OSK content now have matching orientation.

## Remaining limits

PNG streaming is capped at five FPS; this device measured about 3.5 FPS. GPU
readback and scene replay run on the compositor thread, so active capture has a
frame-time cost; encoding and socket writes run on a worker, and there is no
readback when no capture is requested. This is not a 60 FPS video transport.

Keyboard input is forwarded through XKB to native Wayland apps. Shell input is
limited to the keys already represented by Bhoo's existing input model. No audio,
clipboard, file transfer, hardware encoding or lock bypass is included. V1 has
only been exercised on this Poco's current display size/normal transform.
