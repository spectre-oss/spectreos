# System overview

The following details were supplied by the owner; they have not yet been
confirmed through a live session.

## Hardware and boot

| Item | Details |
| --- | --- |
| Device | Xiaomi Poco F1 |
| Device codename | `beryllium` |
| Display panel | Tianma |
| Bootloader | Unlocked |
| Boot setup | Custom U-Boot setup; exact configuration and recovery procedure unknown |

## Software

| Component | Details |
| --- | --- |
| System | Spectre Nix |
| Base | NixOS / mobile-nixos |
| Interface | Phosh |
| On-screen keyboard | Stevia with project-specific custom changes |
| Keyboard user unit to inspect | `mobi.phosh.OSK.service` |
| Graphical service to inspect | `phosh` |

The exact versions, keyboard modifications, service definitions, and source
locations still need to be recorded. The listed service names are supplied
diagnostic targets; their existence and runtime state have not been verified.

## Network and access

| Connection | Phone | Computer / notes |
| --- | --- | --- |
| USB networking | `172.16.42.1` | `172.16.42.2/24` on the USB network interface |
| Wi-Fi | Last known `<phone-lan-ip>` | Address may change; keep the screen awake if SSH stops responding |

SSH password login is configured for `spectre`, according to the owner. Root
SSH login is disabled. The supplied initial login password and sudo password
are both `123456`. Treat these as initial setup credentials; they may become
outdated after a password change. Do not expose SSH to untrusted networks while
the initial password remains active.

## Configuration details still to capture

- Running kernel and exact Spectre Nix / NixOS release.
- Hostname, current IP addresses, and SSH host-key fingerprints.
- Partition layout, mounted filesystems, and available storage.
- Nix configuration path, source repository, revision, and any flake outputs.
- Custom Stevia source, patch set, build instructions, and deployment process.
- U-Boot configuration, boot artifacts, backup, and recovery instructions.
- Verified rebuild, rollback, and upgrade procedure for this particular device.

Do not infer a flashing or rebuild procedure from generic Poco F1 or NixOS
instructions: the custom boot setup and actual configuration must be identified
first.
