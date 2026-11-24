# Controlled secure-call comparison

This diagnostic tests the stock driver's seven-argument command6 against the
existing five-argument command1 for Keymaster command0x205. It is not a general
whitelist implementation. Goodix commands with embedded buffer pointers remain
on the legacy path; they would require populated table entries.

Build `whitelist.nix` with the same Nix store/options as the parent fingerprint
README. Package with `prepare.py`, preserving the installed boot backup's
initrd, DTB and command line. Use temporary fastboot boot only.

The runtime flag is `/sys/module/qcom_scm/parameters/qseecom_diag_whitelist`.
It defaults to N and is accessible only to root. Stop fingerprint services
before testing. With the key-only probe in a separate directory:

```sh
set -eu
flag=/sys/module/qcom_scm/parameters/qseecom_diag_whitelist
trap 'echo N > "$flag"' EXIT
echo N > "$flag"
sh /tmp/fingerprint-key-only/run-info.sh
echo Y > "$flag"
sh /tmp/fingerprint-key-only/run-info.sh
echo N > "$flag"
sh /tmp/fingerprint-key-only/run-info.sh
```

Inspect transport and command statuses, not just the process exit code. The
probe creates no credentials, imports no keys, and captures no fingerprints.
It reports only authentication type, layout, status, and public response header
fields. Any envelope is wiped. Type0 is an invalid-input control, expected -38.
The driver physically packs the response after the request regardless of the
userspace response offset; request64 gives the stock aligned layout.

After testing, keep the flag disabled, stop transient diagnostic services,
restore the fingerprint service and GUI, and verify sensor and hardware health.
The installed boot partition must remain unchanged.

## Result

The temporary kernel booted and both interfaces completed requests successfully
at the transport level. N/Y/N comparisons, both without listeners and with
RPMB/FS/GPFS/time present, returned the same Keymaster failure (-16773121).
Invalid-type controls returned -38. The experiment did not fix enrollment.
See REA-NOTES.md for artifact identity and limitations.
