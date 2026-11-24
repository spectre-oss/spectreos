# Validation status — 2026-10-17

> Latest documentation review (2026-10-21): [status and evidence map](docs/system-project-status.md). Dated observations below retain their original scope.

- Project prepared at `spectre-nix/specrte-nix-f1`; original checkout untouched.
- Latest SDM845 tag checked through upstream Git refs: `sdm845-7.1-rc1-r0`.
- Kernel and Poco firmware archives downloaded and SHA256 verification passed.
- Firmware updated to upstream HEAD `d396d32118803235a58aab0d0bc3643256634a05`,
  whose vendor-qualified paths match the Linux 7.1 Poco F1 device tree.
  `make verify` checks all eight requested board firmware paths against the
  archive, in addition to the source checksums.
- Tianma panel selected from the existing device README (line 22) and logs.
- Full Mobile NixOS image evaluation passed, including the firmware refinement.
  The current image build also disables installed documentation to save space.
- `make test` passed (4 tests, including refusal cases and transfer failure);
  Bash syntax checks passed for fetch and flash scripts.
- Mock flashing checks passed: wrong product, missing erasure acknowledgement,
  and locked bootloader refuse writes; rootfs transfer failure stops before
  boot/reboot; successful flow writes userdata then boot and preserves recovery.
- The initial Mobile NixOS-based kernel reached compilation, but source review
  found that the current upstream SDM845 fragment also requires the PMI8998 fuel
  gauge, shared GPIO proxy, and a userspace PD-mapper workaround for Poco audio.
  That initial build was superseded before installation.
- The final kernel now merges the SDM845 fragment shipped in the pinned kernel
  archive, converting modules to built-ins and preserving the intended version.
  The fragment also removes unrelated hardware to reduce size. The userspace
  PD mapper is restored, following the upstream 7.1-rc1 microphone workaround.
- The firmware package built successfully (90 MiB). All eight requested board
  firmware paths, Adreno A630 files, WCN3990 Wi-Fi board file, and Bluetooth
  directory were verified in the actual output; it has no Nix store references.
- The ARM system now reuses the cached Noto Color Emoji 2.051 font package
  (CPU-independent font data), avoiding a redundant bitmap/font build. The
  native cached font was fetched successfully.
- The hardware-correct kernel build failed when Adreno header generation could
  not find Python. The kernel recipe now supplies native Python explicitly and
  runs the Adreno A6xx generator before compilation. A direct generator smoke
  check succeeded; the corrected Nix build has generated 27 Adreno/display
  headers successfully and progressed to compiling the graphics driver. The original failure log
  is retained in `.build/kernel-failure-missing-python.log`.
- A full-image cross build failed because feedbackd device-theme validation
  could not find its build-host tool. The package contains only JSON data;
  the same native 0.8.8 package was fetched, all JSON parsed successfully,
  the Poco F1 profile exists, and the output has no Nix store references.
  The image now uses that architecture-independent data package.
- Standalone GNOME help is explicitly excluded from configured system packages,
  supplementing the general system documentation setting.
- A later kernel compile failed in the Pixel 3 STMFTS touchscreen driver,
  enabled by the shared SDM845 fragment. `kernel/beryllium.config` now excludes
  that unrelated driver; Poco Novatek/FocalTech support remains selected.
  The failure log is `.build/kernel-failure-unused-stmfts.log`.
- Kernel compilation uses Make's keep-going mode to collect independent errors
  in one pass; any compiler failure still fails the derivation.
- The early boot initrd built successfully (about 23 MiB compressed). XZ
  integrity and CPIO readability passed. Its init entry point and early Poco
  firmware files exist; the bulk modem firmware is excluded as intended.
  The embedded boot config has `NIXOS_SYSTEM` as its root label and contains no
  host workspace paths. This component alone is not a fastboot boot image.
  Evidence is in `.build/initrd-inspection.json` and `.build/initrd-contents.txt`.
- The corrected Linux 7.1.0-rc1 kernel built successfully. Its installed filename
  is `vmlinuz-7.1.0-rc1`; passthru metadata now directs Mobile NixOS to that file
  without changing/rebuilding the compiled kernel derivation.
- The Tianma Android boot image built successfully (39,555,072 bytes, about
  38 MiB). `make inspect-boot` passed, including the actual AArch64 Image header,
  embedded kernel config/phone drivers, version, and appended Tianma DTB.
  Evidence and the SHA256 are in `.build/boot-inspection.json`.
- The full system/rootfs build remains active. That in-flight graph predates
  the installed-kernel filename hint and may fail at its old boot-image node;
  allow its current toolchain dependency build to reach terminal before retrying
  the current Makefile, to preserve expensive completed build outputs.
- Diverted-store builds now keep logical GC roots under `.build/roots/` and
  expose physical `result-*` links readable by host tools, including fastboot.
  The firmware link and its retained GC root were verified. Current builds use
  the corrected result-link recipes.
- `make install` runs `make inspect` first. Inspection validates the actual
  embedded kernel config for phone drivers and the PD-mapper workaround, plus
  boot layout, kernel version, panel DTB, ext4 label/size, and image checksums.
  Actual artifact inspection is pending completion of the build.
- No phone was visible in `fastboot devices` during setup. No real partition
  writes, temporary boot, or hardware validation have been performed.

Build logs and evaluation outputs are under `.build/`. A successful expression
evaluation is not evidence that the kernel builds or that the phone boots.


Release workflow (project version 0.1.0 onward)
-----------------------------------------------

- Tracked post-commit hook installed locally; Python derives SemVer from
  Conventional Commits and commits VERSION when its value changes.
- Successful Make builds publish standalone artifacts plus checksum manifests
  under ../dist/<version>/<panel>/<build-id>/; kernel and boot releases were
  published and their copied contents/checksums verified. The copied Tianma
  boot image also passed the artifact inspector.
- Full system publication remains pending the active root filesystem build.
  Boot/system Make targets validate their outputs before publishing to dist.
- Nix GC removed 822 unreachable paths, freeing 6.7 GiB, while retaining active
  build roots and kernel/boot/firmware/initrd result roots. About 147 GiB remained
  free immediately afterward. Store file deduplication was also started.
- Seven tests passed across flash guards, version calculation/hook integration,
  and atomic release publication (including repeat builds and failed copies).


Qt build correction
-------------------

The full image build stopped at native qtsvg with two incompatible QtBase
store paths in its environment. The pinned Qt5 cross/bootstrap scope disables
translations, while its spliced native qmake hook propagated a different QtBase.
The project now consistently disables Qt5 translations in the scoped QtBase and
Qt module set, preserving Qt's dependency checks. A real native qtsvg build then
completed successfully; the full image build must still complete independently.
Go completed before this failure despite its bootstrap runtime/cgo messages.
The store optimisation also completed: 1.3 GiB saved by hard-linking 89,068 files,
in addition to the 6.7 GiB recovered by garbage collection.
