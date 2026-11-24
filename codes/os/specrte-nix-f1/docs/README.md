# Spectre Nix documentation

> Latest documentation review (2026-10-21): [status and evidence map](system-project-status.md). Dated observations below retain their original scope.

This directory is synchronized both ways with
`<build-host>:<repo-root>/docs/`
on every project sync. Locally it lives at `docs/spectre-nix/`, and the source
tree's `codes/spectre-nix/docs` symlink points here so project-relative paths
continue to work. The server retains its existing project layout.

## System and operation

- [System overview](system.md)
- [USB inspection and VT fix](inspection-2026-10-20.md)
- [Access and maintenance](operations.md)
- [Requirements](requirements.md)
- [Hardware](hardware.md)
- [Phosh](phosh.md)
- [USB SSH](usb-ssh.md)
- [Phone diagnostics](phone-debug.md)
- [Boot recovery](boot-recovery.md)
- [Early power-button shutdown diagnosis and guard](reboot-2026-10-21.md)
- [U-Boot](uboot.md)
- [U-Boot goal](uboot-goal.md)
- [Audio](audio.md)
- [Camera bring-up and live package](camera.md)
- [Camera colour diagnosis and vendor tuning](camera-colour.md)
- [Camera noise reduction](camera-noise.md): measured noise model, RAW filtering, motion-gated bursts, preview/still chroma filtering and phone measurements.
- [Camera pipeline implementation checkpoint](camera-pipeline-progress.md): measured defect correction, module EEPROM, Sunny actuator fix, shading reconstruction and remaining integration work.
- [Original Xiaomi camera reverse engineering](camera-xiaomi-research.md): APK/REA evidence, digital gain, EEPROM-confirmed Sunny module and native CC13 matrix extraction.
- [Original Xiaomi camera pipeline](camera-pipeline.md): recovered preview, snapshot, MFNR, HDR and night paths, with [all recovered pipeline names](camera-pipeline-inventory.md).
- [Xiaomi HDR compatibility helper](camera-xiaomi-compat.md): original ARM32 HDR engine running on NixOS through a camera-specific helper, ABI evidence, build and native tests.
- [Fonts](fonts.md)
- [Nix search path](nix-search-path.md)
- [Command-not-found integration](command-not-found.md)

## Build tooling

- [Build supervisor](build-supervisor.md)
- [Build alerts](build-alert.md)
- [Build task notes](grok-build-task.md)

All builds run on the Tailscale build server. The source project's
`infra/README.md` explains two-way sync, remote build commands and explicit
artifact download. No sync command installs or flashes the phone.

General history, the server file catalog and imported legacy documents are
maintained separately in the outer workspace's `docs/general/` directory.

- [Spectre Camera (Qt)](qt-camera.md): touch interface, RAW pipeline, builds, tests and installation.
