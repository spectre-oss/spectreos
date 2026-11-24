# Install this release from your laptop

1. Copy this entire build directory to your laptop, including `install.py`,
   `manifest.json`, `boot.img`, `system.img`, and the kernel directory. Use a
   **system** build: kernel-only and boot-only releases cannot install the OS.
2. Install Python 3.9+ and Android SDK Platform-Tools on the laptop. Ensure
   `fastboot --version` works from your terminal. Windows may also need the
   phone's fastboot USB driver; Linux needs USB permissions for your user.
3. Back up the phone's data. Its bootloader must already be unlocked. Check the
   manifest's display panel matches your Poco F1 (this phone was identified as
   Tianma). The script checks the model but cannot detect the display panel.
4. Power off the phone. Hold **Volume Down + Power** to enter FASTBOOT, then
   connect it to the laptop with USB.
5. Open a terminal in this directory and run the read-only checks:

   ```sh
   python3 install.py --check
   ```

   On Windows, use `py -3 install.py --check`; if more than one phone is
   connected, add `--serial SERIAL` to the check and install commands.
6. Run `python3 install.py` (Windows: `py -3 install.py`). The script shows five
   steps, verifies SHA256 checksums, checks the beryllium model, unlocked state,
   and partition sizes, then requires you to type `ERASE beryllium`.
   **This permanently erases Android userdata.** It writes the root filesystem
   to userdata first, writes boot only after that succeeds, and then reboots.
7. Allow several minutes for first boot and filesystem expansion. Initial local
   user/PIN: `spectre` / `123456`; change it after login.

No Nix installation, Git checkout, kernel compilation, or access to the build
machine's Nix store is needed on the laptop. These are experimental images;
actual phone boot/functionality remains unverified until tested. If a transfer
fails, the installer stops without proceeding to later writes or reboot. Keep
this complete release directory and the error output when troubleshooting.
