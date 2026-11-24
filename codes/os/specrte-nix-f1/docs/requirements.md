# Poco F1 requirements and release readiness

This document collects the requirements discussed for this project. A setting
being present does not prove that it works on the physical phone.

## Operating system and first boot

- [x] Configure Mobile NixOS with Linux userspace and the Phosh phone interface.
- [x] Configure the Poco F1 kernel, device tree, and firmware. This phone was
  identified from existing records as using the Tianma display panel.
- [x] Enable automatic expansion of the ext4 root filesystem inside the existing
  userdata partition (`fileSystems."/".autoResize = true`).
- [x] Disable partition-table growth (`boot.growPartition = false`). Filesystem
  expansion must use the existing partition, without repartitioning the phone.
- [x] Keep resize logic in the Mobile NixOS early-boot configuration. Phosh does
  not require modifications for filesystem expansion.
- [ ] Verify first boot and filesystem expansion on a real Poco F1: check root
  capacity, successful startup of Phosh, and a subsequent reboot.
- [ ] Verify display, touch, storage, charging, Wi-Fi, Bluetooth, audio,
  suspend/resume, and modem/call functionality. Record limitations explicitly.

## Cutting-edge kernel updates

- [ ] Keep the project tracking the latest upstream Linux release. The goal is
  a cutting-edge OS, including the newest release candidate when it is newer
  than the latest stable release; identify prereleases explicitly.
- [ ] Check upstream release information at [kernel.org](https://www.kernel.org/)
  and the [Linux release archive](https://www.kernel.org/pub/linux/kernel/)
  during kernel updates. A pinned version must never be described as latest
  without a dated upstream check.
- [ ] Update the SDM845/Poco F1 patch stack alongside upstream Linux; rebase
  remaining device patches rather than silently staying on an old kernel.
- [ ] Update source pins, source checksums, kernel configuration, firmware needs,
  device trees, and build metadata together. Each published build remains pinned
  and reproducible even while the development branch follows new releases.
- [ ] Build and validate each kernel update, then run the documented phone
  hardware/regression checks, including first boot and filesystem expansion.
- [ ] If the newest release breaks the port, report the incompatibility, retain
  a clearly labelled previous known-good image, and track the fix needed to move
  forward. Do not silently call the older kernel current or the new one tested.
- [ ] Record both the project semantic version and the upstream kernel version
  in release evidence. Do not equate the two version numbers.

This is a required maintenance policy, not an already implemented automatic
updater or scheduled service. Recording it does not itself upgrade the current
kernel. See [hardware and support evidence](hardware.md) for the current pin and
subsystems that need regression testing.

## Reproducible builds and production readiness

- [x] Pin upstream Mobile NixOS, kernel, firmware, and Nixpkgs inputs; verify
  downloaded kernel and firmware archive checksums.
- [x] Commit package fixes so other builders inherit them: kernel preparation,
  Qt dependency consistency, systemd BPF headers, GUPnP documentation tools,
  and FFmpeg CUDA configuration for this Qualcomm device.
- [ ] Complete and validate the entire system/root filesystem image.
- [ ] Successfully build from a fresh checkout using the documented fetch/build
  procedure, with no dependency on untracked local changes or existing outputs.
- [ ] Make memory-heavy package handling repeatable in the build workflow.
  The default uses all available CPU threads, but SpiderMonkey exhausted this
  machine's RAM and swap with 12 jobs. It was built separately with four jobs,
  then the full build resumed with 12. This manual workaround is not yet encoded.
- [ ] Document tested build-host platforms, Nix/tool prerequisites, RAM/swap and
  disk requirements, and resource settings.
- [ ] Establish repeatable release validation before calling the project
  production-ready. A successful compilation alone is insufficient.

## Versioned artifacts and installation from a laptop

- [x] Calculate project semantic versions from Conventional Commit messages
  using Python; update and commit VERSION through the post-commit hook.
- [x] Publish successful builds to versioned directories under spectre-nix/dist,
  recording the build revision, artifact sizes, and SHA256 checksums.
- [x] Include standalone laptop installer and step-by-step INSTALL.md in every
  release directory. Partial kernel/boot releases refuse full OS installation.
- [ ] Produce a complete system release containing the kernel, device trees,
  boot.img, system.img, manifest, installer, and instructions together.
- [x] Support laptop installation using Python and fastboot, without requiring
  Nix or access to the build machine's store.
- [x] Check image checksums, Poco F1 identity, bootloader unlock state, partition
  capacities, and explicit userdata-erasure confirmation before writing.
- [x] Write userdata first, then boot, stopping on failure before later steps.
- [ ] Verify the installer on an actual connected phone. Automated tests use a
  fake fastboot and do not prove hardware compatibility.
- The user will copy the completed release to their laptop and flash the phone
  there. No automatic flashing from the build machine is required.

## Storage and source control

- [x] Ignore firmware blobs, downloaded archives, generated images, Nix stores,
  build results, logs, caches, and release artifacts in Git.
- [x] Keep source changes committed locally; do not push or create remote repos
  or pull requests unless the user later authorizes it.
- [x] Provide Nix store optimisation and garbage-collection targets. Preserve
  registered results and active builds; never delete whole stores blindly.
- [x] Configure phone store deduplication/garbage collection, bounded journals,
  and compressed RAM swap.
- [ ] Measure the final image and runtime storage usage on the phone.

## Build monitoring

Check long-running builds every ten minutes. If a build fails, inspect the real
failure, fix it within the requested scope, commit source changes, and resume
using completed cached packages. Keep routine progress messages quiet; report
failures, completion, or a decision needed from the user. An active build must
not be restarted merely because an observation timed out.
