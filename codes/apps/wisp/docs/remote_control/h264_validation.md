# H.264 remote video validation

Validated on Poco F1 on 2026-11-23. This supersedes the earlier PNG-only streaming
reports; those reports and their evidence remain available.

## Delivered behavior

Wisp development builds encode a persistent H.264 Annex B stream with software
OpenH264. The Rust viewer decodes it natively; the stateful CLI forwards encoded
access units to attached viewers and uses the Rust helper for its decoded cache.
Periodic and requested IDRs, sequence/epoch validation, bounded queues, and
sleep/wake resets provide recovery without silently reconnecting after takeover.
The physical-only red badge and captured blue tap rings remain enabled. CLI
keyboard, touch and ordered batches share the same connection.

## Device evidence

[Machine-readable checks](evidence/h264_final/checks.json) and the reusable
`src/remote_control/check_h264.py` check actual encoded access units, SPS/PPS/IDR,
commands during video, stale-frame rejection on sleep, wake recovery and a
full-resolution PNG screenshot.

| Measurement | Result |
| --- | --- |
| Requested / measured rate | 15 / 13.19 FPS |
| Encoded dimensions | 720×1496 |
| Full-resolution screenshot | 1080×2246 |
| Key acknowledgement mean / maximum | 53.0 / 101.8 ms |
| Requested IDR | 128.3 ms |
| Wake to new-epoch IDR | 188.5 ms |
| Static-screen encoded payload | 38.45 kbit/s over 5.078 seconds |

These are a short device sample, not a guarantee for moving content or other
networks. Encoder target is 3 Mbit/s. No hardware acceleration or audio is claimed.
Raw Annex B files lack container timing; use protocol timestamps for playback.
[Recorded MP4](evidence/h264_final/demo.mp4) uses the existing host recorder:
75 frames over five seconds at 15 FPS, with 61 unique decoded source frames.

A direct native Rust connection decoded ten H.264 frames successfully
([snapshot](evidence/h264_final/native_direct.png)). Native GUI smoke checks
verified mouse touch, keyboard Shift, release on detach and a surviving CLI
connection with zero held contacts/keys. The device stayed locked; no credential
was needed or recorded. Legacy v1 stream requests with omitted codec still
returned full-resolution PNG. New clients explicitly negotiate H.264.

## Builds and tests

- Wisp remote_control tests: 7 passed; includes encoder/decoder round trip.
- Native Rust viewer tests: 8 passed; includes skipped-access-unit IDR recovery.
- Python controller tests: 16 passed; includes bounded queue overflow, IDR joining
  and rejection of cached frames while sleeping.
- ARM64 release development build and native release viewer build passed.
- Production compositor check without remote_control passed.

Logs are in [evidence/h264_final](evidence/h264_final). Existing unrelated gesture
suite failures are recorded in the older validation; they were not part of this
change. Dependencies are pinned in both Cargo lockfiles: openh264/openh264-sys2
0.9.10, bundled Cisco OpenH264 2.7. See the repository third-party notices and
[upstream codec license](licenses/OpenH264-BSD-2-Clause.txt).

## Deployment

Local ARM64 binary and `~/wisp` on Poco have matching SHA-256:
`ab03964e1766f36c3ffbf6be99211277561b1d6cc94ad5142344bad777d9dbf9`.
`wisp-boot.service` is active. The source build requires the existing ARM64 C++
runtime; `~/run-wisp.sh` now prepends the cross-compiled GCC runtime `lib` directory
to `LD_LIBRARY_PATH`. Without it the initial deployment could not load
`libstdc++.so.6`; this was fixed and the service retested. Pre-change backups are
`~/wisp.before_h264` and `~/run-wisp.sh.before_h264`.
The persistent `poco` session is restored using SSH port 8795, codec H.264, 15 FPS.
