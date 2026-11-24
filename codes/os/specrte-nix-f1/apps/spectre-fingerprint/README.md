# Fingerprint Lab

Version 0.3 connects the GTK4/GJS app to the working **gfdip** protocol:

- Enroll captures one finger, sends SAVE, and requires enumeration to confirm it.
- Saved fingerprints queries the isolated group 1000.
- Check enrolled fingerprint reports a match separately from a completed rejection.
- Live touch prompts and remaining-sample counts arrive over the local socket.
- The existing scroll container and Scroll down button remain available.

The package builds the helper and one combined RPMB/FS/GPFS broker from pinned,
hash-checked source archives. `modules/fingerprint-lab.nix` starts the broker
before the app service and configures `/var/lib/spectre-fingerprint` with mode
0700 (files 0600). This supplies disk-backed template storage on Linux; actual
phone reboot persistence has **not yet been tested**. The GTK app is not installed
on the current Android system.

The unprivileged GUI talks to a root service over a mode-0600 Unix socket, checks
peer UID, and accepts fixed actions only. Hardware requests are serialized.
The fixed helper uses firmware from a read-only modem mount and restores its
search path afterward. No firmware blobs or biometric data are bundled or
printed. Capture runs for at most 100 seconds, with a 125-second service deadline.
SIGTERM/SIGINT request capture cancellation and normal GPIO/session cleanup.

This remains a **single-fingerprint diagnostic**, not screen unlock. Enrollment
creates a random credential for Gatekeeper UID 0x53504650, separate from Android
UID 0, and wipes the credential/handle after obtaining the token. A nonempty
isolated group blocks another enrollment before changing that credential.
Matching imports the wrapped key but does not create another credential. There
is no delete/reset UI or production password-bound authorization policy yet.
Do not manually remove storage state to bypass the enrollment guard.

The app requires the experimental QSEECOM kernel; the Poco device configuration now selects its transport when the app is
enabled. This does not enable desktop authentication.

## Build and install

On the build server, from the project root:

```sh
nix-build --store ../nix-store \
  --option build-users-group '' apps/spectre-fingerprint/live.nix \
  -o .build/roots/result-fingerprint-app
python3 -m unittest discover -s apps/spectre-fingerprint -p 'test_*.py'
```

`modules/fingerprint-lab.nix` supplies the package and root service. The main
configuration enables `spectre.fingerprintLab.enable`; its allowed user
is `spectre` by default. This source change takes effect on the next system
build/install. The Poco device configuration enables the matching QSEECOM
kernel transport alongside the app.

Earlier live GUI tests used a temporary Linux service. Those results predate
version 0.3 and are not proof that the new GUI works on the phone.

## Verification

- Fourteen backend tests cover successful hardware results, invalid/absent OTP,
  zero geometry, missing kernel devices, the power-off error, malformed output,
  unknown versus zero template count, and rejection of false enrollment/match success.
- `spectre-fingerprint --smoke-test` opens the GUI, runs the same callback as
  the button, reports JSON and exits with a nonzero status if it fails.
- `spectre-fingerprint --check-on-start` runs a check and keeps the window open.
- On-phone concurrency check: one request passed; three overlapping requests
  were rejected as busy. Firmware path was restored; Wi-Fi, audio-card detection
  and baseline kernel taint (512) remained unchanged.
- Compositor screenshot capture failed on this phone; this is not evidence
  of a GUI failure. The GUI ran its hardware check successfully on Wayland.

- `--ui-smoke-test` opens an independent window, expands long details, clicks
  Scroll down, and checks that the viewport moves. Test windows use NON_UNIQUE
  so an already-open app cannot swallow the test.

Version 0.3 validation: ARM Nix package builds; backend tests include real
capture/positive/negative transcripts and streamed progress from a child process;
JavaScript parses; pinned upstream storage protocol/notify/loader tests pass.
New GUI rendering and disk-backed reboot persistence remain untested on hardware.
The earlier on-phone concurrency, scrolling and smoke-test results above refer
to version 0.2.

See `../../kernel/fingerprint/ENROLLMENT.md` for current hardware evidence.
