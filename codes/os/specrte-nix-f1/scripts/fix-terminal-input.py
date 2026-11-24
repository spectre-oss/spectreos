#!/usr/bin/env python3
"""Repair the terminal launcher on an existing Phosh image, as its normal user."""
from pathlib import Path
import os


def main():
    source = Path('/run/current-system/sw/share/applications/org.gnome.Console.desktop')
    target = Path.home() / '.local/share/applications/org.gnome.Console.desktop'
    if target.exists() and 'X-Spectre-WaylandInput=true' not in target.read_text():
        raise SystemExit(f'Refusing to overwrite an existing custom launcher: {target}')
    lines = []
    for line in source.read_text().splitlines():
        if line == '[Desktop Entry]':
            lines.extend([line, 'X-Spectre-WaylandInput=true'])
            continue
        if line.startswith('Exec=kgx'):
            line = line.replace('Exec=kgx', 'Exec=/run/current-system/sw/bin/env GTK_IM_MODULE=wayland /run/current-system/sw/bin/kgx', 1)
        if line == 'DBusActivatable=true':
            line = 'DBusActivatable=false'
        lines.append(line)
    target.parent.mkdir(parents=True, exist_ok=True)
    temporary = target.with_suffix('.desktop.tmp')
    temporary.write_text('\n'.join(lines) + '\n')
    os.replace(temporary, target)
    print(target)


if __name__ == '__main__':
    main()
