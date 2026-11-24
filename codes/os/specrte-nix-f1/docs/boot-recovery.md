# Poco F1 boot recovery

## Current state — 2026-11-04

NixOS 0.12.1 has now been flashed to userdata and boot with the owner’s
authorization; first-boot validation is ongoing. See [the installation record](nixos-return.md)
for exact image hashes, logs and the preserved rooted Android boot backup.
The historical recovery images below target an older Linux installation; do not
apply them to the current system without checking compatibility.

RAM-only Linux tests now demonstrate TMD2725 proximity changes (near/far) and
light readings (0–36 lux) during user interaction. The firmware package had
dropped the FastRPC configuration tree, and normalized config timestamps
triggered registry rebuild requests. Restoring that tree and its cached mtimes,
starting the sensor-PD listener, and retrying an initial failed attach produced
working readings. Automatic SLPI crash recovery remains disabled; secure-memory
errors are not bypassed. See [the sensor bring-up record](../kernel/sensors/README.md)
for evidence, the integration kernel and remaining validation limits.

## Historical recovery

The first real-device boot of 0.5.5 displayed repeated SLPI FastRPC probe failures at remoteproc 5c00000 with SCM memory assignment error -22. A successful image build did not establish successful phone boot.

The recovery kernel disables the beryllium SLPI remote processor in both panel device trees. This prevents that processor and its FastRPC channel from starting; motion, orientation, proximity and other SLPI-backed sensors are unavailable. This is a temporary workaround, not a complete sensor driver repair. Other boot blockers remain possible until tested on the phone.

The upstream report describes lost memory ownership tracking across SDSP restarts: https://lists.openwall.net/linux-kernel/2026/09/22/3194 . We do not bypass failed secure-memory permission checks. Restore SLPI after diagnosing the initial crash and validating a lifecycle/firmware fix.

Use the recovery boot image with the already installed 0.5.5 filesystem. Update only boot; do not erase or flash userdata. Retain the original 0.5.5 boot.img for rollback through fastboot. Hardware acceptance requires reaching the login screen and capturing full boot logs if it still stalls.

## Built recovery package

Release 0.5.7 (Tianma) was built successfully. The packaged boot image is 39,608,320 bytes, SHA-256 `10e098bfe6aeb390a1ad1c662f74c63413aa30a74d1e0f17d73f6a15fb0becea`. Offline inspection confirms `/soc@0/remoteproc@5c00000/status = "disabled"` in the appended DTB. The initrd and command-line bytes match the original 0.5.5 image. Installer tests cover boot-only writes, refusal conditions and transfer failure without reboot. Phone boot remains unverified until the user installs this recovery image.
