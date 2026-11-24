#!/usr/bin/env python3
"""Trace stock IMX363 Chromatix CC13 trees and module EEPROM descriptors.

Run against the owner's extracted binaries on the build host. This preserves mode,
trigger and pointer provenance; it does not imply that libcamera reproduces
Qualcomm's exposure/CCT trigger selection. No phone writes are performed.
"""
import argparse
import hashlib
import json
import math
from pathlib import Path
import struct

class Container:
    def __init__(self, path):
        self.path = Path(path)
        self.data = self.path.read_bytes()
        if self.block(0,20) != b'QTI Chromatix Header':
            raise ValueError('Unexpected container signature')
        if self.ints(28,3) != (len(self.data),1,3):
            raise ValueError('Unsupported size/version')
        table,count = self.ints(152,2)
        sections={}
        for index in range(count):
            kind,offset,size=self.ints(table+index*12,3)
            self.block(offset,size)
            if kind in sections: raise ValueError('Duplicate section')
            sections[kind]=(offset,size)
        self.sections=sections
        offset,size=sections[0]
        if size%64: raise ValueError('Bad symbol table')
        self.symbols={}
        for pos in range(offset,offset+size,64):
            ident=self.ints(pos,1)[0]
            name=self.block(pos+4,48).split(b'\0')[0].decode('ascii')
            mode,relative,length=self.ints(pos+52,3)
            if relative+length>sections[1][1]: raise ValueError('Payload overflow')
            if ident in self.symbols: raise ValueError('Duplicate symbol')
            self.symbols[ident]={'id':ident,'name':name,'mode_id':mode,
                                'file_offset':sections[1][0]+relative,'size':length}
        self.modes={}
        offset,size=sections[2]
        if size%16: raise ValueError('Bad mode table')
        for pos in range(offset,offset+size,16):
            ident,packed,section,parent=self.ints(pos,4)
            if ident in self.modes: raise ValueError('Duplicate mode')
            self.modes[ident]={'id':ident,'packed_selector':packed,
                              'processing_section':section,'parent':parent}
    def block(self,offset,size):
        if offset<0 or size<0 or offset+size>len(self.data): raise ValueError('Bounds violation')
        return self.data[offset:offset+size]
    def ints(self,offset,count):
        return struct.unpack('<'+'I'*count,self.block(offset,count*4))
    def floats(self,offset,count):
        values=struct.unpack('<'+'f'*count,self.block(offset,count*4))
        if not all(math.isfinite(x) for x in values): raise ValueError('Nonfinite float')
        return values
    def mode_path(self,ident):
        path=[];seen=set()
        while True:
            if ident in seen: raise ValueError('Mode cycle')
            seen.add(ident);mode=self.modes[ident];path.append(mode)
            if mode['parent']==ident: return path[::-1]
            ident=mode['parent']
    def cc13(self,ident,level=0,count=None):
        names=['mod_cc13_drc_gain_data','mod_cc13_hdr_aec_data',
               'mod_cc13_led_idx_data','mod_cc13_aec_data','mod_cc13_cct_data']
        sizes=[16,24,12,24,56]
        symbol=self.symbols[ident];size=sizes[level]
        if symbol['name']!=names[level] or symbol['size']%size:
            raise ValueError('Unexpected CC13 child type or length')
        if count is not None and count*size!=symbol['size']: raise ValueError('Child count mismatch')
        records=[]
        for pos in range(symbol['file_offset'],symbol['file_offset']+symbol['size'],size):
            row={'file_offset':pos}
            if level==4:
                values=self.floats(pos,14)
                row.update(cct_bounds=values[:2],matrix=[values[2+i*3:5+i*3] for i in range(3)],offsets=values[11:])
                if values[0]>values[1]: raise ValueError('Reversed CCT bounds')
                if max(abs(sum(r)-1) for r in row['matrix'])>0.001: raise ValueError('Non-neutral matrix')
            else:
                row['trigger_words']=self.ints(pos,size//4-2) if level==2 else self.floats(pos,size//4-2)
                length,child=self.ints(pos+size-8,2)
                if not length: raise ValueError('Empty CC13 region')
                row['child']=self.cc13(child,level+1,length)
            records.append(row)
        return {**symbol,'records':records}
    def report(self):
        result={'source_name':self.path.name,'sha256':hashlib.sha256(self.data).hexdigest(),
                'status':'Decoded pointer topology; stock runtime trigger selection requires separate verification',
                'cc13_roots':[],'module_custom_info':[]}
        for ident,symbol in self.symbols.items():
            if symbol['name'] in ('cc13_ipe','cc13_bps'):
                if symbol['size']!=64: raise ValueError('Unexpected CC13 root size')
                count,child=self.ints(symbol['file_offset']+56,2)
                result['cc13_roots'].append({**symbol,'mode_path':self.mode_path(symbol['mode_id']),
                    'header_words':self.ints(symbol['file_offset'],14),'tree':self.cc13(child,count=count)})
            elif symbol['name']=='customInfo':
                if symbol['size']%12: raise ValueError('Unexpected customInfo')
                for pos in range(symbol['file_offset'],symbol['file_offset']+symbol['size'],12):
                    length,ref,value=self.ints(pos,3);child=self.symbols[ref]
                    if length!=child['size']: raise ValueError('String length mismatch')
                    name=self.block(child['file_offset'],length).rstrip(b'\0').decode('ascii')
                    result['module_custom_info'].append({'name':name,'value':value,'file_offset':pos})
        return result

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('binary',type=Path)
    args=parser.parse_args()
    print(json.dumps(Container(args.binary).report(),indent=2))
