# Read-only time listener diagnostic

`time-listener.c` registers legacy QSEE listener 11 with a 20,480-byte buffer.
It only reads host wall-clock time and replies to secure-world callbacks. It
never sets a clock, creates credentials, reads RPMB, or writes storage.

Protocol references:

- Qualcomm minkipc commit `5e3c6951d13162a262ea65ea8e8351228f092b58`,
  `listeners/libtimeservice/time_msg.h` and `atime.c`.
- Actual MIUI V12.0.3.0 vendor `/lib64/libdrmtime.so`, BuildID
  `dada25d6249ac2aa0d82bfd463b53956`, inspected read-only with debugfs and
  objdump. Disassembly is in ignored `.build/fingerprint/drmtime.asm`.
  Registration at 0x2044 uses 0x5000 bytes. Dispatch jump table at 0x830
  maps commands 0x302 through 0x306 to 0x2268, 0x2444, 0x22e4,
  0x2360, and 0x26cc respectively. The qseecomd listener table contains
  listener ID 11 for time.

All requests begin with a 32-bit command. Responses start with a zero word
(the MIUI daemon leaves its separately zeroed reply header untouched); the
remaining response layouts are packed:

| Command | Response after command | Total bytes |
| --- | --- | --- |
| 0x302 UTC seconds | u32 seconds, u32 nanoseconds (zero), i32 status | 16 |
| 0x303 UTC broken-down time | nine i32 struct-tm fields, i32 status | 44 |
| 0x304 milliseconds | u64 milliseconds | 12 |
| 0x305 host UTC | u32 seconds, u32 nanoseconds, i32 status | 16 |
| 0x306 end | i32 status | 8 |

Unknown commands receive -1; too-small buffers receive a transport failure
without writes. The provider returns zero in the first reply field, matching
the actual MIUI daemon. The newer public callback instead leaves the original
request command intact because it operates in-place. This diagnostic explicitly
uses the installed MIUI behavior.

MIUI calls `time_genoff_operation` for 0x302/303/304 and falls back to
`CLOCK_REALTIME` if the Qualcomm offset service is unavailable. This diagnostic
implements only that fallback. In particular, 0x304 returns whole UTC seconds
times 1000, matching the MIUI implementation at 0x2324–0x233c. Neither this
provider nor the public protocol defines a monotonic request.

This is an experimentally available prerequisite, not evidence that time is the
cause of the fingerprint key exchange failure. It has not been run on the phone
as part of its implementation. Run only under the coordinator's controlled
diagnostic workflow; do not add it as a persistent service yet.

Build and local bounds validation:

```sh
aarch64-linux-gnu-gcc -static -O2 -Wall -Wextra -Werror \
  kernel/fingerprint/diagnostics/time-listener.c \
  -o .build/fingerprint/diagnostics/time-listener
cc -O1 -g -Wall -Wextra -Werror -fsanitize=address,undefined \
  kernel/fingerprint/diagnostics/test-time-listener.c \
  -o .build/fingerprint/diagnostics/test-time-listener
.build/fingerprint/diagnostics/test-time-listener
```

All 520 dispatch/bounds cases and five injected clock/calendar failures passed
with ASan and UBSan. Known-command clock errors return a failure status
(or UINT64_MAX for the status-less milliseconds response). No phone commands were
issued when developing this diagnostic.
