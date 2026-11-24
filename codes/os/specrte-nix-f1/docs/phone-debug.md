# Remote Phosh debugging over USB

The pinned Phosh 0.54.0 source is in `phosh/`, fetched by
`python3 scripts/fetch-phosh.py`. Its revision is recorded in `sources.json`;
the upstream checkout and captured images are ignored by Git.
The running shell need not be patched: Phosh/Phoc already expose the required
screen and keyboard interfaces. The project adds an SSH JSON agent and real
Linux virtual mouse/multitouch devices.

## Connection and installation

The phone must be running Linux with USB SSH at `172.16.42.1`, and the host
must have `172.16.42.2/24` on the USB Ethernet interface. Authenticate as
`spectre` using SSH normally; no password is embedded in the tools.

`make phone-debug` builds the ARM64 agent. Future full system images install
it when `spectre.phoneDebug.enable = true` (enabled for this development project).
Set this option to false for a build without input-injection access.
The agent runs only on demand over SSH: there is no HTTP server, listening debug
port, root shell endpoint or automatic AI process.

The current phone also has a user-local deployment at
`~/.local/bin/spectre-phone-debug-agent`; the host client detects either install.
Its `/dev/uinput` ACL is temporary until reboot. Full images use a dedicated
`phone-debug` group and a udev rule instead. The agent must run as the graphical
user, not root. On the existing recovery system, input access can be re-enabled
explicitly with `sudo setfacl -m u:spectre:rw /dev/uinput`.

## Host commands

Run from the project directory:

```bash
# Session and keyboard state (JSON).
python3 scripts/phone-debug.py '{"action":"status"}'

# A native screenshot, without the other phone's camera.
python3 scripts/phone-debug.py --output .build/screen.png '{"action":"screenshot"}'

# Relative pointer motion; distances are input deltas and acceleration may apply.
python3 scripts/phone-debug.py '{"action":"mouse","dx":40,"dy":20}'
python3 scripts/phone-debug.py '{"action":"mouse","click":"left"}'
python3 scripts/phone-debug.py '{"action":"mouse","click":"right"}'
python3 scripts/phone-debug.py '{"action":"mouse","wheel":-3}'

# Request the keyboard, then inspect both its reported state and actual pixels.
python3 scripts/phone-debug.py --output .build/keyboard.png \
  '[{"action":"keyboard","visible":true},{"action":"screenshot"}]'
```

`--control-path PATH` reuses an authenticated SSH ControlMaster. Without it,
SSH prompts normally. `--host USER@ADDRESS` changes the destination.
`--remote-command COMMAND` is for an explicitly selected agent installation.

## Touch, swipes and multitouch

Touch coordinates are normalized from 0 to 1 across the virtual touchscreen.
Use portrait/native panel orientation for reproducible gesture coordinates;
rotation and compositor output mapping can change their relation to screenshot
pixels. Mouse movement is relative, not an absolute pixel-position API.

Each frame lists **all contacts currently down**; omitting a previous contact
releases it. IDs 0..9 are stable slots within a gesture. `delay_ms` is the hold
time after that frame. All contacts are released when the action ends, including
on exceptions. Example two-finger movement:

```json
{"action":"touch","frames":[
  {"points":[{"id":0,"x":0.4,"y":0.5},{"id":1,"x":0.6,"y":0.5}],"delay_ms":80},
  {"points":[{"id":0,"x":0.3,"y":0.5},{"id":1,"x":0.7,"y":0.5}],"delay_ms":80},
  {"points":[],"delay_ms":16}
]}
```

Save an action or list of actions as JSON and pass `@path.json` to the host
client. A list keeps the input devices alive between actions. For a swipe,
interpolate multiple frames with the same ID. A one-contact frame followed by
an empty frame is a tap; increasing the hold time produces a long press.
At most ten simultaneous contacts, 500 frames and 30 seconds per gesture are
accepted. No synthetic hardware keyboard is created, which avoids changing
Phosh's keyboard-detection behavior simply by connecting this tool.

## AI workflow and evidence

1. Capture the screen and inspect its dimensions and current orientation.
2. Read `status`, focus the target field with touch or pointer, and capture again.
3. Check `keyboard_visible` and the screenshot together. The D-Bus property
   alone does not prove the keyboard is unobscured by a panel or lock screen.
4. Inject a bounded gesture and capture the result. Stop on an `ok:false` reply.

Screenshot requests wake the Poco display before capture. An asleep output
initially produced zero-size buffers / `no supported format found`; waking it
made direct Grim screenshots work. This does not wake the separate camera phone.

Live validation: native PNG capture, swipe-to-passcode-screen, mouse movement,
left/right button injection and two simultaneous touch contacts were exercised.
The keyboard's visibility API returns state successfully. Application-specific
pinch behavior and automatic keyboard appearance in every app are not guaranteed
by successful event injection; validate those with before/after screenshots.

## Implementation

- `debug/phone.py`: bounded JSON-lines actions, D-Bus, Grim, Linux uinput.
- `scripts/phone-debug.py`: SSH transport, JSON replies, local PNG saving.
- `modules/phone-debug.nix`: opt-in package and scoped uinput group access.
- `tests/test_phone_debug.py`: invalid input and contact-release regression tests.

Upstream references: https://gitlab.gnome.org/World/Phosh/phosh and
https://world.pages.gitlab.gnome.org/phosh/phoc/ .

### Lock-screen keypad protection

The debug agent refuses to force the external keyboard while logind reports
the graphical session locked. Phosh hides its PIN keypad when the OSK reports
visible, even if the compositor is not showing that keyboard on the lock screen.
`ScreenSaver.GetActive` reports screen blanking and is not used as a lock check.
Forced keyboard visibility in an unlocked session is restored when the agent
exits; use a batch containing show and screenshot to inspect it. Explicit hide
requests remain available to recover a stale visibility state.
