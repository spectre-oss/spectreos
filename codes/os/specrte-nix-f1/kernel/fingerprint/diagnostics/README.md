# Fingerprint authorization diagnostics

These tools are **not installed by the app**. Enrollment is still incomplete.
Build on the build host with `kernel/fingerprint/diagnostics/build.sh`.
The resulting ARM64 binaries are under `.build/fingerprint/diagnostics/`.

`rpmb-bridge --geometry` only reads UFS descriptors and request-sense data.
Without arguments, the broker registers QSEE listener 8192 and forwards
firmware-authenticated RPMB operations. This mode can change secure storage:
Gatekeeper credential creation and verification update its records. It is not
a read-only test. It never provisions a key or erases/reformats RPMB. Both read
and write paths reject key-programming frames, validate counts and offsets,
and keep frame contents and authentication material out of logs.

The diagnostic Gatekeeper UID is `0x53504650`, separate from Android UID 0.
The probe creates an ephemeral random credential only after the wrapped-key
exchange succeeds; the credential and handle are not persisted. Do not use
this ephemeral credential scheme as production enrollment policy.

`gatekeeper-preflight.c` reproduces the original Goodix client's Keymaster
`{0x205, 2}` request and class-2 command 109 before attempting enrollment.
With `gfdip`, the key exchange and signed enrollment start now succeed.
Compile with `-DCAPTURE_PROBE -DFINGERPRINT_TA='"gfdip"'` to enable
`--capture-probe` (no SAVE), `--save-probe` (SAVE then cancel),
`--save-match-probe` (isolated SAVE and matching),
and `--match-probe` (one matching attempt against exactly one saved template).
The latter imports the key without replacing the diagnostic credential.
Hardware evidence and unverified limits are in [ENROLLMENT.md](../ENROLLMENT.md).
The app still uses the older helper and must not claim completed enrollment.

`capture-init` is a RAM boot init with a 360-second Android-return watchdog.
`storage-services.c` combines RPMB, FS and GPFS in a single receiver, using
qsee-supplicant commit `36e06680cf7f690fccbdcd07abc2a64c4bb061d8` from
https://github.com/wrobelda/qsee-supplicant. Its existing local source changes
must also be audited before treating this as a reproducible production build.
Use `/run/fingerprint-state` for test storage; never expose it via the log
HTTP server. The old FS-only service table is not compatible with a separate
RPMB receiver on this driver.

The UFS reference files are fetched at build time from Qualcomm MinkIPC commit
`5e3c6951d13162a262ea65ea8e8351228f092b58`, under
`listeners/librpmbservice/`. Their SPDX licenses are retained in downloaded
files: BSD-3-Clause-Clear and BSD-3-Clause. See
https://github.com/qualcomm/minkipc/tree/5e3c6951d13162a262ea65ea8e8351228f092b58/listeners/librpmbservice.
The local broker is GPL-2.0-only; downloaded firmware is not included.

Run bounds/key-programming rejection tests on the build host after building:

```sh
cc -O1 -g -Wall -Wextra -Werror -fsanitize=address,undefined \
  -DRPMB_DEBUG_CONSOLE_OUTPUT -I.build/fingerprint/diagnostics \
  kernel/fingerprint/diagnostics/test-rpmb.c \
  -o .build/fingerprint/diagnostics/test-rpmb
.build/fingerprint/diagnostics/test-rpmb
```

Thirteen malformed or disallowed requests passed without reaching a storage
ioctl. This is not exhaustive validation or a production security review.

`kernel.nix` explicitly builds a diagnostic kernel with a root-only
`/sys/kernel/debug/poco-qsee-log` buffer. It is not part of the normal kernel
or the R2 fingerprint build. Treat its logs as sensitive; inspect only the
needed error messages. The log allocation lives until reboot because secure
firmware retains it. The SCM driver cannot be unbound. Use temporary fastboot
boot only, preserving the installed image; do not use kexec.

`key-exchange-only` loads Goodix and opens the resident `keymaster64`, then
compares 8-byte and 64-byte-padded command 0x205 requests, including the
stock adjacent-response layout, for authentication types 2/1/4 and invalid-type 0 control. The TEE driver
repackages response bytes adjacent to the request in physical memory; the
userspace offset is therefore not an independent physical-layout variable.
It does not create a
Gatekeeper credential, import the returned envelope, or attempt enrollment.
Responses are wiped without logging key/envelope contents; only status and
public offset/length fields are printed. The dispatcher clears those fields
on error, so zeros do not identify the failing stage. Use the normal `run-info.sh`
firmware-path/lock wrapper, installing this binary as `gfber-identify` in its
own temporary directory. Exit status indicates transport-test completion;
inspect each printed Keymaster status for command success.

REA analysis and evidence references are recorded in [REA-NOTES.md](REA-NOTES.md).

The opt-in transport comparison is documented in [WHITELIST.md](WHITELIST.md).


## Working Android comparison (2026-11-04)

`stock-probe --resident gfdip` queries only an already loaded trustlet and the
resident Keymaster; it refuses to load a replacement. With the normal Android
services present, wrapped-key request 0x205/type 2 succeeds. Compile the Linux
`key-exchange-only.c` and `gatekeeper-preflight.c` with
`-DFINGERPRINT_TA='"gfdip"'` to select the matching firmware. The legacy default
remains gfber for reproducibility, not as the recommended trustlet.

`key-import-only.c` selects gfdip and imports the wrapped key (class 2/109).
It sends no credential, enrollment, capture, save or template commands.
It wipes the shared buffers and prints only status. Build it with the same
static ARM64 compiler and warning flags as the other diagnostic tools.

`android-comparison-init` is the RAM-only USB harness for the comparison,
accepted by `kernel/sensors/pack.py --init`. The separately supplied extra root
contains `/opt/key-exchange-gfdip` and signed `/opt/firmware/gfdip.*` extracted
from this phone. No firmware is committed. Its watchdog returns to Android
after 180 seconds. The RPMB broker and enrollment probe are manually invoked
only for the explicit isolated-identity test documented in ENROLLMENT.md.
