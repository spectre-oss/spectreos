> [!WARNING]
> **Experimental, and flashing erases data.** The README states that this new NixOS port still needs physical boot, display/touch, storage, Wi-Fi, audio and modem testing before it can be called working. Installing overwrites **userdata** (all Android apps and personal data) and **boot**. Back up first.

## Flashing safety checklist

These points are taken from the port's README; read it in full below before you flash.

1. Use a Xiaomi Pocophone F1 (`beryllium`) with an **already unlocked bootloader**. The install script refuses any other product and performs no bootloader unlock.
2. Try the early boot and display check first. `make boot` only temporarily loads the kernel and initrd; it supplies no root filesystem.
3. Install with an explicit acknowledgement: `make install ERASE_USERDATA=beryllium` (add `SERIAL=<phone serial>` if several devices are attached).
4. `make install` runs `make inspect` first (boot header, kernel version, panel device tree, ext4 root label and size, checksums), flashes the root filesystem, then boot, and stops on any transfer failure.
5. Recovery, modem firmware, persist and the partition table are preserved. Never use the upstream-generated `flash-critical.sh`: it also flashes recovery.
6. First login is user `spectre` with PIN `123456`. Change it with `passwd`. SSH is disabled.

For current validation evidence see [`STATUS.md`](repo:codes/os/specrte-nix-f1/STATUS.md).

## README
