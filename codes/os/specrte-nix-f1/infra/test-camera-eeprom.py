#!/usr/bin/env python3
"""Run on build host against original module and privately retained EEPROM."""
import importlib.util
from pathlib import Path
import sys,tempfile
s=importlib.util.spec_from_file_location('eeprom',Path(__file__).with_name('decode-camera-eeprom.py'))
m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
module,eeprom=map(Path,sys.argv[1:]);r=m.decode(module,eeprom)
assert r['actuator']['positionShift']==6
assert r['actuator']['infinityCode']==370 and r['actuator']['macroCode']==859
assert r['whiteBalance']['rOverG']==548/1024
assert r['whiteBalance']['bOverG']==682/1024
assert r['whiteBalance']['grOverGb']==1026/1024
assert r['lensShading']['genericStockEnabled'] is False
assert len(r['lensShading']['storedChannels'])==4
assert all(len(c)==221 for c in r['lensShading']['storedChannels'])
with tempfile.TemporaryDirectory() as t:
 p=Path(t)/'eeprom.bin';data=eeprom.read_bytes()
 def reject(blob):
  p.write_bytes(blob)
  try:m.decode(module,p)
  except ValueError:return
  raise AssertionError('Corrupt EEPROM accepted')
 reject(data[:-1]);d=bytearray(data);d[1]=6;reject(d)
 d=bytearray(data);d[0x44:0x48]=b'\0'*4;reject(d)
 d=bytearray(data);d[0x5a:0x60]=b'\0'*6;reject(d)
 d=bytearray(data);d[0x87:0x87+8]=b'\0'*8;reject(d)
print('PASS: stock AF/WB/LSC descriptor, real module data, truncated/wrong module/invalid AF, WB and LSC rejection')
