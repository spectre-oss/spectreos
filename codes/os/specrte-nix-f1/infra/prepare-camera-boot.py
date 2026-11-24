#!/usr/bin/env python3
"""Build-host-only packaging of the Sunny autofocus kernel for RAM-only boot tests.

Preserves the owner's backed-up initrd, addresses and command line. No device,
SSH, flashing or installation operations are performed by this tool.
"""
import argparse
import gzip
import hashlib
import importlib.util
import json
from pathlib import Path
import struct
import subprocess
import zlib

ROOT=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location('boot_package',ROOT/'bootloader/package.py')
package=importlib.util.module_from_spec(spec);spec.loader.exec_module(package)

def digest(b): return hashlib.sha256(b).hexdigest()
def kernel_config(compressed):
    image=gzip.decompress(compressed)
    if image[56:60]!=b'ARM\x64' or b'Linux version 7.1.0-rc1 ' not in image:
        raise ValueError('Expected pinned ARM64 Linux 7.1.0-rc1')
    start=image.find(b'IKCFG_ST\x1f\x8b\x08')
    if start<0: raise ValueError('Kernel configuration absent')
    return zlib.decompress(image[start+8:],31).decode()

def config_map(text):
    return {line.split('=')[0]:line for line in text.splitlines() if line.startswith('CONFIG_')} | {
        line.split()[1]:line for line in text.splitlines() if line.startswith('# CONFIG_')}

def prepare(backup,running_config,kernel_dir,out):
    template=backup.read_bytes()
    if len(template)!=64*1024*1024 or struct.unpack_from('<I',template,24)[0]!=0:
        raise ValueError('Expected whole Poco F1 boot-partition backup with no second stage')
    old_kernel,ramdisk,old_dtb,cmdline=package.split_boot(template)
    old_config=kernel_config(old_kernel)
    running=gzip.decompress(running_config.read_bytes()).decode()
    if config_map(old_config)!=config_map(running):
        raise ValueError('Boot backup configuration does not match the running phone')
    kernel=(kernel_dir/'vmlinuz-7.1.0-rc1').read_bytes()
    dtb_path=kernel_dir/'dtbs/qcom/sdm845-xiaomi-beryllium-tianma.dtb'
    dtb=dtb_path.read_bytes()
    new_config=kernel_config(kernel)
    before,after=config_map(old_config),config_map(new_config)
    changes={k:[before.get(k),after.get(k)] for k in before.keys()|after.keys() if before.get(k)!=after.get(k)}
    if changes!={'CONFIG_VIDEO_AK7375':['# CONFIG_VIDEO_AK7375 is not set','CONFIG_VIDEO_AK7375=y']}:
        raise ValueError('Unexpected configuration changes: '+json.dumps(changes))
    def get(node,prop):
        return subprocess.check_output(['fdtget',str(dtb_path),node,prop],text=True).strip()
    actuator='/soc@0/cci@ac4a000/i2c-bus@0/actuator@c'
    sensor='/soc@0/cci@ac4a000/i2c-bus@0/camera@10'
    if get('/','compatible').split()[0]!='xiaomi,beryllium-tianma' or get(actuator,'compatible')!='asahi-kasei,ak7374':
        raise ValueError('Wrong panel or focus actuator')
    if get(sensor,'lens-focus')!=get(actuator,'phandle'):
        raise ValueError('Lens phandle is not connected to the camera')
    get(actuator,'vdd-supply');get(actuator,'vio-supply')
    image=package.wrap_boot(template,kernel+dtb,ramdisk)
    new_kernel,new_ramdisk,new_dtb,new_cmdline=package.split_boot(image)
    if (new_kernel,new_ramdisk,new_dtb,new_cmdline)!=(kernel,ramdisk,dtb,cmdline):
        raise ValueError('Round-trip payload validation failed')
    # Compare every preserved header byte, apart from sizes and legacy SHA-1 ID.
    a=bytearray(template[:4096]);b=bytearray(image[:4096])
    for offset,length in [(8,4),(16,4),(24,4),(40,4),(576,32)]:
        a[offset:offset+length]=b[offset:offset+length]
    if a!=b: raise ValueError('Preserved boot header changed')
    out.mkdir(parents=True,exist_ok=False)
    (out/'boot.img').write_bytes(image)
    # A trimmed, byte-identical original payload is suitable for RAM-only fallback.
    used=4096+package.align(len(old_kernel)+len(old_dtb))+package.align(len(ramdisk))
    fallback=template[:used]
    (out/'previous-boot.img').write_bytes(fallback)
    report={'schema':1,'purpose':'RAM-only autofocus kernel test; no flashing authorized',
        'camera':'beryllium-imx363-sunny','panel':'tianma','configurationChanges':changes,
        'backupSha256':digest(template),'bootSha256':digest(image),'bootBytes':len(image),
        'previousBootSha256':digest(fallback),'preservedInitrdSha256':digest(ramdisk),
        'preservedCommandLine':cmdline,'kernelSha256':digest(kernel),'dtbSha256':digest(dtb),
        'kernelDirectory':str(kernel_dir),'phoneBootTested':False}
    (out/'manifest.json').write_text(json.dumps(report,indent=2)+'\n')
    subprocess.run(['python3',str(ROOT/'scripts/inspect-images.py'),str(out/'boot.img'),
                    '--panel','tianma','--boot-only'],check=True)
    print(json.dumps(report,indent=2))

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('backup',type=Path);p.add_argument('running_config',type=Path)
    p.add_argument('kernel_dir',type=Path);p.add_argument('output',type=Path)
    a=p.parse_args();prepare(a.backup,a.running_config,a.kernel_dir,a.output)
