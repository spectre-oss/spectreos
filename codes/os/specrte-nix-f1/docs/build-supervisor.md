# Background build repair and DeltaChat notifications

Two user services are installed and enabled:

- `spectre-build-supervisor.service`: checks the project's live Nix/make processes
  every ten minutes. Runs Grok for an initial audit, after builds stop, and to verify
  successful images. Healthy builds are not restarted just for polling.
- `spectre-build-notify.service`: delivers queued messages to the already paired,
  pinned owner in the existing encrypted private DeltaChat conversation.

Every agent call explicitly selects `--model grok-4.7 --reasoning-effort low`.
The full agent task is [grok-build-task.md](grok-build-task.md). Grok repair calls
have a 30-minute/40-turn bound. The supervisor retries agent failures up to three
consecutive times, and continues repair rounds while progress is possible. It stops and notifies the owner
when blocked or verified complete. It does not blindly retry unchanged failures
forever. Unchanged active builds produce no notifications and no new LLM calls.
Grok calls use the installed account and may consume its usage allowance.

The supervisor owns future build launches. It uses `scripts/build-managed.sh`: WebKit runs sequentially with four threads,
then smaller packages use two slots with three threads each and `--keep-going`.
Existing healthy builds retain their settings. Completion requires a verified full
distribution and successful `scripts/finish-build.py --apply` cleanup; the final
notification includes the release path. Complete releases remain preserved. Do not independently change settings/restart builds while an
agent repair is active. It adopts existing project builds by inspecting `/proc`,
so no fragile hardcoded build PID is required. A running process is not proof of
progress: this version does not automatically kill a suspected hung build.

## Status and control

```sh
systemctl --user status spectre-build-supervisor spectre-build-notify
journalctl --user -u spectre-build-supervisor -u spectre-build-notify -f
cat .build/supervisor.json
python3 scripts/deltachat-notify.py --status
```

Agent transcripts are `.build/grok-*.log`; the current build log is
`.build/image.log`. On each retry the previous build log is archived. Files under
`.build`, the DeltaChat account, queue database and settings remain outside Git.

Stop supervision with `systemctl --user stop spectre-build-supervisor`.
Systemd also stops subprocesses launched inside that service, including a build
it launched. Existing builds started outside the service are not affected.
Stop notification delivery separately with `systemctl --user stop spectre-build-notify`.

After a terminal `blocked` or `complete` state, inspect and resolve its reason,
stop the supervisor, archive `.build/supervisor.json`, then start the service for
a fresh session. Ordinary service restarts retain pending work and counters.

Templates are in `scripts/systemd/`; install them into `~/.config/systemd/user/`,
run `systemctl --user daemon-reload`, then enable/start both services. Paths in
these templates describe this machine; adjust them for another checkout.
User services survive terminal closure. Surviving the last login session or
starting on reboot requires user lingering; check `loginctl show-user "$USER" -p Linger`.

## Sending a notification from Grok or another local task

```sh
python3 scripts/deltachat-notify.py --key 'distinct-event-id' \
  'Spectre Nix F1: description and next action.'
# Equivalent entry point alongside the bot:
python3 $SPECTRE_DELTABOT_DIR/notify.py \
  --key 'distinct-event-id' 'Spectre Nix F1: description and next action.'
```

The CLI inserts a durable event and returns immediately. Reusing a key does not
queue a duplicate. Messages are limited to 3500 characters and the configured
owner; recipient selection cannot be overridden from the CLI. The worker checks
owner address, pinned contact ID, membership and encryption before sending.
Core's delivered state means transport acceptance, not proof the user read it.

The worker respects the bot's account lock. If the full interactive DeltaChat bot
is using that account, notifications remain queued until it releases the lock;
never open the same account concurrently or kill the bot just to send a notice.
Core-queued messages are checked again after restart. A crash between Core accepting
a send and saving its message ID can still duplicate a message; exactly-once
network delivery is not claimed. Explicit Core failures retry on later passes.

Repair calls carry `.build/grok-progress.md` and the previous response forward, including after turn-limit failures. Grok must checkpoint findings during investigation, implement source fixes and validate them. Committed or pending source changes reset the identical-failure comparison.
