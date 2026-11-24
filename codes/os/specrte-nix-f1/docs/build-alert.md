# Remote build sound

`scripts/build-alert.py` watches a build process and plays a short three-tone
alert on your desktop when that process exits, whether it succeeds, fails or
is killed. It checks every ten seconds without calling an LLM. A Linux process
start-time check protects against PID reuse. The build continues independently
of the monitor.

The build host must be able to SSH into the desktop as the logged-in desktop
user, using an already trusted host key and noninteractive authentication.
An SSH alias configured only on your laptop is not automatically available on
the build host. The desktop needs Python 3 and a working `paplay`, `pw-play` or
`aplay` command. The sound uses the existing desktop audio session and volume.

First test the connection and sound from the project directory:

```sh
python3 scripts/build-alert.py --host user@desktop-address --test
```

Then monitor the current build PID (replace `12345`):

```sh
nohup python3 scripts/build-alert.py --host user@desktop-address --pid 12345 \
  > .build/alert.log 2>&1 < /dev/null &
```

The current detached build PID is recorded as `imagePid` in
`.build/processes.json`. Start one monitor for each new build process; it exits
after delivering its alert. Avoid launching duplicate monitors for one PID.
If SSH or audio fails, the monitor preserves the event and retries every minute.
`.build/alert-state.json` records whether it is watching, waiting to notify, or
has delivered the alert. Logs and state are ignored by Git. The sound is generated
in a temporary directory on the desktop and removed after playback. No audio
files or credentials are committed. This monitors process exit, not a hung build.

## Run from your own PC instead

When `<build-host>` is your PC's SSH alias for the build machine, run
this on your PC. It watches the current build without requiring reverse SSH.
SSH failure prints a connection warning instead of claiming the build stopped.
The terminal bell needs audible bells enabled in your terminal settings.

```sh
if ssh <build-host> 'while kill -0 2132221 2>/dev/null; do sleep 10; done'; then
  printf '\nBuild stopped or finished.\a\a\a\n'
else
  printf '\nSSH disconnected; reconnect to check the build.\n'
fi
```

`2132221` is the build PID when this example was written. For later builds,
read `imagePid` from the project's `.build/processes.json` and replace it.
Keep this terminal open while watching. This simple shell example does not
protect against PID reuse; the Python monitor above does.
