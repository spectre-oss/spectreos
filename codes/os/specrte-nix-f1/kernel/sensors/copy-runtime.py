#!/usr/bin/env python3
"""Copy selected Nix ELF tools and DT_NEEDED libraries into a small initramfs tree.

Unlike the full Nix closure this excludes manuals, locales and unrelated tools.
It is suitable for this diagnostic, not arbitrary dlopen-based applications.
"""
import argparse
from pathlib import Path
import re
import shutil
import subprocess

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--store', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
p.add_argument('executables', nargs='+')
a = p.parse_args()
seen = set()

def copy(name):
    if not name.startswith('/nix/store/') or '..' in Path(name).parts:
        raise ValueError('Expected an absolute Nix store path: ' + name)
    if name in seen:
        return
    seen.add(name)
    src = a.store / name.lstrip('/')
    dest = a.output / name.lstrip('/')
    dest.parent.mkdir(parents=True, exist_ok=True)
    if dest.exists():
        if dest.read_bytes() != src.read_bytes():
            raise ValueError('Conflicting runtime file: ' + name)
    else:
        shutil.copy2(src.resolve(), dest)
    text = subprocess.check_output(['readelf', '-d', str(src)], text=True)
    paths = [str(Path(name).parent)]
    for rpath in re.findall(r'(?:RUNPATH|RPATH).*?\[(.*?)\]', text):
        paths.extend(rpath.split(':'))
    headers = subprocess.check_output(['readelf', '-l', str(src)], text=True)
    for interp in re.findall(r'Requesting program interpreter: (.*?)\]', headers):
        copy(interp)
    for lib in re.findall(r'NEEDED.*?\[(.*?)\]', text):
        for directory in paths:
            candidate = directory.replace('$ORIGIN', str(Path(name).parent)) + '/' + lib
            if (a.store / candidate.lstrip('/')).exists():
                copy(candidate)
                break
        else:
            raise ValueError(f'Unresolved dependency {lib} for {name}')

for executable in a.executables:
    copy(executable)
print(f'Validated/copied {len(seen)} ELF runtime files')
