#!/usr/bin/env python3
"""Build a RAM-only sensor diagnostic; never accesses the phone."""
import argparse, gzip, hashlib, importlib.util, json, os, shutil, stat, subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
def module(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m); return m
boot = module('boot', ROOT / 'bootloader/package.py')
archive = module('cpio', ROOT / 'kernel/fingerprint/diagnostics/stock-kernel/pack.py')
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--store', type=Path, required=True)
p.add_argument('--firmware', required=True)
p.add_argument('--rpc', required=True)
p.add_argument('--busybox', required=True)
p.add_argument('--dtc', required=True)
p.add_argument('--loader', required=True)
p.add_argument('--template', type=Path, required=True)
p.add_argument('--kernel', type=Path, help='Optional diagnostic Image.gz')
p.add_argument('--extra-root', type=Path)
p.add_argument('--dtb', type=Path, help='Device tree from the matching test kernel')
p.add_argument('--init', type=Path, help='Alternative RAM diagnostic init script')
p.add_argument('--compare-client-ids', action='store_true', help='Diagnostic only: allow a second SDSP session to compare client IDs 1 and 2')
p.add_argument('--output', type=Path, required=True)
a=p.parse_args(); a.output.mkdir(parents=True, exist_ok=False)
stage=a.output/'root'; stage.mkdir()
def physical(s): return a.store / s.lstrip('/')
for d in ['bin','dev','proc','sys','config','run','logs','opt','opt/firmware','opt/qcom']:
    (stage/d).mkdir(parents=True,exist_ok=True)
shutil.copy2(physical(a.busybox)/'bin/busybox', stage/'bin/busybox')
shutil.copy2(a.init or ROOT/'kernel/sensors/init',stage/'init')
(stage/'init').chmod(0o755)
closure=subprocess.check_output(['nix-store','--store',str(a.store),'--query','--requisites',a.rpc],text=True).splitlines()
for s in closure:
    shutil.copytree(physical(s),stage/s.lstrip('/'),symlinks=True)
if a.extra_root:
    
    for src in a.extra_root.rglob('*'):
        dest=stage/src.relative_to(a.extra_root)
        if src.is_file() and not dest.exists():
            dest.parent.mkdir(parents=True,exist_ok=True); shutil.copy2(src,dest)
os.symlink(a.rpc+'/bin/hexagonrpcd',stage/'opt/hexagonrpcd')
fw=physical(a.firmware)
shutil.copytree(fw/'share/qcom/sdm845/Xiaomi/beryllium',stage/'opt/qcom',dirs_exist_ok=True,symlinks=True)
# Use an ephemeral registry; never write calibration back to Android persist.
registry=stage/'opt/qcom/sensors/registry'
if registry.is_symlink(): registry.unlink()
registry.mkdir(exist_ok=True)
if not (stage/'opt/qcom/socinfo').exists():
    os.symlink('/sys/devices/soc0',stage/'opt/qcom/socinfo')
name='qcom/sdm845/Xiaomi/beryllium/slpi.mbn'
(stage/'opt/firmware'/name).parent.mkdir(parents=True,exist_ok=True)
shutil.copy2(fw/'lib/firmware'/name,stage/'opt/firmware'/name)
for jsn in (fw/'lib/firmware'/name).parent.glob('slpi*.jsn'):
    shutil.copy2(jsn,(stage/'opt/firmware'/name).parent/jsn.name)
kernel,_,dtb,_=boot.split_boot(a.template.read_bytes())
if a.kernel:
    kernel = a.kernel.read_bytes()
    if gzip.decompress(kernel)[56:60] != b'ARM\x64':
        raise ValueError('Diagnostic kernel is not ARM64')
if a.dtb:
    dtb = a.dtb.read_bytes()
    if dtb[:4] != bytes.fromhex('d00dfeed'):
        raise ValueError('Invalid device tree magic')
dtbpath=a.output/'boot.dtb';dtbpath.write_bytes(dtb)
loader=physical(a.loader)
dtc=physical(a.dtc)
subprocess.run([str(loader),'--library-path',str(dtc/'lib')+':'+str(loader.parent),str(dtc/'bin/fdtput'),'-t','s',str(dtbpath),'/soc@0/remoteproc@5c00000','status','okay'],check=True)
if a.compare_client_ids:
    subprocess.run([str(loader),'--library-path',str(dtc/'lib')+':'+str(loader.parent),str(dtc/'bin/fdtput'),'-t','i',str(dtbpath),'/soc@0/remoteproc@5c00000/glink-edge/fastrpc/compute-cb@0','qcom,nsessions','2'],check=True)
    (stage/'opt/compare-client-ids').touch()
files=[]
for path in sorted(stage.rglob('*')):
    mode=path.lstat().st_mode
    data=os.readlink(path).encode() if path.is_symlink() else b'' if path.is_dir() else path.read_bytes()
    files.append((str(path.relative_to(stage)),data,mode))
initrd=gzip.compress(archive.cpio(files),mtime=0)
header=bytearray(a.template.read_bytes()[:4096])
cmd=b'console=tty0 rdinit=/init panic=5'
header[64:576]=cmd+bytes(512-len(cmd));header[608:1632]=bytes(1024)
image=boot.wrap_boot(header,kernel+dtbpath.read_bytes(),initrd)
(a.output/'boot.img').write_bytes(image)
manifest={'purpose':'RAM-only hardware diagnostic; no partition mounts/writes','bootSha256':hashlib.sha256(image).hexdigest(),'kernelSha256':hashlib.sha256(kernel).hexdigest(),'dtbSha256':hashlib.sha256(dtbpath.read_bytes()).hexdigest(),'bytes':len(image),'template':str(a.template),'firmware':a.firmware,'rpc':a.rpc,'tested':False}
(a.output/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n');print(json.dumps(manifest))
