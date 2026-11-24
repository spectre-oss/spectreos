# REA investigation, 2026-11-03

Used existing REA 6.7.0 CLI and Ghidra 12.1.4 locally. No REA client
configuration or firmware was modified to perform analysis. Set
`GHIDRA_INSTALL_DIR=<camera-research-dir>/tools/ghidra_12.1.4_PUBLIC`.
Saved complete JSON evidence under ignored `.build/fingerprint/`.

## Findings and limits

- Keymaster command 0x205 obtains a recipient via `get_auth_ta_name`, obtains
  its authentication key, then calls `qsee_encapsulate_inter_app_message`.
  REA/Ghidra entry `0x115c30` corresponds to ELF VA `0x15c30`.
  Evidence `ev_ee43ed984451eb86c00a62d30a023065436b843e3cb8f085d3855c4b6c79cd25`
  (`keymaster-rea-getauth.json`).
- Recipient helper queries `fpta_name` for auth type 2 and falls back to
  `fingerprint` if configuration retrieval fails. The live returned property
  remains unknown. Evidence
  `ev_bba0fb8f76b3209ab66a72d9bdd97ad131fa1d28df3bf47e493fca94a0a9dee3`
  (`keymaster-rea-recipient.json`).
- Secure-library encapsulation opens singleton service 21. Missing service,
  several service errors, and unmapped errors collapse to `0xff000fff`.
  This is not a unique diagnostic of a recipient-name mismatch. Evidence
  `ev_1cecd7f80ba09c94f6a22c23e8668b2c194b3f72d81cc764f21fa78dadbe089c`
  and `ev_b804b13ddf5cc68624a95646d5de2d59b656e75b6a860d1e18649b2ea283c79f`
  (`cmnlib-rea-encapsulate.json`, `cmnlib-rea-singleton.json`).
- Stock libQSEEComAPI start-app/handle path does not rewrite the application
  name in the inspected implementation. Its send-command wrapper pads an
  externally supplied request to 64 bytes and uses an adjacent reply buffer
  on QSEE version 0x14. Evidence
  `ev_a09cdc5b5127728f4ee17c14850600c1211bea4c1632b9ce3022ac115fb32abf`
  and `ev_9aefdfbea965a1a2b37856f651d8703de4d900645c02205721f8bf6b246a554a`
  (`qseeapi-rea-handle-exact.json`, `qseeapi-rea-send.json`).
  Credential-free live tests of 8/2048, 64/2048 and 64/64 request-size/reply-offset
  pairs all returned transport 0 and Keymaster -16773121 on R2.

These are static decompiler observations corroborated against disassembly;
they do not identify which secure branch executed. REA rebases these ELF
images by 0x100000. Supplying an unmapped address can resolve it as a file
offset: `keymaster-rea-15c30.json` selected the wrong function and is NOT
valid evidence for the authentication-key handler. Use exact mapped addresses.

## Other concrete comparisons

- gfber's signed ELF `TA_APP_NAME` at VA0xc100c points to `gfber` at0xc1000;
  metadata at0xc1038 is `m=x;n=gfber;p=b:9ba609ff2d7001,82:2,400`.
- MIUI also ships `fingerpr.*`, a distinct AArch64 Qualcomm fingerprint stack.
  Its TA name is `fingerprint`, with QFP sensor/capture/calibration code.
  It is not established as a Goodix gateway and was not loaded as a workaround.
- Phone keymaster/devcfg/cmnlib64/tz A and B partitions have identical hashes.
  Alternate firmware copies therefore do not explain the inspected mismatch.
- Stock SDM845 DTS marks no crypto-clock support and cmnlib64 loaded by UEFI.
  Stock skips APP_REGION_NOTIFICATION when `androidboot.keymaster=1`, which
  is present on this phone. Earlier failed region-notification experiments
  should not be repeated as an assumed prerequisite.
- Upstream lookup allocates uninitialized TZ memory and copies only the name
  bytes; stock zeroes its buffer and terminates the name. Both pass a length
  excluding NUL. `qsee-name-init.patch` removes that discrepancy as an explicit
  diagnostic kernel, without claiming it fixes enrollment.
- Stock can use the whitelist send interface; our current driver uses legacy
  sends. Its relevance to this failure is unproven. No whitelist experiment
  has been performed.

Enrollment remains unresolved. Do not patch signed firmware, suppress token
validation, or report a successful start as a saved fingerprint.

## Name-buffer kernel hardware result

Temporary boot image
`503d53202a391c88cb97e1378c0032bfbc57228d5d1ab36618b4693b5d4f5282`
built and booted. All three request layouts still returned -16773121. The
initial gfber load/retry symptom also recurred in dmesg. Sensor test passed
88x108 with OTP present, Wi-Fi and physical ALSA card were present, taint512,
and no failed services. The patch is a buffer-initialization correction, not
a demonstrated enrollment fix, and remains an explicit experiment.

A separate minimal MIUI-kernel boot failed to expose USB logs (see
stock-kernel/README.md), so stock-kernel behavior is still unknown. A read-only
time listener is implemented and bounds-tested; live results follow.


## Recovered R2: time and combined listener tests (2026-11-03)

After manual fastboot recovery, temporary R2 boot restored USB SSH. Installed
boot partition hash remained 8c27a090763dc1a31ec4cd8b0419756a81d876cf40e9b5b9589f28ffebbbfdf0;
pstore was empty. No stock-kernel comparison data was obtained.

Standalone MIUI-compatible time listener 11 registered but received no callbacks.
Keymaster 0x205 still returned -16773121 for all three request layouts.
Then a single supplicant registered RPMB 8192 (28672 bytes), FS 10 (20480),
GPFS 28672 (516096), and time 11 (20480) together. FS state was an empty
/run/qsee-key-test tmpfs directory; RPMB writes were explicitly rejected.
All three layouts again returned -16773121, with no listener callback logs.
This does not establish parity with Android's existing filesystem state, nor
prove that all secure-world prerequisites are satisfied.

Both transient services were stopped. Restored spectre-fingerprint service
and fingerprint-lab GUI. Sensor test returned vendor status zero, 88x108,
32 OTP bytes present (contents not logged). Enrollment remains unsupported.
Wi-Fi was up, Xiaomi Poco F1 ALSA card present, taint 512, no failed system
services. No flashing, erasing, or kexec was used.


## Continued isolation: dispatcher and recipient matrix

The key-only probe now reports only public envelope offset/length metadata
and tests authentication types 2 (fingerprint), 1 (gatekeeper), and 4 (iris),
with the same three buffer layouts. R2 returned -16773121 for all nine cases.
Both header fields were zero. This is NOT evidence that encapsulation was
never reached: the dispatcher clears the response on handler failure and
writes back only its status (raw keymaster text offsets 0x998–0x9a4, 0x9dc).
No key/envelope contents were logged or saved.

REA getDerivedKey evidence
`ev_c0e1e1bbef97ef5857ec03f8766bcecc89d62fb52187d93ed53c2aa56ba02d35`
shows type4 returns only 0, -29, or -49; it caches a 32-byte random value.
The encapsulation wrapper can return 0xff000fff for missing service21,
unmapped errors, and mapped errors13/14/22. Numeric status cannot distinguish
these causes. Static code has not established the identity of the resident
running binary or the live recipient property.

`whitelist.nix` builds a separate diagnostic with an off-by-default root-only
`qcom_scm.qseecom_diag_whitelist` parameter. When enabled, only pointer-free
0x205 requests with a 256-byte reply use stock secure command6, seven arguments,
and a zeroed four-entry (32-byte) whitelist table. All other traffic retains
the original transport. This is a comparison experiment, not a fix or a full
whitelist implementation; it does not support Goodix external-buffer sends.

The invalid authentication type0 control returns -38 on all layouts, matching
the handler's invalid-input path. Static GET_VERSION handler0xb1c copies
{3,0,3,132} from file offset0x2af90; this matches live API3.0/TA3.132 but is
not a hash comparison of the resident executable. Focused inspection of tz.bin
found no service21 anchor; error13/14/22 semantics remain unknown.


## Whitelist hardware result

Temporary image SHA256
`1a0cfe240851a5f7ddbc6e0ab30524772facf8ef78e593995bfa9cd461ea6edf`
booted successfully. Kernel derivation output:
`k1x5qgqfb04kqc9f6dj9labfyiyslr95-linux-aarch64-unknown-linux-gnu-7.1.0-rc1`.

Performed N/Y/N runtime transport comparisons, first without listeners and then
with RPMB/FS/GPFS/time registered together. Each sequence completed36 requests:
27 valid-type requests returned transport0/status-16773121; nine invalid-type0
controls returned transport0/status-38. No callbacks were logged for the
combined service. Thus the newer send interface is accepted, but does not fix
this failure under the tested conditions. The switch was reset to N and the
combined service stopped. This patch stays an explicit negative experiment,
not part of the normal kernel or an enrollment fix.

Taint remained512; installed boot hash stayed
`8c27a090763dc1a31ec4cd8b0419756a81d876cf40e9b5b9589f28ffebbbfdf0`.
Original R2 temporary image was selected again afterward.

The stock-kernel baseline remains missing. Both MIUI and TWRP artifacts contain
matching Beryllium DTBs and support gzip initramfs/QSEECOM/ION/NCM; no concrete
packaging defect explaining the failed MIUI boot was found. No additional
stock/TWRP boot was attempted in this comparison. If that branch is resumed,
a syscall-only init that immediately reboots to bootloader would isolate
whether early userspace is reached before attempting a full diagnostic again.


Post-comparison R2 restoration verified: sensor status0, 88x108 with OTP present;
GUI running; Wi-Fi up; physical ALSA card present; no failed system services;
no Oops/panic in current dmesg; taint512 and unchanged installed boot hash.
Prepared syscall-only TWRP-kernel sentinel image (see stock-kernel/README.md),
not booted pending user's availability for physical recovery if it hangs.


## Stock-derived boot follow-up with user present

The TWRP-kernel syscall-only sentinel was temporarily booted; Fastboot was
observed afterward, but no execution log proves which path returned it.
The full stock-derived diagnostic and a separate syscall-supervised/musl-worker
variant produced no USB log connection. User confirmed manual Fastboot recovery.
No reliable watchdog completion or Keymaster result is established. Stock-kernel
comparison therefore remains unresolved; these observations do not isolate a
libc failure, USB configuration issue, kernel hang, or secure-world hang.
R2 restoration initiated immediately after recovery; no flash/erase performed.

Restoration completed and verified: app sensor test status0/88x108/OTP present,
GUI relaunched, Wi-Fi up, physical Poco F1 ALSA card present, no failed services,
no captured Oops/panic, taint512, empty pstore. Installed boot hash remains
8c27a090763dc1a31ec4cd8b0419756a81d876cf40e9b5b9589f28ffebbbfdf0.
