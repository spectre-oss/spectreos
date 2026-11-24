# Spectre Nix F1 recovery boot update

For a Poco F1 with the Tianma/EBBG panel named in manifest.json and Spectre Nix 0.5.5 already installed. This package contains only boot.img. It preserves userdata. SLPI-backed sensors are temporarily disabled to avoid the observed FastRPC failure loop; successful phone boot still needs testing.

1. Extract the entire package on your laptop. Install Python 3.9+ and Android SDK Platform-Tools (`fastboot`).
2. Hold Power and Volume Down to enter FASTBOOT, then connect USB.
3. Run `python3 install.py --check` to verify files, device and boot partition size.
4. Run `python3 install.py` and type the exact phrase `UPDATE beryllium`.
5. Wait for reboot. If boot still fails, capture the new screen and, where available, full kernel logs.

Do not run the old full-system installer: it erases userdata. This script never unlocks the bootloader or writes userdata. A wrong-panel release must not be installed.

To roll back, enter FASTBOOT and use `fastboot -s YOUR_SERIAL flash boot /path/to/original-0.5.5/boot.img`, then `fastboot -s YOUR_SERIAL reboot`.

## USB access after boot

This release adds password SSH over USB for diagnostics. Follow **USB-SSH.md** to set the laptop address to 172.16.42.2/24, then connect with `ssh spectre@172.16.42.1`. Use the existing spectre password.
