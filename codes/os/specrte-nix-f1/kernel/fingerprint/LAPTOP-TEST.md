# Revised fingerprint transport kernel: USB test

The first kexec test produced a regulator Oops and several hardware failures.
This image adds an IRQ initialization fix. It is still experimental and does
not yet implement fingerprint enrollment or authentication.

Use the laptop with the phone attached by USB. Do not use kexec, flash a
partition, rebind a driver, or manually start remote processors during this
baseline test.

1. Extract the bundle into a new directory and run `sha256sum -c SHA256SUMS`.
2. Confirm that USB SSH works with the original kernel:

   ```sh
   ssh -F /dev/null -o IdentitiesOnly=yes -i ~/.ssh/<device-key> spectre@172.16.42.1
   ```

3. On the phone, run `sudo systemctl reboot --reboot-argument=bootloader`.
   If it hangs, ask the owner to restart the phone into fastboot with the
   physical buttons. Do not repeatedly attempt remote resets.
4. On the laptop, run `fastboot devices` and `fastboot getvar product`.
   Proceed only for the intended unlocked **beryllium** phone. If multiple
   devices are connected, select its serial explicitly for every command.
5. Run `fastboot boot boot.img`. This loads the test image into RAM and leaves
   the installed boot partition unchanged. If the command fails, stop and
   report the output; do not substitute a flash command.
6. Reconnect over USB SSH and copy the diagnostic script and probe to `/tmp/`
   (this phone needs legacy SCP, `scp -O`, rather than SFTP). On the phone:

   ```sh
   sudo sh /tmp/collect.sh /tmp/poco-fingerprint-r2-audit
   chmod +x /tmp/qsee-probe
   sudo /tmp/qsee-probe
   ```

   Run the probe **without arguments**: it only queries the TEE interface
   version. Do not load a trustlet or send fingerprint commands yet.
7. Report the boot-image checksum, the probe output, QSEECOM config flags,
   regulator crash/warning status, memory-assignment errors, Wi-Fi and internet
   status, ALSA cards, remoteproc states and failed services. Preserve the full
   audit directory. If hardware is healthy, leave the phone on the test kernel
   so the build agent can inspect it with `ssh poco-f1`.

If hardware fails, collect logs through USB before a normal `sudo reboot`.
If the phone freezes, ask the owner to hold power until it restarts. A normal
boot returns to the installed kernel. Do not attempt further changes merely to
make an unhealthy experimental boot appear functional.
