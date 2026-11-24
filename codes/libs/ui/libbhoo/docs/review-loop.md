# Focused webxdc review loop

The full [goal](goal.md) and root [AGENTS.md](../../../AGENTS.md) require this loop
for future Bhoo component deliveries. Previous standalone send scripts are retained
as history and must not be used for new review tests.

## Transport and identity

Use the user-supplied delta CLI v0.1.1. The `bhoo-review.py` CLI also accepts a global `--ledger LEDGER` option to point to an alternate review ledger file. The downloaded executable's SHA-256 is
`9468bfaa5a6ea308f2d23664ad6b4b52eca35644acfadf4ecb7bdbca454f84ad`, checked
against the publisher's release checksum before execution. Its reference and
source snapshots are in ignored `context/delta-cli/`.

- [Pinned CLI release](https://github.com/omidz4t/delta/releases/tag/v0.1.1)
- [CLI reference](https://github.com/omidz4t/delta/blob/v0.1.1/docs/REFERENCE.md)

The existing delivery identity was exported through core backup and restored into
a separate private delta data directory. Configuration is ignored at
`context/bhoo-review/config.json`. Keys, credentials, encrypted migration backup,
passphrase and logs are outside the repository. The live account is in
`~/.bhoo-review/d`; other private files remain in `~/.local/share/bhoo-review`.
The short account path keeps Delta app control sockets below the Unix path limit.
Only the restored account is used for this loop; do not run legacy senders or
another worker against the old identity concurrently. No account reset is needed.

## Package and operate

First build the relevant focused Bhoo-only component webxdc using
`tools/build-bhoo-webxdc.py --plain --component ...`. Prepare a review JSON document
like `reviews/select-typeahead-r1.json`, then run:

```sh
python tools/bhoo-review.py status
python tools/bhoo-review.py package \
  --source context/dist/single-component/bhoo-select-typeahead.xdc \
  --spec codes/libs/ui/libbhoo/docs/reviews/select-typeahead-r1.json \
  --test-id select-typeahead-r1 \
  --output context/dist/single-component/bhoo-select-typeahead-review.xdc
python tools/bhoo-review.py serve
```

For a repaired issue append `--repairs issue-TEST_ID` to the package command, and
use a new test ID. IDs are unique; a sent package is immutable and hash-checked.
The supervisor sends every ready test in registration order without waiting for
prior decisions; an unresolved rejection does not block delivery. It registers
an explicit `--type update` hook filtered to protocol `bhoo-test-review/v1`, then
starts the named app worker. There is no dependency on ephemeral realtime delivery.
The app is framed around the actual Bhoo WebAssembly component, with no Angular
comparison. Review controls are ordinary embedded HTML, CSS and JavaScript without
an additional UI framework; the component itself uses Bhoo.

## Feedback and deletion

`reviews/state.json` is the authoritative, fsynced ledger; `reviews/problems.json`
is its readable repair-queue projection. They contain no account credentials.
`status`, the hook and the supervisor reconstruct the projection from the ledger.
Feedback has its test, purpose, explanation, decision, update serial, event sequence
and receive time. The first valid review wins across duplicate batches and devices.

The hook checks test ID, instance token, named app and original core message ID.
Blank/oversized rejection details, foreign instances, unknown decisions and invalid
identifiers cannot schedule deletion. Text is stored literally. Acknowledgment is
returned only after durable writes complete. If a hook acknowledgment fails after
the write, the supervisor recovers it through app-update; the journal provides
evidence of hook completion. Rejections become repair issues; a linked retest must
be accepted before they are resolved. A further rejection reopens linked issues.

After feedback is saved and the hook action succeeds, the supervisor stops the
current worker and calls `delta delete-message ORIGINAL_ID --for-all`. Only that registered
reviewed message is eligible. Source packages and local feedback are retained.
Core queues a recipient deletion request; recipient support and subsequent network
delivery determine when it disappears. This is not proof of remote deletion.
The supervisor keeps networking alive to transmit queued operations.

## Restart and failures

The supervisor uses a process lock, and its PID is saved privately. Keep it running
as a service or restart `python tools/bhoo-review.py serve` after a host restart.
On this workstation `bhoo-review.service` is installed and enabled as a systemd
user service. It restarts on process failure and starts with the user service
session. A systemd drop-in uses `KillMode=mixed` and a 240-second stop timeout
to let an in-progress send finish before child processes are stopped. Inspect it with `systemctl --user status bhoo-review.service` and
`journalctl --user -u bhoo-review.service`. Its unit is in
`~/.config/systemd/user/bhoo-review.service`; it runs the CLI above from this
workspace with a private umask. Do not start a second manual supervisor alongside it.
The CLI's app worker survives its launch command but is not itself a boot service.
Because delta permits one account worker, the supervisor rotates among all
outstanding test workers. Durable updates and the hook journal are replayed on
each worker start. Feedback may take a full rotation to be processed. Already saved decisions are idempotent. Only one
supervisor may own the review queue at a time.

A send is marked `sending` before touching the network. Recovery inspects the saved
app registry instead of sending a second message blindly. If the result cannot be
established, inspect private logs and the delta app registry before changing that
state. Disk failures or a failed hook keep the original message available. Do not
delete unrelated or unreviewed messages to unblock a queue.

## Validation

### Long review examples on phones

Embedded demos normally render into a fixed viewport. For a vertically static example taller than that viewport, the review specification may set `demo_height` to an integer between 500 and 4096 CSS pixels. The packager gives that demo a taller canvas and native touch scrolling, while retaining its own page and controls. Verify the complete target example at phone width and test its interactions before queuing. Do not opt in when the demo itself handles vertical drag gestures: native page panning would compete with them. This is an explicit per-test workaround, not general scrolling support for every Bhoo scene. Existing sent reviews and their feedback must not be altered.

The previously identified long, unsent Android control reviews were updated with this viewport; their content and review tokens are unchanged. A 320 × 500 touch-emulated browser check of the SeekBar thumb example confirmed that a swipe scrolls to the formerly clipped controls, and a subsequent horizontal drag changes the custom thumb slider from 50 to 86 without runtime errors. A 320 × 500 automated browser pass also opened all 17 updated queued archives: each initialized without reported runtime errors and all exposed control bounds fell within its enlarged canvas. This geometry check does not establish visual parity or verify every interaction; the individual reviews remain necessary. The review workflow unit tests cover viewport generation and invalid height or document rejection.

`python tools/test_bhoo_review.py` checks durable-before-ack ordering, malformed
updates, foreign app/message/token rejection, duplicate and conflicting feedback,
literal explanation storage, failure before persistence, and linked retest issue
resolution. Browser checks exercise required rejection details, actual button
payloads and acknowledgment presentation. Live hook checks use an isolated ledger
and the bot's self-chat, never a simulated approval of the user's production test.

Validation on 2026-11-12: nine Python checks passed; `node tools/test-bhoo-review.mjs`
passed purpose rendering, required rejection details, OK/rejection payloads,
acknowledgment and 390px phone layout. The real delta self-chat worker processed an
isolated rejection with a successful `review-feedback` hook result, saved an issue
and queued an acknowledgment. `delete-message --for-all` accepted cleanup of that
self-test. The production Select test is awaiting the user's independent review;
no acceptance or rejection has been synthesized for it.

Neither automated tests nor a focused user approval establish completion of the
full Android and Material goal. Component-level implementation and device-parity
limitations remain documented separately.
