# Current result — 2026-11-04

The trusted-authorization blocker has been resolved in a Linux diagnostic.
Working Android loads **gfdip**, not the **gfber** used by earlier Linux tests.
The running stock kernel log identifies gfdip; the captured stock libgf_ca.so
also contains `gfdip` at file offset 0xd2b. Earlier analysis concentrated on
gfber despite this vendor-client name. Matching geometry/OTP was insufficient
to identify the correct authentication trustlet.

Measured results:

- Rooted Android: resident gfdip lookup succeeds (app 7), Keymaster 0x205/type 2
  returns status 0. No envelope bytes are printed or persisted.
- Temporary Linux: the same signed gfdip firmware yields status 0 for all
  three tested fingerprint-key request layouts (public envelope length 152).
  Types 1/4 remain unsuccessful; invalid type 0 returns -38.
- Goodix class 2 command 109 imports that envelope with transport/vendor 0.
- Sensor detect/init returns vendor 0, 88×108 geometry and 32 OTP bytes present.
- With the bounded RPMB listener, isolated Gatekeeper UID 0x53504650 receives
  a 58-byte handle and a challenge-bound signed token. This test updates that
  diagnostic UID's secure credential record; Android UID 0 is not used.
- Goodix ENROLL 1007 and CANCEL 1009 return vendor 0.
- The capture-only run processed twelve progress reports down to zero and
  cancelled without saving (`capture-r1-logs/capture.txt`).
- The subsequent RAM-only save run processed progress down to zero, sent
  SAVE 1012 successfully, and enumerated exactly one template in group 1000.
- AUTHENTICATE 1010 followed by IRQ processing 1016 reported one successful
  match with the expected group and saved template identifier.
- The different-finger request was answered after the helper's 100-second
  window ended. There is no verified negative control; do not infer rejection
  or false acceptance from this run.

The credential, handle, token and envelope are wiped rather than logged.
Controlled positive/negative matching subsequently passed (see below). A stable
Linux credential policy, persistent template storage and app integration remain
unfinished. The GTK app now uses this protocol in version 0.3, but the new app has not yet
been exercised on the phone. Only verified SAVE plus enumeration reports
completed enrollment.

Evidence: `.build/fingerprint/android-comparison/`, especially
`key-exchange-gfdip.txt` and `linux-r2-logs/{key-exchange,key-import,gfdip-sensor,enroll-start}.txt`.
The test uses the existing QSEECOM kernel; no new transport workaround was
needed. Boot image SHA256:
`39775936215d159fce238d0e74c5bcbea707d79dfb7351d20685d79737e640ed`.
It was RAM-booted, never flashed. The init script mounts no phone partitions;
only the explicitly started RPMB broker performs authenticated secure-storage
operations. The broker was stopped before returning to Android.

Android recovery was verified afterward: boot completed, Magisk root available,
one enrolled fingerprint still reported, zero HAL deaths. Actual Android unlock
was not retested during this recovery check.

## Capture/SAVE evidence and limits

`capture-probe.h` implements the Milan HV IRQ exchange using a 0x36a20-byte
TEE payload. REA decompilation of the stock `libgf_hal.so` functions at ELF
0x3072c and 0x30fec established the phases and progress fields. SAVE uses
command 1012/108 bytes; AUTHENTICATE uses 1010/120 bytes. No raw images,
templates, credentials or authentication tokens are exported.

The combined `storage-services.c` broker serves RPMB 8192, FS 10 and GPFS
28672 in one receive loop. Separate brokers conflict on the driver's receive
context (EBUSY). Filesystem state is confined to `/run/fingerprint-state` in
RAM. RPMB still performs authenticated writes for the isolated diagnostic UID;
RAM-only template storage does not mean all secure-storage operations are
read-only. No phone filesystem partition is mounted.

Evidence: `.build/fingerprint/android-comparison/save-r3-logs/`, including
capture, storage, kernel and uptime logs. The RAM boot image SHA256 was
`171e22c3ac8c9b06acf293f01bd8c6f20cc3785c892222988842f7d35fe5f12f`.
The capture executable was replaced over USB after boot to add expected
identity validation, so the embedded executable alone does not reproduce
that guard. The later `--match-probe` implementation builds with warnings
as errors but has not yet been exercised on hardware. It enumerates the
isolated template and imports the wrapped key without creating a credential,
then runs a single match attempt and cancels.

The 360-second watchdog returned the phone to Android. Recovery was checked:
boot completed, Magisk root available, Android fingerprint count one, HAL deaths
zero. The diagnostic RAM template was lost at reboot. Neither persistence
across reboot nor Linux desktop unlocking is established.

## Controlled follow-up (2026-11-04)

The next RAM session (`controlled-r4-logs/`) again completed enrollment and
SAVE, then cancelled. A fresh helper process enumerated the one saved template
and initialized authentication without creating another Gatekeeper credential.
This establishes reload across helper processes within the same RAM session,
not persistence across reboot.

The user confirmed touching a different finger. That attempt returned 1064
(0x428), whose vendor HAL branch says "match fail and retry". It was not an
accepted match, but it is not the terminal 1006 rejection result either.
The single-attempt helper initially cancelled on this retry; it now continues
collecting on 1064 within the existing bounded timeout. Both variants compile
with `-Wall -Wextra -Werror`.

The enrolled-finger prompt received no response during the live session.
The initial window had zero IRQ events; the reopened window saw one empty
IRQ response and no image event. Both timed out and cancelled successfully.
No controlled positive match or terminal negative result is claimed for this
session. `--save-probe` now stops after verified SAVE so test fingers cannot
be confused with enrollment touches during an automatic transition to matching.

## Controlled matching passed (2026-11-04)

In `controlled-r5-logs/`, enrollment reached zero remaining, SAVE succeeded,
and enumeration returned one template. After restarting the helper, the user
confirmed touching the enrolled finger: `positive.txt` reports vendor status 0,
`MATCH_RESULT accepted=yes identity=expected`, and successful CANCEL. This
checks the returned group and template ID, not just the command status.

The confirmed different-finger attempt initially stalled after 1064. Inspection
of `gfdip-hv-irq.c` (stock ELF function 0x3072c) established that its retry loop
sets payload phase at 0x36a18 to 2 and immediately invokes 1016 again with the
same payload. Waiting for another GPIO event was incorrect. The diagnostic now
continues that phase with a bounded sixteen-command loop. It also distinguishes
1006 accompanied by fast/accidental-touch flags at offsets 108/109 from a normal
rejection.

After rebuilding and uploading the corrected helper in the same RAM session,
the user again confirmed a different-finger touch. `negative-fixed.txt` records
1064, phase-2 continuation, then 1006 and `MATCH_RESULT accepted=no`, followed
by successful CANCEL. Thus one controlled positive and one controlled negative
passed. This is functional evidence, not a measurement of false-accept or
false-reject rates. Source compiles with `-Wall -Wextra -Werror`.

All logs are under `.build/fingerprint/android-comparison/controlled-r5-logs/`.
Template bytes were not exported. The phone was rebooted normally to Android
after collecting the evidence. Persistence across phone reboot, production
credential policy, GTK integration and desktop login remain unimplemented.

## Historical investigation

# Enrollment investigation — 2026-11-03

Measured on the R2 temporary kernel, through USB SSH. No boot partition write.

The beryllium `libgf_hal.so` has different payload sizes and offsets from the
`gfenu` research client. Its Milan HV implementation delegates enrollment to
`gf_hal_common_enroll`, and uses these class-1 commands:

| Command | Payload | Fields/results |
|---|---:|---|
| INIT_FINISHED 1005 | 100 | zeroed; succeeds |
| SET_ACTIVE_GROUP 1014 | 100 | u32 group at 96; group 1000 succeeds |
| ENUMERATE 1015 | 180 | group IDs at 100; ten fingerprint IDs at 140 |
| PRE_ENROLL 1006 | 104 | u64 challenge returned at 96; nonzero |
| ENROLL 1007 | 176 | group at 96; packed 69-byte HAT at 105 |
| CANCEL 1009 | 96 | succeeds after rejected enrollment |

Enumeration before SET_ACTIVE_GROUP returns 1088, GF_ERROR_NO_GROUP_ID.
After selecting group 1000, enumeration succeeds and returns zero fingerprints.
This is the app's group, not a claim about all groups or Android fingerprints.
No template persistence has been verified.

A challenge-only token (version 0 and current challenge, remaining fields zero)
returns 1057, GF_ERROR_UNTRUSTED_ENROLL. The firmware's error table confirms this
name. The gfenu challenge-only approach does **not** work on gfber. The shipped
app reports this error; it never treats enrollment start as enrollment success.

## Signed token investigation

The original vendor partition was mounted read-only to inspect:

- `lib64/hw/android.hardware.gatekeeper@1.0-impl-qti.so`
- `lib64/libkeymasterdeviceutils.so`

These binaries use the legacy binary Gatekeeper protocol, not the Android 13
CBOR protocol documented for gfenu. Keymaster is bootloader-resident and opened
on tee0 by name `keymaster64`; do not dynamically load a replacement.

GET_VERSION 0x200 succeeded: API 3.0, TA 3.132.
Negotiation 0x207 with `{4,5,4,5,0}` succeeded, matching the stock library.
A diagnostic enroll request 0x1001 for an isolated app UID 0x53504650, with a
random 32-byte credential and no prior handle, returned -30. No handle or HAT
was produced. The random credential was wiped and was not logged or saved.
Android UID 0 was not used, no existing credential was supplied, and no delete
or reset command was issued.

Request layout derived from the actual vendor library:

- enroll: command, u32 UID, old handle offset/length, old secret offset/length,
  desired secret offset/length; 32-byte header, followed by data.
- verify: command, u32 UID, u64 challenge, handle offset/length, secret
  offset/length; 32-byte header, followed by data.
- response: status, object offset, object length; validate bounds before copying.

Next work: determine why the vendor-derived fresh credential request is
rejected, obtain a legitimate challenge-bound HAT, and only then implement the
Milan HV IRQ/capture, progress, SAVE and matching protocol. The IRQ payload is
223776 bytes; it cannot fit the current 4096-byte probe buffer.

Artifacts under `.build/fingerprint/` include `gatekeeper.asm`,
`keymaster-utils.asm`, `gfber-gatekeeper-test.c`, `gatekeeper-token.h`, and the
app action JSON results. These experimental Gatekeeper probes are not shipped
in the app. Do not report enrollment/matching as complete.

UI validation: the independent on-phone test passed with 1872 pixels of content,
a 976-pixel viewport, and a 780.8-pixel scroll movement. All eight backend tests
passed. Final phone checks: kernel taint 512, no failed system services, Wi-Fi
active, and the original firmware search path restored.

## Follow-up: secure storage and signed HAT (evening, 2026-11-03)

The earlier -30 result has now been resolved in a diagnostic test. Disassembly
of the phone's actual Keymaster firmware located the failing Gatekeeper path:
`gk_get_rpmb_data_uid()` returned no credential record when its RPMB storage
initialization failed. The missing Linux RPMB listener was implemented using
Qualcomm's published UFS transport and listener protocol.

Measured on this phone:

- UFS 2.1, RPMB BSG `/dev/bsg/0:0:0:49476`, 32768 512-byte sectors,
  reliable transfer capacity 64 frames.
- Listener 8192 registration with 25600 bytes returned EINVAL. Using the
  page-rounded 28672-byte buffer succeeded, with no kernel change.
- A read-only broker returned successful authenticated reads; Gatekeeper then
  attempted writes and failed when the broker intentionally refused them.
- With bounded authenticated writes enabled, Gatekeeper command 0x1001 returned
  success and a 58-byte handle. Verify 0x1002 returned a 69-byte signed HAT with
  the requested challenge. Two diagnostic enroll/verify runs succeeded.
  These runs used app UID 0x53504650, not Android UID 0. Credentials/handles/HATs
  were wiped, never logged or exported. Gatekeeper secure records were updated;
  this was not a read-only experiment.
- Goodix still returned 1057 for the signed HAT before key exchange.

The original `libgf_ca.so` supplies an additional prerequisite during open:
Keymaster `{0x205, 2}` returns a wrapped authentication-key envelope, then the
whole 256-byte response goes to Goodix **class 2, command 109**. The Linux
preflight now reproduces that path, without logging or persisting key material.
Keymaster currently returns `-16773121` (`0xff000fff`) before the envelope can
be passed to Goodix. Requests with and without version negotiation, and with
256- or 2048-byte response capacities, gave the same failure.

Adding FS listener 10 and GPFS listener 28672 alongside RPMB did not change the
failure. These were temporary services using an isolated root-owned directory;
they were stopped after the tests. HMAC-sharing command 0x20e is unsupported
by this API 3 firmware (-40); it is not a solution.

Keymaster's code obtains the recipient from `fpta_name`, falling back to
`fingerprint`, then calls `qsee_encapsulate_inter_app_message`. The devcfg
firmware contains the default `fingerprint` string, while the loaded trustlet
is `gfber`. This is a lead, **not proof of a configuration mismatch**: the live
property value and exact failing secure-world call have not yet been observed.
Do not change secure configuration, bypass HAT validation, or replace firmware
based on this inference. A separate diagnostic kernel is being built to obtain
the secure firmware's actual failure message.

Reproducible sources and bounds tests are in `diagnostics/`. The installed app
is unchanged, no fingerprint was saved, and matching remains unimplemented.

### Diagnostic kernel result

The explicit log kernel built and temporarily booted successfully. Image SHA256:
`9253d4b2dd89ea1346c2fe1f45ef3f1f602596456d02fb7a857f3a73fa916ab0`.
The firmware accepted the standard 32 KiB log registration; the root-only
buffer stayed entirely zero after both normal and early key-exchange tests.
No secure error detail was obtained. Repeating Keymaster 0x205 before Goodix
commands 1000/1001 also returned `0xff000fff`, ruling out that initialization
order difference for this test.

The phone's `cmnlib64` implements that value as a generic inter-app-message
error. It can represent a missing service object or several underlying
failures, so the numeric status alone does not identify the cause. The
`fpta_name` hypothesis remains unconfirmed. No secure configuration, firmware,
boot state, rollback value, or provisioning key was modified to test it.

The diagnostic kernel was then replaced by a temporary fastboot boot of the
previously tested R2 image. The logging patch remains explicit opt-in only.
The required next evidence is a known-working Android enrollment baseline or
an equivalent vendor trace of the key exchange; the previous Android ROM and
whether enrollment worked there have been requested from the user.

Recovery verification: R2 is running (diagnostic log node absent), app and
hardware service restarted, app sensor check passed with 88x108 geometry and
32 OTP bytes present. No failed system services; kernel taint remains 512.
Installed boot SHA256 remains
`8c27a090763dc1a31ec4cd8b0419756a81d876cf40e9b5b9589f28ffebbbfdf0`.

### Local MIUI reference comparison

Inspected the user-provided
`miui_POCOF1Global_V12.0.3.0.QEJMIXM_cf3fccffce_10.0.zip` without installing it.
The complete ROM images for keymaster, devcfg, cmnlib64 and tz match the
corresponding previously captured phone partition prefixes byte for byte.
The phone partitions are larger than the packaged images; this comparison
covers the image lengths, not the extra partition padding.

All nine gfber files (MDT and B00–B07) extracted from the ROM modem FAT16 image
match the phone's firmware exactly. Reconstructed the vendor ext4 image from
its Brotli new-data stream and transfer list; `/lib64/libgf_ca.so` and
`/lib64/libgf_hal.so` also match the captured phone libraries exactly.
Artifacts remain under ignored `.build/fingerprint/miui-reference/`.

This rules out a difference between these specific components and this ROM
as an explanation. It does not prove Android enrollment works or rule out
missing Linux initialization, secure services, or persistent secure state.
Replacing these components with the archive copies would make no change.
No firmware was flashed or installed during this comparison. Phone SSH was
reachable and reported 7.1.0-rc1; no enrollment success is claimed.

The user confirmed enrollment worked in Android immediately before Linux and
that the working ROM was this MIUI version. This is a user-reported historical
baseline, not a fresh Android trace. The stock `libgf_ca.so` embeds
`keymaster64` at file offset 0xfa0, matching our resident-app selection.
A new credential-free probe tested command 0x205 with both the stock 8-byte
request and a 64-byte-padded request while Goodix was loaded. Both had transport
status 0 and Keymaster status -16773121. Padding does not resolve the failure.
No Gatekeeper credentials or fingerprint templates were created by this test.
Phone remained reachable, no failed system services, taint 512 after testing.
The missing prerequisite for secure inter-app key exchange remains unresolved.

## App integration staged (2026-11-04)

Fingerprint Lab 0.3 builds the gfdip capture helper and combined storage broker
from pinned source archives. The Linux service has a persistent root-only state
directory, live progress events and a nonempty-group enrollment guard. This is
source/package integration, not a completed on-phone deployment. Android remains
installed; disk-backed persistence across reboot and the new GTK flow still need
a Linux userspace test. No production login or password-bound enrollment policy
is claimed. See `apps/spectre-fingerprint/README.md` from the project root.
