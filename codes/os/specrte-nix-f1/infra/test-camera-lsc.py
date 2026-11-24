#!/usr/bin/env python3
"""Check independent LSC calibration arithmetic and pinned source provenance."""
import importlib.util
from pathlib import Path
import sys
spec=importlib.util.spec_from_file_location('lsc',Path(__file__).with_name('decode-camera-lsc.py'))
lsc=importlib.util.module_from_spec(spec); spec.loader.exec_module(lsc)
def table(values): return [[v]*221 for v in values]
actual=lsc.calibrated_gains(table([2,3,4,5]),table([10,20,30,40]),table([20,30,40,50]))
assert [row[0] for row in actual] == [1,2.5,2.5,4]
assert actual[1] is not actual[2]
for bad in (0,float('nan'),float('inf'),-1):
    invalid=table([1,1,1,1]); invalid[2][100]=bad
    try:lsc.calibrated_gains(table([1]*4),table([1]*4),invalid)
    except ValueError:pass
    else:raise AssertionError('Invalid calibration accepted')
result=lsc.decode(*map(Path,sys.argv[1:4]))
assert len(result['roots'])==4
assert result['candidate']['enabled'] is False
assert result['genericStockEepromLscEnabled'] is False
assert result['candidate']['symbolPath']==[432,433,434,435,436,437]
print('Sunny LSC: 4 source trees, pinned provenance, green averaging and invalid-value rejection passed')
