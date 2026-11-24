# Local / build-server workflow

- This computer is the code workspace and USB phone installation host.
- Local sources live in `codes/spectre-nix/` under the outer workspace. General
  notes live in outer `docs/general/`; `docs/spectre-nix/` maps two ways to the
  server project's `docs/` on every sync. The local source `docs` symlink points
  there. Keep the outer visible root limited to README.md, AGENTS.md, codes/,
  and docs/. The root AGENTS.md provides automatic workspace-wide guidance.
- Run all builds on the configured build host (`<build-user>@<build-host>`), in
  `<repo-root>`. Never build locally.
- Preview two-way sync with `python3 infra/sync.py`; apply with `--apply`.
- Use `python3 infra/server.py build TARGET` for server builds. This syncs first.
- Pause edits on both sides while syncing. Resolve conflicts explicitly; do not
  overwrite either side's unsynchronized changes or change the remote Git history.
- Only text source/configuration/documentation is synchronized. Git internals,
  generated outputs, firmware binaries, archives, local state and downloaded
  upstream trees stay on their respective hosts.
- Fetch published artifacts separately using `python3 infra/server.py fetch
  VERSION/PANEL/BUILD-ID`. Artifact transfer does not authorize flashing/erasing.
- Do not flash or erase the phone without explicit user authorization.

## Use the documentation vector database

- On the local computer, search project memory before answering documentation,
  hardware, architecture or known-troubleshooting questions:

  ```bash
  # From codes/spectre-nix/:
  python3 infra/docs-vector.py search "YOUR QUESTION" --limit 5
  ```

- Open the returned files at the reported lines and distinguish historical
  snapshots from current measurements. Cite source files, not vector scores.
- After documentation changes/imports or a sync that changes docs, run
  `python3 infra/docs-vector.py update`. Use `status` for index health and `test`
  after changes to retrieval behavior.
- The local helper uses `~/llmcli` and the owner-approved Cloudflare
  `@cf/baai/bge-m3` embeddings. Approval covers this project's normal document
  indexing and search; do not ask again for the same scope.
- The index is local in `.code-sync/docs-vector/` and never synced. On the build
  server, this database/binary and the outer general docs are not available;
  read project docs directly or perform vector searches on the local computer.
- If retrieval fails, explain the limitation and fall back to `rg`/source reads.
  Do not silently change providers or treat archived statements as live state.
