#!/usr/bin/env python3
"""Build only: replace stock-derived boot ramdisk; preserve kernel/DTBs/layout."""
import argparse, gzip, hashlib, json, struct, zipfile
from pathlib import Path

def align(n, a=4096): return (n+a-1)//a*a

def cpio(files):
    out=bytearray()
    for ino,(name,data,mode) in enumerate(files+[("TRAILER!!!",b"",0)],1):
        name=name.encode()+b'\0'
        fields=(ino,mode,0,0,1,0,len(data),0,0,0,0,len(name),0)
        out+=b'070701'+b''.join(f'{x:08x}'.encode() for x in fields)+name
        out+=bytes(align(len(out),4)-len(out));out+=data
        out+=bytes(align(len(out),4)-len(out))
    return bytes(out)

def main():
    p=argparse.ArgumentParser(description=__doc__)
    source=p.add_mutually_exclusive_group(required=True)
    source.add_argument('--rom',type=Path);source.add_argument('--boot-image',type=Path)
    p.add_argument('--reboot-sentinel',action='store_true',help='Package only syscall-only init; no probe or firmware')
    p.add_argument('--init',type=Path,required=True)
    p.add_argument('--worker',type=Path,help='Diagnostic child for syscall-only supervisor')
    p.add_argument('--probe',type=Path);p.add_argument('--gfber',type=Path)
    p.add_argument('--cmnlib-dir',type=Path,help='Include only cmnlib.mdt and cmnlib.b00 through b05')
    p.add_argument('--output',type=Path,required=True);a=p.parse_args()
    if a.reboot_sentinel:
        if a.probe or a.gfber or a.cmnlib_dir or a.worker:p.error('Sentinel must not include probe or firmware')
    elif not a.probe:p.error('--probe is required except in sentinel mode')
    if a.rom:
        with zipfile.ZipFile(a.rom) as z: original=z.read('boot.img')
    else:original=a.boot_image.read_bytes()
    if original[:8]!=b'ANDROID!':raise ValueError('Android boot magic missing')
    ks,rs,ss,page,version=(struct.unpack_from('<I',original,o)[0] for o in (8,16,24,36,40))
    if page!=4096 or version not in (0,1) or ss:raise ValueError('Unsupported boot layout')
    header=bytearray(original[:page]);kernel=original[page:page+ks]
    original_cmdline=(original[64:576].split(b'\0')[0]+original[608:1632].split(b'\0')[0]).decode()
    if any(t=='skip_initramfs' or t.startswith(('rdinit=','init=','rootfstype=','panic=')) for t in original_cmdline.split()):
        raise ValueError('Unexpected existing init/root safety flags')
    # If the ramdisk is missing/corrupt, prohibit mounting any disk root and
    # prohibit fallback to Android init. A pre-PID1 failure hangs safely.
    safety='rdinit=/init init=/diagnostic-never-run-root rootfstype=diagnostic-no-rootfs panic=0'
    if a.reboot_sentinel or a.worker:safety=safety.replace('panic=0','panic=5')
    command_line=original_cmdline+' '+safety
    encoded=command_line.encode()
    if len(encoded)>=1536:raise ValueError('Command line exceeds header capacity')
    header[64:576]=encoded[:512].ljust(512,b'\0')
    header[608:1632]=encoded[512:].ljust(1024,b'\0')
    if len(kernel)!=ks or kernel[:3]!=b'\x1f\x8b\x08':raise ValueError('Invalid kernel')
    files=[]
    entries=[('init',a.init)]
    if a.probe:entries.append(('probe',a.probe))
    if a.worker:entries.append(('diagnostic',a.worker))
    for name,path in entries:
        data=path.read_bytes()
        if data[:5]!=b'\x7fELF\x02' or struct.unpack_from('<H',data,18)[0]!=183:raise ValueError(f'{name}: not AArch64 ELF')
        phoff=struct.unpack_from('<Q',data,32)[0];ents,count=struct.unpack_from('<HH',data,54)
        if any(struct.unpack_from('<I',data,phoff+i*ents)[0] in (2,3) for i in range(count)):raise ValueError(f'{name}: must be static without dynamic interpreter')
        files.append((name,data,0o100700))
    if a.gfber:files.append(('gfber.img',a.gfber.read_bytes(),0o100400))
    if a.cmnlib_dir:
        files.extend([('lib',b'',0o040755),('lib/firmware',b'',0o040755)])
        for name in ['cmnlib.mdt']+[f'cmnlib.b{i:02}' for i in range(6)]:
            files.append(('lib/firmware/'+name,(a.cmnlib_dir/name).read_bytes(),0o100400))
    ramdisk=gzip.compress(cpio(files),mtime=0)
    struct.pack_into('<I',header,16,len(ramdisk))
    recovery=b''
    if version==1:
        size,offset,hsize=struct.unpack_from('<IQI',header,1632)
        if hsize!=1648:raise ValueError('Unexpected v1 header size')
        if size:
            recovery=original[offset:offset+size]
            if len(recovery)!=size:raise ValueError('Truncated recovery DTBO')
            struct.pack_into('<Q',header,1636,page+align(ks)+align(len(ramdisk)))
    digest=hashlib.sha1()
    for blob in (kernel,ramdisk,b''):
        digest.update(blob);digest.update(struct.pack('<I',len(blob)))
    if version==1:
        digest.update(recovery);digest.update(struct.pack('<I',len(recovery)))
    header[576:608]=digest.digest()+bytes(12)
    image=bytes(header)+kernel+bytes(align(ks)-ks)+ramdisk+bytes(align(len(ramdisk))-len(ramdisk))
    if recovery:image+=recovery+bytes(align(len(recovery))-len(recovery))
    if len(image)>64*1024*1024:raise ValueError('Boot image too large')
    if image[page:page+ks]!=kernel:raise AssertionError('Kernel changed')
    a.output.mkdir(parents=True,exist_ok=True);(a.output/'boot.img').write_bytes(image)
    info={'purpose':'temporary minimal stock-kernel diagnostic; never flashed','testedOnPhone':False,'headerVersion':version,'originalHeaderCmdline':original_cmdline,'headerCmdline':command_line,'headerCmdlineAdded':safety,'preInitFailurePolicy':'No disk-root filesystem type allowed; no default init fallback; panic hangs for manual recovery','automaticBootloaderRestartSeconds':180,'http':'http://172.16.42.1:8080/','originalKernelSha256':hashlib.sha256(kernel).hexdigest(),'originalBootSha256':hashlib.sha256(original).hexdigest(),'bootSha256':hashlib.sha256(image).hexdigest(),'ramdiskFiles':[n for n,_,_ in files],'recoveryDtboBytes':len(recovery),'bootBytes':len(image)}
    if a.worker:
        info.update(automaticBootloaderRestartSeconds=90,preInitFailurePolicy='No disk root or fallback init; panic restarts after 5 seconds; syscall-only PID1 supervises diagnostic child')
    if a.reboot_sentinel:
        info.update(purpose='temporary syscall-only early-userspace reboot sentinel; never flashed',preInitFailurePolicy='Disk root and fallback init prohibited; panic reboots after 5 seconds; hangs may require manual recovery',automaticBootloaderRestartSeconds=None,http=None,expectedAction='PID1 immediately requests bootloader reboot')
    (a.output/'manifest.json').write_text(json.dumps(info,indent=2)+'\n');print(json.dumps(info,indent=2))
if __name__=='__main__':main()
