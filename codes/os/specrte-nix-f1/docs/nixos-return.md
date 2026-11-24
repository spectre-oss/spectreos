# NixOS return — 2026-11-04

User requested building a combined kernel and reinstalling NixOS after Android
hardware investigation. Installation is authorized for boot and userdata;
recovery, modem, persist/calibration and GPT must remain unchanged.

## Combined configuration

The device selects QSEECOM when Fingerprint Lab is enabled, deferred userspace
SLPI startup when sensors are enabled, and the tested OV7251/LM3644 IR wiring.
The shared kernel builder owns the IR patch, driver and device tree, so standalone
IR diagnostics and the installed kernel use the same implementation. LAB/IBB
IRQ ordering, rear-camera and Wi-Fi patches remain included.

IR illumination is off by default, with an explicit root-only software-bounded
pulse interface. It remains diagnostic; no face authentication is configured.
Fingerprint Lab is a single-template workbench, not PAM/lock-screen login.

## Installation preparation

- Device: USB serial <device-serial>, product beryllium, Tianma panel.
- Android boot completed; battery 100%, userdata approximately 1.9 GiB used.
- Current rooted Android boot saved to
  `.build/nixos-return/android-rooted-boot.img`, SHA256
  `fda5ec93039d558be63a7c616f9f15e5c2aec9e0c14bf4e0e54f87c3dfd667f0`.
- Existing stock ROM artifacts and partition backups retained.
- Nix GC reclaimed 11.8 GiB of unreachable store data; result roots retained.
- Configuration evaluation passed; 14 fingerprint backend and four flash-script
  tests passed. Combined kernel compilation started, configuration validation
  reported zero errors and warnings.

At this preparation stage, image build and hardware validation were pending.
The later installation result is recorded below. Initial build log:
`.build/nixos-return-build.log`. The full image plan includes 332 derivations
and 448.5 MiB of downloads; it includes desktop dependencies, not just a kernel.

## Combined kernel RAM test

Built kernel `/nix/store/cml4d4zj3b3h1ggc4dz8fnpl8g99l1qm-linux-aarch64-unknown-linux-gnu-7.1.0-rc1`,
SHA256 `c1a9176d577e37fbfea496559633faf8364b2450651df6845ce4ed8120e90fe1`.
RAM image `.build/nixos-return/combined-ram-final/boot.img`, SHA256
`910a4d9a582513121d8dd2ee890f8f39dcf449ae0f1ae42f91be8968cdcde919`.

The combined kernel captured five pattern and 90 real IR frames, rejected invalid
pulse lengths, and left PWM disabled/HWEN low. Both camera probes succeeded; no
Oops was captured. QCOM_QSEECOM and TEE_QSEECOM are enabled, with tee0/teepriv0
present. SLPI started and its FastRPC listener stayed running. This RAM image
did not include the SSC CLI, so no new light/proximity samples were collected.
Other remote processors were not started by this minimal diagnostic.
Evidence: `.build/nixos-return/combined-logs/`. Returned to normal Android
while the desktop image continues building. No partition writes yet.

The early firmware set now includes QCA `crbtfw21.tlv`/`crnv21.bin` and
`wireless-regdb` so built-in radio probes need not wait for stage 2. Full build
was restarted to include this correction, reusing completed dependencies;
current log is `.build/nixos-return-build-radio.log`. Android normal boot and
Magisk UID 0 were verified after the combined RAM test.

Nine orphaned Nix temporary build directories were removed after confirming
their owner PIDs and process working directories were inactive. Published
outputs, store roots, phone backups and research evidence were retained. This
left approximately 29 GiB free during the compiler dependency build. Additional
release-installer (5), USB SSH (3) and phone-debug (6) tests passed.

## Full boot image

`result-boot-tianma` built and passed image inspection: 47,607,808 bytes,
SHA256 `15bced456c40de3aff412291f68a97210cfc24f490948d33d1b56e291b818486`.
The XZ initrd archive contains both requested Bluetooth files plus regulatory.db
and its signature. Kernel config confirms QCOM_QSEECOM, TEE_QSEECOM, VIDEO_OV7251
and PWM built in. The complete root filesystem is still building.

Nix store optimization completed, freeing 11.9 GiB by hard-linking 279,753
identical files. Log: `.build/nixos-return-optimise.log`. This is additional to
the unreachable-store GC and orphaned temporary-build cleanup above.

## Camera system-package build correction

The full-system camera derivation failed at Qt CMake configuration. The host
CMake path incorrectly used Qt's dev output; this pinned Qt stores it in the
main output. After correcting that, the cross Qt runtime was also found to lack
Qt Quick. The system camera package now uses the complete pinned native ARM Qt
runtime, as the already tested live-camera recipe does, with the host Qt tools
and cross compiler. A focused rebuild passed and produced
`/nix/store/pz71p7ckaf6fz81f8i1lfx0625y0j9mq-spectre-camera-aarch64-unknown-linux-gnu-0.4.4`.
Log: `.build/nixos-return-camera-fix3.log`. The running keep-going system build
uses the earlier evaluation; rerun it after independent packages finish to
incorporate this corrected derivation.

WebKitGTK 4.1 completed its 9,465 build steps and linked successfully. During
the heaviest compilation, a temporary user-owned scheduler guard paused Ninja
below 2 GiB available RAM and resumed it above 5 GiB; active compiler jobs
continued to finish. The guard exited after that Ninja process ended. No host
swap or system settings were changed. Evidence: `.build/nixos-return/memory-guard.log`.


## Full image installation

The corrected full build completed successfully. Final build log:
`.build/nixos-return-build-final.log`. Image inspection and the filesystem
consistency check passed. Release directory:
`../dist/0.12.1/tianma/20261104T192536193142Z-685a80ddec41-system`.

- Boot: 47,607,808 bytes, SHA256
  `15bced456c40de3aff412291f68a97210cfc24f490948d33d1b56e291b818486`.
- Root filesystem: 9,494,351,872 bytes, label `NIXOS_SYSTEM`, SHA256
  `2d65643b9985dd576eea09c22744acdffdd2eaf68e8dd7b07e2160188d032106`.

ADB rebooted the phone to Fastboot. Product and unlock/partition-size checks
passed; all eleven userdata transfer chunks and the boot write returned OKAY.
The installer rebooted the phone. Recovery, modem, persist and GPT were not
written. Flash log: `.build/nixos-return-flash.log`.

USB RNDIS enumerated after boot, but initial SSH/ARP attempts did not receive
replies. The user confirmed the lock screen/desktop appeared and reports the desktop
works. SSH-based hardware/service validation remains pending: USB networking
still does not answer ARP. The host alias `poco-f1` also points to the previous
installation’s Tailscale address, not an established route to this fresh system.
Requested the current Wi-Fi address to obtain authenticated diagnostic access.
The initrd performs a first-boot ext4 check and resize before mounting root.

Final post-build Nix store optimization reclaimed another 9.4 GiB through
hard-linking 65,072 files; approximately 23 GiB remains free on the build host.
Log: `.build/nixos-return-optimise-final.log`.

## Network and launcher after installation

On the installed system USB SSH initially failed because NetworkManager marked
`usb0` unmanaged via a udev rule. Wi-Fi connected and SSH was reached at
<phone-lan-ip>. A NetworkManager device override in
`/etc/NetworkManager/conf.d/90-spectre-usb.conf` sets `managed=1` for usb0;
this override was also added to `modules/usb-ssh.nix` for future builds. The
existing static USB profile then activated at 172.16.42.1. The build host's public
SSH key was installed in the phone user's authorized keys and its `poco-f1`
alias was updated; key-based `ssh poco-f1` and automatic USB reconnection
were verified after reboot. The Wi-Fi IP is DHCP-assigned and may change.

Settings and Console were present in the image, but Phosh's `app-filter-mode`
was set to `adaptive`, hiding apps without that metadata. The user setting was
changed to an empty filter and persisted through reboot; the default is also
specified in `modules/keyboard.nix` for future builds. Launcher appearance
still awaits visual confirmation.

The reboot displayed a message that the IIO sensor proxy was stopping, and the
phone eventually rebooted without intervention. Previous-boot logs record the
start of the proxy shutdown but no completion before the log ended; the cause
and any delay remain unverified. Sensor processor and fingerprint services ran
on the first boot; this is not a functional enrollment or proximity test.

## Files and launcher repair — 2026-11-05

Nautilus 50.2.2 was already installed through `configuration.nix`. A native
`nix-build <nixpkgs> -A nautilus --no-out-link` on the phone returned the same
existing store path; no compilation was needed. The user confirmed Files opens.

The missing launcher entries had a second cause beyond the adaptive filter:
`mobi.phosh.Shell.service` supplied a PATH containing only core utilities.
GLib could not resolve the executable named by several desktop files and did
not load them. A persistent user service drop-in sets PATH to
`/run/current-system/sw/bin:/run/wrappers/bin`. The same setting is in
`modules/phosh-console.nix` for future builds. After restarting Phosh, the user
confirmed Files, Settings and Console are visible. The user-level drop-in
applies to the current installation; the project change has not been rebuilt
into the installed system generation.

A source snapshot was copied to `~/Projects/spectre-nix-f1` on the phone. It
excludes Git history, build outputs, local sync state and some upstream source
checkouts. Its `configuration.nix` and `modules/phosh-console.nix` were checked
against the build-host copies. This is a synchronized snapshot, not automatic
bidirectional synchronization; check both sides for edits before updating it.
