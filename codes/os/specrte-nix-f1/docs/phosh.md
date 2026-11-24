# Phosh startup and active terminal

## Reboot validation (2026-10-20)

The installed phone's persistent drop-in at
`/etc/systemd/system.control/phosh.service.d/10-spectre-vt.conf` executes
`ExecStartPre=+/run/current-system/sw/bin/chvt 1`. A full system reboot was
tested: the boot ID changed, tty1 became active, Phosh started with zero
restarts, and USB diagnostic SSH returned. See the
[inspection report](inspection-2026-10-20.md) for evidence and the separate
computer USB address/profile correction. No new image was flashed or built.

## Original diagnosis

On the first phone test, the mobile boot process left tty2 active, while
phosh.service used tty1. Phoc repeatedly timed out waiting for logind to
activate its seat, then exited with "Could not create backend".

Switching to tty1 allowed the compositor and Phosh lock screen to start.
The service now executes chvt 1 as root before starting the unprivileged
session. Full images include this in their service definition; boot-only
images install a runtime drop-in during stage 1 for existing root filesystems.

The live fix was tested by switching back to tty2 and restarting Phosh.
A camera capture confirmed the lock screen. This fixes session startup;
application warnings about GSettings and other hardware features need
separate investigation if they affect use.

## Terminal on-screen keyboard

The original GNOME environment forced `GTK_IM_MODULE=ibus`. The terminal
therefore never activated Phosh's Wayland input method when touched. Launching
Console with `GTK_IM_MODULE=wayland` restored automatic activation and the
terminal keyboard layout, verified by touch followed by a native screenshot.
Future system builds force this environment setting. For the existing image,
`scripts/fix-terminal-input.py` installs a user-local Console desktop entry
with the Wayland variable and disables D-Bus activation for that launcher,
so the explicit environment reaches the app. It refuses to overwrite unrelated
custom desktop entries. Close existing Console processes before testing a new
launch; already running apps retain their old environment.

## Wi-Fi password keyboard

The Wi-Fi authentication dialog runs inside Phosh itself. Fixing the Console
launcher does not change the environment of that already running shell. On
19 October 2026, the password field had focus but Phosh still used
`GTK_IM_MODULE=ibus`, so no keyboard appeared.

The system configuration now adds `GTK_IM_MODULE=wayland` directly to
`mobi.phosh.Shell.service` as a user-service drop-in. The existing phone has the
same persistent setting in
`~/.config/systemd/user/mobi.phosh.Shell.service.d/20-wayland-input.conf`.
After restarting the session, reopening Wi-Fi authentication automatically
displayed the password keyboard, confirmed by a native screenshot and OSK
status without forcing keyboard visibility. This source change still requires
a new system image before it is included in downloadable releases.

## Poco F1 notch coverage

The phone Phosh build applies `patches/phosh-fixed-physical-top-bar.patch`.
The top status bar has an opaque black background and reserves 90 physical
panel pixels independently of the UI scale. Layer-shell requires integer
logical coordinates, so the logical height is rounded upward: 45 at 2×,
30 at 3× (90 physical pixels). The reserved application area, panel folding,
and cutout layout use the same height and update when display scaling changes.
This is part of the system/Phosh package; a boot-only image update does not
replace the installed shell.
