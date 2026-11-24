#!/usr/bin/env python3
"""Install a server-built camera NAR and user launcher; never flash the phone."""
import argparse
import getpass
import hashlib
import json
import os
from pathlib import Path
import re
import shlex
import subprocess


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('artifact', type=Path, help='directory with camera.nar and closure.json')
    parser.add_argument('--host', default='spectre@172.16.42.1')
    parser.add_argument('--known-hosts', type=Path, required=True)
    parser.add_argument('--password-env', help='environment variable containing phone login/sudo password')
    args = parser.parse_args()
    data = json.loads((args.artifact / 'closure.json').read_text())
    app = data['app']
    if not re.fullmatch(r'/nix/store/[a-z0-9]{32}-(megapixels|spectre-camera)-[A-Za-z0-9.+_-]+', app):
        parser.error('unexpected camera application store path')
    is_qt = bool(re.fullmatch(r'/nix/store/[a-z0-9]{32}-spectre-camera-[A-Za-z0-9.+_-]+', app))
    executable = 'spectre-camera' if is_qt else 'megapixels'
    identity = 'org.spectre.Camera' if is_qt else 'me.gapixels.Megapixels'
    gc_root = '/var/lib/spectre-camera/gcroots/' + executable
    nar = args.artifact / 'camera.nar'
    with nar.open('rb') as stream:
        digest = hashlib.file_digest(stream, 'sha256').hexdigest()
    expected = data.get('camera.nar')
    if expected and (expected['sha256'] != digest or expected['bytes'] != nar.stat().st_size):
        raise RuntimeError('local camera artifact checksum mismatch')
    password = os.environ[args.password_env] if args.password_env else getpass.getpass('Phone login/sudo password: ')
    env = dict(os.environ, SSHPASS=password)
    ssh = ['sshpass', '-e', 'ssh', '-F', '/dev/null', '-o', 'ConnectTimeout=15',
           '-o', 'UserKnownHostsFile=' + str(args.known_hosts.resolve()), args.host]

    def run(command, content=None, binary=False):
        result = subprocess.run(ssh + [command], input=content, env=env,
                                stdout=subprocess.PIPE, check=True, text=not binary)
        return result.stdout

    temporary = run('mktemp -d /tmp/spectre-camera.XXXXXXXX').strip()
    if not re.fullmatch(r'/tmp/spectre-camera\.[A-Za-z0-9]+', temporary):
        raise RuntimeError('unexpected remote temporary directory')
    destination = temporary + '/camera.nar'
    with nar.open('rb') as stream:
        subprocess.run(ssh + ['umask 077; cat > ' + shlex.quote(destination)],
                       stdin=stream, env=env, check=True)
    remote_digest = run('sha256sum ' + shlex.quote(destination)).split()[0]
    if remote_digest != digest:
        raise RuntimeError('camera artifact checksum mismatch')
    root_command = ('set -e; nix-store --import < ' + shlex.quote(destination)
                    + '; install -d /var/lib/spectre-camera/gcroots; '
                    + 'nix-store --add-root ' + shlex.quote(gc_root) + ' --realise '
                    + shlex.quote(app))
    print(run("sudo -S -p '' sh -c " + shlex.quote(root_command), password + '\n'))
    desktop = ('[Desktop Entry]\nName=' + ('Spectre Camera' if is_qt else 'Megapixels')
               + '\nExec=' + app + '/bin/' + executable + '\n'
               + 'TryExec=' + app + '/bin/' + executable + '\nTerminal=false\nType=Application\n'
               + 'Categories=' + ('Qt' if is_qt else 'GTK') + ';Photography;Graphics;\nIcon=' + identity + '\n'
               + 'X-Purism-FormFactor=Workstation;Mobile;\nDBusActivatable=false\n'
               + 'StartupNotify=true\nX-Spectre-CameraManaged=true\n')
    script = ('set -e; p="$HOME/.local/share/applications/' + identity + '.desktop"; '
              'mkdir -p "$(dirname "$p")"; '
              'if test -e "$p"; then grep -q "^X-Spectre-CameraManaged=true$" "$p" '
              '|| { echo "Refusing to overwrite unrelated user camera launcher" >&2; exit 1; }; '
              'cp -p "$p" "$p.previous"; fi; cat > "$p"')
    if is_qt:
        run('mkdir -p "$HOME/.local/share/icons/hicolor/scalable/apps"; install -m644 '
            + shlex.quote(app + '/share/icons/hicolor/scalable/apps/org.spectre.Camera.svg')
            + ' "$HOME/.local/share/icons/hicolor/scalable/apps/org.spectre.Camera.svg"')
    run(script, desktop)
    manifest = {'app': app, 'camera.nar': {'bytes': nar.stat().st_size, 'sha256': digest},
                'phone': args.host, 'gc_root': gc_root,
                'launcher': '~/.local/share/applications/' + identity + '.desktop'}
    (args.artifact / 'installed.json').write_text(json.dumps(manifest, indent=2) + '\n')
    run('rm -f ' + shlex.quote(destination) + ' && rmdir ' + shlex.quote(temporary))
    print('Installed camera package and user launcher. No system image or partition changed.')


if __name__ == '__main__':
    main()
