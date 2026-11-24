# Qualcomm shutdown ordering — 26 October 2026

The previous power-off stopped NetworkManager, wpa_supplicant and rmtfs
concurrently. rmtfs (-P) tore down the modem while Wi-Fi still used firmware
services. The recorded shutdown took roughly 60 seconds, with a modem port
halt failure followed by ath10k QMI configuration/enable timeouts.

`modules/qualcomm-shutdown.nix` orders rmtfs before NetworkManager,
wpa_supplicant and ModemManager at startup, so those clients stop first.
It orders rmtfs after pd-mapper and tqftpserv so the helpers remain available
through modem teardown. Existing QRTR ordering is retained. No timeouts are
shortened and no services are forcibly killed.

The equivalent persistent live drop-in is
`/etc/systemd/system.control/rmtfs.service.d/60-spectre-shutdown.conf`.
The usual /etc/systemd/system directory is an immutable NixOS symlink.
Systemd-analyze verify passed for all six affected units; Nix source parses.

A normal reboot at 00:04 Vienna time on 26 October completed without physical
intervention. NetworkManager and WPA stopped at 23:04:20 UTC, then rmtfs
stopped within the same second, followed by pd-mapper/tqftpserv/QRTR.
No previous 60-second QMI timeout occurred. The modem still emitted
`port failed halt`, but without a visible delay. New boot ID:
cd6ec4fe-112b-41de-b884-33d95454bf63. USB SSH, Phosh, NetworkManager and rmtfs
returned active; Wi-Fi reconnected. The screen was locked/off after checking.

This verifies one normal reboot, not repeated reboots or complete power-off.
Do not describe the remaining modem warning as repaired.
To undo the live change, remove only the named drop-in and daemon-reload;
remove the module import to undo the declarative change.

## Repeated reboot checks

Three further normal reboots passed after adding boot-service guards:

| Boot ID | Startup | Result |
| --- | --- | --- |
| a45c6818-fc5a-483f-ab29-aa768a996014 | 24.061 s | USB/Phosh/Wi-Fi returned; zero failed system units |
| dd3d5aba-20b7-4884-b75e-435581300975 | 23.907 s | USB/Phosh/Wi-Fi returned; zero failed system units |
| f038de6a-789a-4595-b94d-e5338fec9348 | 23.915 s | USB/Phosh/Wi-Fi returned; zero failed system units |

First-boot pings initially preceded Wi-Fi association; the follow-up passed.
All three boots passed three router pings once associated. No forced restart.
Full power-off remains pending physical user observation/manual power-on.

`modules/boot-reliability.nix` disables Android A/B boot-control for this
single-boot-partition device. Its live equivalent is a boot-control condition
requiring androidboot.slot_suffix; no bootloader metadata is modified.
SIM provisioning has an ExecCondition checking for a present card, with QMI
query errors still treated as failures. This prevents empty-slot provisioning
when no SIM is present. SIM-inserted provisioning remains untested.
Persistent live drop-ins are under /etc/systemd/system.control and the SIM
condition script is /var/lib/spectre-boot-checks/sim-card-present.sh.

Startup remains about 24 seconds: approximately 18 seconds before userspace,
6 seconds userspace, with graphical.target at 3.6–3.8 seconds into userspace.
No blanket timeout reduction or unrelated hardware-service disabling applied.
