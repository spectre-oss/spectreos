# Local code, remote builds

The two working directories are peers:

- Here: `<local-checkout-path>`.
- Server: `<build-user>@<build-host>:<remote-checkout-path>`.

`remote.json` contains connection settings. Commands use the supplied SSH key and
explicit connection options, bypassing the host's broken system SSH config.
SSH still checks known host keys normally. No password or private key is copied.
Python 3 is required on both hosts; rsync is required for artifact downloads.

## Split local layout

The outer workspace contains `README.md`, `AGENTS.md`, `codes/` and `docs/` as
visible root entries; AGENTS.md supplies automatic Codex guidance. `docs/general/` holds the general knowledge base and does not sync
into the build-server project. `docs/spectre-nix/` maps to the server project's
`docs/` on every sync, in both directions. The source tree's `docs` symlink
points to that local directory to preserve project-relative references.

The coordinator scans the external documentation directory explicitly, without
following arbitrary symlinks. A missing mapped docs directory aborts sync,
preventing an accidental mass deletion. `remote.json` configures the relative
local docs mapping. The server's project path/layout does not change.

The original `.code-sync/` baseline and backups were moved with the source;
deletion history is preserved. General knowledge-base files formerly uploaded
under the server's `docs/knowledge-base/` are removed from that project during
migration, with its normal per-file backups, and retained under local
`docs/general/`. This is not a reset of the baseline or Git history.

## Sync code both ways

From the source project root (`codes/spectre-nix/`):

```bash
python3 infra/sync.py                 # preview only
python3 infra/sync.py --apply         # synchronize both directions
```

Run the sync command on this computer; it detects edits made on either computer.
It uses content hashes and executable bits against a local baseline, rather
than modification times. The first run merges unique files from both trees;
different content at the same path is a conflict. Subsequent runs propagate
one-sided additions, edits, executable-bit changes, and deletions. If both sides
change a file differently, the entire plan stops before applying any change.
Resolve both copies to the intended content, or save one under a different name,
then retry. There is no automatic winner or force-overwrite option.

Overwritten/deleted files are backed up on their respective hosts under
`.code-sync/backups/RUN-ID/`. `.code-sync/baseline.json` on this computer records
the last verified common state. Keep it: deleting it loses deletion history.
Backups and state are excluded from synchronization. No Git commit, stash,
checkout, clean, reset, or push is performed; the remote worktree remains dirty.

Pause editors/agents on both sides during sync and server builds. Sync checks
expected content before each write, atomically replaces individual files, and
verifies both trees before updating the baseline. It is not a distributed
filesystem transaction: interruption or an editor racing the checks may leave
a partial run. Backups preserve overwritten content; rerun and inspect conflicts.
Only one coordinator on this computer is supported; do not independently set up
a second coordinator/baseline against the same server directory.

Git metadata, `result*`, `.build`, Nix stores, `dist`, `artifacts`, caches,
credentials with common key names, binaries, archives, and generated images are
excluded. Root upstream checkouts `mobile-nixos`, `u-boot`, and `phosh` are also
excluded, matching the project's regeneration workflow. Maintain their changes
as project patches. The implementation's exclusion list is in `infra/sync.py`.
Only regular UTF-8 files up to 10 MiB without NUL bytes are synchronized; symlinks
are never followed. Skipped binary/oversized files are reported. Empty
directories, ownership, ACLs, and non-executable permission bits are not synced.
This filter is not a secret detector: do not put credentials into source files.

## Build only on server

```bash
python3 infra/server.py build test
python3 infra/server.py build boot-image --panel tianma
python3 infra/server.py build build --panel tianma --jobs 3 --max-jobs 2
```

The wrapper syncs first, then runs the selected Makefile target through SSH.
Install/flash targets are not exposed. Remote sources, firmware archives and Nix
stores stay on the server. The existing Makefile remains available for server
use; do not invoke its build targets locally. Do not sync while a build runs.

## Artifacts are separate and opt-in

The existing release publisher writes to `../dist` on the server.

```bash
python3 infra/server.py releases
python3 infra/server.py fetch VERSION/tianma/BUILD-ID
```

Copy the relative directory from the listing, replacing the placeholders.
Downloads go to `artifacts/VERSION/tianma/BUILD-ID/` here, outside code sync.
Rsync supports resuming interrupted downloads; the wrapper verifies file sizes
and SHA-256 hashes against the release manifest. Confirm the panel before
installation. Fetching performs no phone command, reboot, flash,
erase, or installation. Phone installation will be configured separately when
requested.
