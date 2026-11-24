#!/usr/bin/env python3
"""Package four separate offline Bhoo application integration fixtures."""
from pathlib import Path
import json,zipfile
ROOT=Path(__file__).resolve().parents[1];SITE=ROOT/'codes/apps/bhoo-web/site'
APPS={'fossify':('Fossify Messages','Search, conversations, contact picker and local sending.'),'deltachat':('Delta Chat','Search, chat list, conversations and local sending.'),'thunderbird':('Thunderbird','Inbox, starred/archive folders, reading, replying and composing.'),'sunflower':('Sunflower','Plant catalog, search, plant details and a saved garden.')}
def main():
 out=ROOT/'context/dist';out.mkdir(parents=True,exist_ok=True)
 for app,(name,features) in APPS.items():
  description=f'Unofficial Bhoo integration demo. {features} Original XML row templates, Rust controllers and local persistence. Fictional offline data; not a complete Android port. Source and licenses included.'
  files={n:(SITE/n).read_bytes() for n in ['bhoo.wasm','app-test.js','canvas-painter.js','icon.png','mark.svg','source.zip','licenses/GPL-3.0.txt','licenses/Apache-2.0.txt','licenses/NOTICES.md']}
  files['index.html']=(SITE/'app-test.html').read_bytes();files['app-config.json']=json.dumps({'app':app}).encode();files['manifest.toml']=f'name = {json.dumps(name+" · Bhoo test")}\ndescription = {json.dumps(description)}\n'.encode()
  files['README.md']=(ROOT/'codes/libs/ui/libbhoo/docs/android-app-tests.md').read_bytes()
  fixtures=ROOT/'codes/apps/bhoo-web/android-app-tests'/app
  for p in fixtures.iterdir():
   if p.is_file():files['reference/'+p.name]=p.read_bytes()
  path=out/f'bhoo-{app}-system-test.xdc'
  with zipfile.ZipFile(path,'w',compression=zipfile.ZIP_DEFLATED,compresslevel=9) as z:
   for name,content in sorted(files.items()):
    entry=zipfile.ZipInfo(name,date_time=(2026,1,1,0,0,0));entry.external_attr=0o100644<<16;z.writestr(entry,content,compress_type=zipfile.ZIP_DEFLATED,compresslevel=9)
  with zipfile.ZipFile(path) as z:assert z.testzip() is None
  print(f'{path}: {path.stat().st_size:,} bytes')
if __name__=='__main__':main()
