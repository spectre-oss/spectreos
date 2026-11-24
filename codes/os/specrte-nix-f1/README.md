# specrte-nix-f1

> Latest documentation review (2026-10-21): [status and evidence map](docs/system-project-status.md). Dated observations below retain their original scope.

Experimental Mobile NixOS port for Xiaomi Pocophone F1 (beryllium), with Phosh.
This is a local fork in `spectre-nix`, based on the existing checkout in
`pocof1/spectreos-nix/mobile-nixos`. The original checkout is preserved.
`mobile-nixos` is on the local `specrte-nix-f1` branch; no hosted fork was created.

## Hardware and kernel

The existing `pocof1/README.md` identifies this phone's panel as confirmed Tianma
(NT36672A), consistent with the previous build and flashing logs. Tianma is the
default. `PANEL=ebbg` selects the alternative DTB if needed.

The source archive under `kernel/` is the SDM845 mainline project kernel,
`sdm845-7.1-rc1-r0`, revision `85f1df2a4ec71d7a91dd95a7a49f889d1595ffa8`.
It was the newest SDM845 release tag returned by the upstream repository during
setup on 2026-10-17. It is a release candidate, with device-specific patches,
not a claim that all phone functionality is supported by vanilla Linux.
`kernel/default.nix` supplies it to Mobile NixOS. The monolithic configuration
starts with Mobile NixOS's SDM845 config, then merges the SDM845 fragment from
this exact kernel source (modules become built-ins). This enables the battery
fuel gauge and GPIO proxy, removes unrelated hardware, and preserves upstream's
userspace PD-mapper workaround for a Poco microphone regression in 7.1-rc1.
`kernel/beryllium.config` applies final phone-specific overrides, including
excluding the unrelated Pixel 3 STMFTS touchscreen driver.
The local builder uses `olddefconfig` to handle new options without prompts;
required Mobile NixOS options are still validated. Legacy vendor-tree FHS path
rewrites and optional perf/selftest shebang rewrites are skipped for this
mainline kernel; required kernel build scripts are still patched. Native Python is supplied for
Adreno register generation, with an early generator smoke check. The versioned
installed `vmlinuz` filename is exposed through kernel metadata for the image
builder; the compile target remains `Image.gz`.

Firmware is pinned to the latest upstream HEAD observed on 2026-10-17.
It is proprietary; do not assume redistribution permission. Kernel and firmware
archives stay compressed to save host disk space. `make fetch` restores missing
archives and verifies their SHA256 checksums. `make verify` also checks that
the firmware bundle satisfies the kernel device tree's firmware requests. Revisions are in `sources.json`;
Nixpkgs is pinned by `mobile-nixos/npins/sources.json`. If the checkout is
missing, `make fetch` restores its pinned base and applies the saved builder
patch from `patches/`.

## Build

Install Nix on the Linux build host. Normal use with a working Nix daemon:

```sh
make fetch
make eval
make kernel
make build JOBS=4
make inspect
```

The Makefile automatically uses a workspace store if `/nix/store` is absent.
You can also select that user-owned diverted store explicitly:

```sh
make eval NIX_STORE_ROOT=../nix-store
make build JOBS=4 NIX_STORE_ROOT=../nix-store
```

A diverted store requires Linux user/mount namespace support. The full system
cross-compiles from x86_64 to aarch64; cache misses can require substantial time
and build space. The kernel archive is about 251 MiB. No existing archives or
other projects are deleted. Build results are retained through logical Nix GC roots in `.build/roots/`.
The public `result-*` links resolve to the physical store so inspection and
fastboot work outside the diverted store namespace.

## Storage

The writable ext4 root goes on the existing userdata partition and expands its
filesystem to fill that partition on boot. GPT resizing is disabled: other
partitions must not be changed. Store hardlink deduplication is enabled, weekly
GC deletes generations older than 14 days, system documentation and the standalone GNOME help bundle are omitted, journal size is bounded to 64 MiB
persistent / 32 MiB runtime, and swap uses compressed RAM rather than a swapfile. Runtime firmware is limited
to the Poco bundle and Qualcomm graphics/Bluetooth files. Cross builds reuse
the pinned native emoji font and feedback-device JSON data packages from the
binary cache. The ARM feedback daemon itself is still built for the phone.

## Boot and install

`android-tools` is only needed on the host: its `fastboot` program transfers
images over USB. Connect an already-unlocked phone in fastboot mode:

```sh
make devices
# This smaller target can finish while the full desktop build continues:
make boot-image
make boot
```

`make boot` temporarily loads the kernel/initrd; it does not supply a rootfs to
an uninstalled phone. Use it as an early boot/display check. The full desktop
requires the userdata rootfs.

Installing overwrites **userdata (all Android apps and personal data)** and
**boot**. Back up your files first. Recovery, modem firmware, persist, and the
partition table are preserved. The script requires the connected product to be
`beryllium`, an unlocked bootloader, and explicit erasure acknowledgement:

```sh
make install ERASE_USERDATA=beryllium
# If multiple devices are attached, add SERIAL=<phone serial>.
```

`make install` runs `make inspect` before flashing. It checks the built boot
header, actual kernel version and embedded phone-driver configuration, appended
panel device tree, ext4 root label/size, and prints image checksums.
Rootfs is flashed first, then boot. A transfer failure stops the script. Do not
use the upstream generated `flash-critical.sh`, which also flashes recovery.
No bootloader unlock, repartitioning, or automatic DTBO erasure is performed.
If early boot needs a different DTBO state, investigate against this phone's
current firmware before changing that partition.

Initial local username: `spectre`; initial PIN: `123456`. Change it with `passwd`
after first boot. SSH is disabled. This new NixOS port needs physical boot,
display/touch, storage, Wi-Fi, audio, and modem testing before it can be called
working. See `STATUS.md` for current validation evidence.


Project versions and releases
-----------------------------

`python3 scripts/version.py` calculates the project version from first-parent
Git history, starting at `0.0.1`. Every `feat:` commit increments minor; `fix:`,
`perf:` and `revert:` increment patch. A Conventional Commit with `!` or a
`BREAKING CHANGE:` footer increments major (including before 1.0). Other types,
including `chore(release):`, leave the version unchanged. Kernel upstream versions
are separate from this project version.

Run `make hooks` in a new clone to enable the tracked post-commit hook. After a
commit that changes the calculated version, it writes `VERSION` and creates a
separate `chore(release): <version>` commit using the triggering author's identity.
A recursion guard prevents repeated release commits. If the version is unchanged,
no extra commit is needed. The hook commits only VERSION, preserving other staged
files. It never pushes. Use Conventional Commit subjects for all changes.

Every successful `make kernel`, `make boot-image`, or `make build` publishes its
artifacts in `../dist/<project-version>/<panel>/<UTC-build-id>-<git-revision>-<kind>/`.
`make build` publishes both the kernel (including device trees/config), `boot.img`,
and the root filesystem `system.img`; partial kernel/boot builds publish only
those finished artifacts. A manifest records the version, revision, sizes and
SHA256 checksums. Versions/revisions are captured when Make starts; unique build
IDs preserve repeat builds without overwriting them. The dist files are standalone
copies (reflinks when supported, sparse copies otherwise), not Nix-store symlinks.
Incomplete publications are removed. The root filesystem appears only after the
full system build succeeds. All dist artifacts stay outside the Git repository;
a repository-local dist directory is also ignored.

`make optimise-store` deduplicates identical files in the selected Nix store.
`make gc-store` removes only Nix-unreachable paths, retaining registered result
roots and active-build temporary roots. Neither command deletes a whole store or
known-good result. Old dist releases are independent files: remove specific old
release directories when no longer needed; GC deliberately does not delete them.


Installing from another laptop
------------------------------

Every dist build directory includes `install.py` and `INSTALL.md`. Copy a
complete **system** release directory to your laptop and follow INSTALL.md:
`python3 install.py --check` checks files and the connected phone without writes;
`python3 install.py` guides installation and requires a typed userdata-erasure
confirmation. Windows can use `py -3` instead of `python3`. Python 3.9+ and fastboot
are the only laptop dependencies. Kernel/boot-only releases explicitly refuse
full installation. Existing dist releases also receive these helper files;
image checksums, project versions, and original build revisions are preserved.
The phone will be flashed by the user from their laptop after copying the release.

Build parallelism defaults to `nproc` (all available CPU threads; currently 12).
Override with `JOBS=N` when needed. No project RAM cap is imposed; compiler
processes use memory as needed. Serial build steps may still use only one core.
