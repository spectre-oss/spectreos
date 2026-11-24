#!/usr/bin/env python3
"""Extract Sunny LSC34 tuning and a deliberately disabled calibration candidate.

Run on build host with the original tuning, module descriptor and private EEPROM.
This reconstructs table arithmetic, not Qualcomm trigger selection or geometry.
The owner's generic EEPROM LSC-enable flag is false; do not call this the stock
runtime result or enable it without an independent flat-field validation.
"""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path

def module(name):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(name+'.py'))
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result

eeprom = module('decode-camera-eeprom')
TUNING_SHA256 = '554d5195774fec8b07166cf83ae37e9a0bca952b01a993f98e73c68e5e174480'
CHANNELS = ['R', 'Gr', 'Gb', 'B']
POINTS = 17 * 13

def calibrated_gains(base, golden, measured):
    if any(len(table) != 4 or any(len(channel) != POINTS for channel in table)
           for table in (base, golden, measured)):
        raise ValueError('Expected four 17x13 tables')
    import math
    if any(not math.isfinite(x) or x <= 0 for table in (base, golden, measured)
           for channel in table for x in channel):
        raise ValueError('Invalid shading sample')
    result = [[base[c][i] * golden[c][i] / measured[c][i] for i in range(POINTS)]
              for c in range(4)]
    # LSC34 InterpolateAndCalibrate explicitly gives both green planes the mean.
    green = [(a+b)/2 for a,b in zip(result[1], result[2])]
    result[1] = green.copy(); result[2] = green.copy()
    if any(not 0.25 <= x <= 8 for channel in result for x in channel):
        raise ValueError('Candidate exceeds the conservative software gain bounds')
    return result

def decode(tuning, descriptor, unit):
    c = eeprom.profile.Container(tuning)
    if hashlib.sha256(c.data).hexdigest() != TUNING_SHA256:
        raise ValueError('Unsupported Sunny tuning binary')
    calibration = eeprom.decode(descriptor, unit)
    def tables(ident, golden=False):
        s = c.symbols[ident]
        name = 'mod_lsc34_golden_cct_data' if golden else 'mod_lsc34_cct_data'
        if s['name'] != name or s['size'] % 3544:
            raise ValueError('Invalid LSC leaf')
        rows = []
        for pos in range(s['file_offset'], s['file_offset']+s['size'], 3544):
            values = c.floats(pos, 886)
            if values[0] > values[1] or any(x <= 0 for x in values[2:]):
                raise ValueError('Invalid LSC bounds or table')
            rows.append({'fileOffset':pos, 'cctBounds':values[:2],
                         'channels':[values[2+i*POINTS:2+(i+1)*POINTS] for i in range(4)]})
        return {**s, 'records':rows}
    def tree(ident, count, level=0):
        names = ['lens_position','drc_gain','hdr_aec','led_idx','aec']
        sizes = [16,16,24,12,24]
        if level == 5:
            leaf = tables(ident)
            if len(leaf['records']) != count: raise ValueError('LSC leaf count mismatch')
            return leaf
        s = c.symbols[ident]; size = sizes[level]
        if s['name'] != 'mod_lsc34_'+names[level]+'_data' or count <= 0 or s['size'] != size*count:
            raise ValueError('Invalid LSC tree layout')
        rows = []
        for pos in range(s['file_offset'],s['file_offset']+s['size'],size):
            n, child = c.ints(pos+size-8,2)
            values = c.ints(pos,size//4-2) if level == 3 else c.floats(pos,size//4-2)
            rows.append({'fileOffset':pos,'triggerWords':values,'child':tree(child,n,level+1)})
        return {**s,'records':rows}
    roots = []
    for s in c.symbols.values():
        if s['name'] not in ('lsc34_bps','lsc34_ife'): continue
        if s['size'] != 72: raise ValueError('Invalid LSC root')
        count, child, golden_count, golden_child = c.ints(s['file_offset']+56,4)
        golden = tables(golden_child,True)
        if len(golden['records']) != golden_count: raise ValueError('Golden count mismatch')
        roots.append({**s,'headerWords':c.ints(s['file_offset'],14),
                      'modePath':c.mode_path(s['mode_id']), 'tree':tree(child,count),
                      'golden':golden})
    root = next(x for x in roots if x['id'] == 9)
    leaf = root['tree']
    path = [leaf['id']]
    while 'child' in leaf['records'][0]:
        leaf = leaf['records'][0]['child']; path.append(leaf['id'])
    if path != [432,433,434,435,436,437] or root['golden']['id'] != 441:
        raise ValueError('Unexpected reference branch')
    gains = calibrated_gains(leaf['records'][0]['channels'],
                             root['golden']['records'][0]['channels'],
                             calibration['lensShading']['storedChannels'])
    return {'schema':1,'camera':'beryllium-imx363-sunny','tuningSha256':TUNING_SHA256,
            'moduleSha256':calibration['moduleSha256'],'eepromSha256':calibration['eepromSha256'],
            'halSha256':calibration['halSha256'],'grid':[17,13],'channelOrder':CHANNELS,
            'roots':roots,'candidate':{'enabled':False,'root':9,'symbolPath':path,
                'goldenSymbol':441,'gains':gains,
                'range':[[min(v),max(v)] for v in gains],
                'formula':'base gain * golden measured response / module measured response; average Gr/Gb'},
            'genericStockEepromLscEnabled':calibration['lensShading']['genericStockEnabled'],
            'evidence':{'tableLoader':'0xf6df8','calibrate':'0x25a1f4','apply':'0x259d70'},
            'limitations':['Generic stock EEPROM LSC enable flag is false. Candidate deliberately differs from that disabled path.',
                'Lens/DRC/HDR/LED/exposure/CCT trigger selection is preserved as evidence, not reproduced.',
                'Mesh placement, optical alignment and sensor-mode geometry require flat-field validation.',
                'No chromatic-aberration displacement model has been recovered.']}

def write_candidate(result, profile_path, header_path):
    selected={k:v for k,v in result.items() if k != 'roots'}
    selected['runtimeScope']='Colour-only correction is separately enabled by the phone launcher after wall validation; full brightness correction remains opt-in.'
    profile_path.write_text(json.dumps(selected,indent=2)+'\n')
    channels=[]
    for channel in result['candidate']['gains']:
        channels.append('    {{'+','.join(f'{v:.9f}f' for v in channel)+'}}')
    header_path.write_text('// Generated by infra/decode-camera-lsc.py; see imx363-sunny-shading.json.\n'
        '#pragma once\n#include <array>\nnamespace spectre {\n'
        'using ShadingTable = std::array<std::array<float,221>,4>;\n'
        'inline constexpr ShadingTable sunnyShading{{\n'+',\n'.join(channels)+'\n}};\n}\n')

if __name__ == '__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('tuning',type=Path); p.add_argument('descriptor',type=Path); p.add_argument('eeprom',type=Path)
    p.add_argument('--output',type=Path,required=True)
    p.add_argument('--profile',type=Path);p.add_argument('--header',type=Path)
    a=p.parse_args()
    if bool(a.profile)!=bool(a.header):p.error('--profile and --header must be used together')
    result=decode(a.tuning,a.descriptor,a.eeprom)
    a.output.write_text(json.dumps(result,indent=2)+'\n')
    if a.profile:write_candidate(result,a.profile,a.header)
