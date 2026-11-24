#!/usr/bin/env python3
"""Decode this Sunny IMX363 module using its pinned stock descriptor.

Read-only analysis. Run on build host against the original module binary and an
8192-byte read. No raw EEPROM contents/serial number are included in the report.
Addresses are from REA/Ghidra camera.qcom.so with the 0x10000 image base.
"""
import argparse
import hashlib
import importlib.util
import json
import math
from pathlib import Path
import struct

spec = importlib.util.spec_from_file_location('profile', Path(__file__).with_name('decode-camera-profile.py'))
profile = importlib.util.module_from_spec(spec)
spec.loader.exec_module(profile)
MODULE_SHA256 = 'd3fe8a1f0f9ccc3cc9dec0678672a8b96f866a3235453d76a959005dbd11396a'
HAL_SHA256 = '0c0d7db395703421728f01fde68e28130a36faf84b18b16571a540ce3474f034'

class Cursor:
    def __init__(self, data, offset=0): self.data, self.offset = data, offset
    def read(self, fmt):
        n = struct.calcsize('<'+fmt)
        if self.offset+n > len(self.data): raise ValueError('Truncated descriptor')
        value = struct.unpack_from('<'+fmt, self.data, self.offset); self.offset += n
        return value[0] if len(value)==1 else value
    def memory(self):
        address, mask = self.read('HI')
        return {'address': address, 'mask': mask}

def decode(module, eeprom):
    c = profile.Container(module)
    if hashlib.sha256(c.data).hexdigest() != MODULE_SHA256:
        raise ValueError('Unsupported module descriptor; this decoder is Sunny-specific')
    b = Path(eeprom).read_bytes()
    if len(b) != 8192 or b[1] != 1: raise ValueError('Expected 8192-byte Sunny vendor ID 1 EEPROM')
    def symbol(ident, name):
        s=c.symbols[ident]
        if s['name'] != name: raise ValueError('Unexpected symbol')
        return c.block(s['file_offset'],s['size'])
    def integer(info, endian=0):
        mask=info['mask']; address=info['address']
        if mask==0:return 0
        # Only the contiguous low-bit masks present in this pinned descriptor.
        if mask & (mask+1):raise ValueError('Non-contiguous mask')
        length=(mask.bit_length()+7)//8
        if address+length>len(b) or endian not in (0,1):raise ValueError('Invalid EEPROM address/endian')
        return int.from_bytes(b[address:address+length], 'big' if endian==0 else 'little') & mask
    # Loader chain: version (12), slave info (38), read settings (8), AF.
    cur=Cursor(symbol(5,'EEPROMDriverData'),58)
    af_enabled,af_endian=cur.read('II')
    af_base=[cur.memory() for _ in range(4)]; cur.read('I')
    af_limits=[cur.memory() for _ in range(4)]
    af_margins=cur.read('ff'); af_modes=cur.read('II')
    af_adjust=[cur.memory() for _ in range(2)]; af_constants=cur.read('hh')
    if af_modes!=(0,0) or any(x['mask'] for x in af_limits+af_adjust) or af_constants!=(0,0):
        raise ValueError('Unsupported AF adjustment branch')
    wb_enabled,wb_endian,wb_ratio,wb_count,wb_ref=cur.read('IIIII')
    green_flag=cur.memory(); wb_validity=cur.memory(); qvalue=cur.read('f'); invert=cur.read('I')
    if (wb_count,wb_ref,wb_ratio)!=(1,2065,1) or not math.isfinite(qvalue) or qvalue<=0:
        raise ValueError('Unsupported white-balance encoding')
    light=Cursor(symbol(wb_ref,'lightInfo')); light_enum=light.read('I')
    wb_fields=[light.memory() for _ in range(7)]
    ratios=[integer(x,wb_endian)/qvalue for x in wb_fields[4:]]
    if integer(green_flag,wb_endian) and invert:
        if ratios[2]<=0:raise ValueError('Invalid green ratio')
        ratios[2]=1/ratios[2]
    if not all(0.1<=x<=10 for x in ratios):raise ValueError('Invalid WB calibration')
    lsc_enabled,lsc_endian,lsc_count,lsc_ref=cur.read('IIII')
    count,*steps=cur.read('HHHHH')
    if (lsc_count,lsc_ref,count,steps)!=(1,2066,221,[8,8,8,8]):raise ValueError('Unexpected LSC layout')
    light=Cursor(symbol(lsc_ref,'lightInfo')); lsc_light=light.read('I')
    fields=[light.memory() for _ in range(8)]
    channels=[]
    for channel in range(4):
        hi,lo=fields[2*channel:2*channel+2]
        values=[]
        for i in range(count):
            h={**hi,'address':hi['address']+i*steps[channel]}
            l={**lo,'address':lo['address']+i*steps[channel]}
            values.append((integer(h)<<8)|integer(l))
        if not all(0<x<=1023 for x in values):raise ValueError('Invalid shading sample')
        channels.append(values)
    name=symbol(2044,'actuatorName').rstrip(b'\0').decode('ascii')
    reg=struct.unpack('<9I',symbol(2047,'registerParam'))
    if name!='beryllium_imx363_ak7374_sunny' or reg!=(1,2,0,0,1,0,0,0,6):
        raise ValueError('Unexpected actuator register encoding')
    macro,infinity=[integer(x,af_endian) for x in af_base[:2]]
    if not 0<infinity<macro<1024:raise ValueError('Invalid calibrated focus endpoints')
    return {
        'schema':1,'camera':'beryllium-imx363-sunny','moduleSha256':MODULE_SHA256,
        'eepromSha256':hashlib.sha256(b).hexdigest(),'halSha256':HAL_SHA256,
        'actuator':{'name':name,'linuxAddress':12,'positionRegister':0,'positionBits':10,'positionShift':6,
                    'controlRegister':2,'activeValue':0,'enabled':bool(af_enabled),
                    'infinityCode':infinity,'macroCode':macro,'stockMargins':af_margins},
        'whiteBalance':{'enabled':bool(wb_enabled),'lightEnum':light_enum,'rOverG':ratios[0],
                        'bOverG':ratios[1],'grOverGb':ratios[2],
                        'validityByte':integer(wb_validity,wb_endian),
                        'note':'Module calibration ratios, not scene AWB gains. Golden reference mapping remains separate.'},
        'lensShading':{'genericStockEnabled':bool(lsc_enabled),'lightEnum':lsc_light,'grid':[17,13],
                       'storedChannels':channels,'note':'Raw measured samples, NOT correction gains. Channel semantics, golden-reference scaling and validation are required before application.'},
        'evidence':{'AF':'0x27552c','WB':'0x2757e8','LSC':'0x275bcc','integer':'0x277a8c',
                    'memoryLoader':'0xada48','AFLoader':'0xadab0','WBLoader':'0xadef4','LSCLoader':'0xae2b8','actuatorMove':'0x2619b4'},
        'limitations':['No EEPROM checksum scheme has yet been recovered; two matching reads establish repeatability only.',
                       'Factory scene-dependent AWB and lens correction are not reconstructed by decoding these fields.']}

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('module',type=Path);p.add_argument('eeprom',type=Path);p.add_argument('--output',type=Path)
    args=p.parse_args();result=decode(args.module,args.eeprom);text=json.dumps(result,indent=2)+'\n'
    if args.output:args.output.write_text(text)
    else:print(text,end='')
