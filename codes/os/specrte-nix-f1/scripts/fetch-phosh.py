#!/usr/bin/env python3
"""Fetch the pinned upstream Phosh checkout without overwriting local changes."""
import json
from pathlib import Path
import subprocess

root = Path(__file__).resolve().parents[1]
source = json.loads((root / 'sources.json').read_text())['phosh']
target = root / 'phosh'
if not target.exists():
    subprocess.run(['git', 'clone', '--depth', '1', '--branch', source['tag'], source['url'], str(target)], check=True)
revision = subprocess.check_output(['git','-C',str(target),'rev-parse','HEAD'],text=True).strip()
if revision != source['revision']:
    raise SystemExit('Phosh checkout differs from sources.json; left unchanged')
print(target)
