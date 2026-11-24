#!/usr/bin/env python3
"""Measure Sunny RAW10 colour uniformity on two retained flat-field pairs.

This measures one scene and does not certify uniform illumination, calibrate
absolute colour, or validate every lighting/focus trigger in the stock tables.
Run locally as data analysis; it does not compile the phone software.
"""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path

import numpy as np


HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('noise', HERE / 'calibrate-noise.py')
noise = importlib.util.module_from_spec(spec)
spec.loader.exec_module(noise)


def samples(channel):
    h, w = channel.shape
    return np.array([[channel[int(y)-35:int(y)+36, int(x)-35:int(x)+36].mean()
                      for x in np.linspace(35, w-36, 17)]
                     for y in np.linspace(35, h-36, 13)])


def metrics(channels):
    g = (samples(channels[1]) + samples(channels[2])) / 2
    if np.min(g) <= 8:
        raise ValueError('Flat field is too dark for reliable colour ratios')
    result = {}
    for c, name in [(0, 'redGreen'), (3, 'blueGreen')]:
        ratio = samples(channels[c]) / g
        normalized = ratio / ratio[6, 8]
        result[name] = {'normalizedRms': float(np.sqrt(np.mean((normalized-1)**2))),
                        'normalizedGrid': normalized.tolist()}
    return result


def gain_at(table, c, xs, ys):
    x0, y0 = np.minimum(xs.astype(int), 15), np.minimum(ys.astype(int), 11)
    dx, dy = xs-x0, ys-y0
    t = table[c]
    return ((t[y0[:, None], x0]*(1-dx) + t[y0[:, None], x0+1]*dx)*(1-dy[:, None]) +
            (t[y0[:, None]+1, x0]*(1-dx) + t[y0[:, None]+1, x0+1]*dx)*dy[:, None])


def validate(raw_paths, profile_path):
    profile = json.loads(profile_path.read_text())
    if (profile.get('camera') != 'beryllium-imx363-sunny' or
            profile.get('grid') != [17, 13] or
            profile.get('channelOrder') != ['R', 'Gr', 'Gb', 'B']):
        raise ValueError('Unexpected camera or mesh layout')
    table = np.array(profile['candidate']['gains'], dtype=np.float64)
    if table.shape != (4, 221) or not np.all(np.isfinite(table)) or np.any((table < .25) | (table > 8)):
        raise ValueError('Invalid shading mesh')
    table = table.reshape(4, 13, 17)
    pairs = [noise.unpack(p) for p in raw_paths]
    means = [float(frame.mean()) for pair in pairs for frame in pair]
    clipped = float(sum(np.count_nonzero(pair >= 1020) for pair in pairs) /
                    sum(pair.size for pair in pairs))
    if clipped > .001:
        raise ValueError('Clipping prevents reliable flat-field validation')
    averaged = sum(pair.mean(axis=0, dtype=np.float64) for pair in pairs) / len(pairs)
    del pairs
    channels = np.array([averaged[y::2, x::2]-64 for y, x in [(0, 0), (0, 1), (1, 0), (1, 1)]])
    result = {'schema': 1, 'camera': profile['camera'], 'blackLevel': 64,
              'rawSha256': {str(p.name): hashlib.sha256(p.read_bytes()).hexdigest() for p in raw_paths},
              'profileSha256': hashlib.sha256(profile_path.read_bytes()).hexdigest(),
              'frameMeans': means, 'clippedFraction': clipped, 'before': metrics(channels),
              'limitations': 'One wall under one light; illumination is not certified uniform. Ratios are normalized at the centre. Full brightness correction and trigger selection remain unvalidated.'}
    h, w = channels.shape[1:]
    centre_green = (table[1, 6, 8]+table[2, 6, 8])/2
    for mode in ['colour', 'full']:
        corrected = np.empty_like(channels)
        for c, (y, x) in enumerate([(0, 0), (0, 1), (1, 0), (1, 1)]):
            # Match the full-resolution runtime's actual CFA pixel coordinates.
            xs = (np.arange(w)*2+x)*16/4031
            ys = (np.arange(h)*2+y)*12/3023
            gain = gain_at(table, c, xs, ys)
            if mode == 'colour':
                green = (gain_at(table, 1, xs, ys)+gain_at(table, 2, xs, ys))/2
                gain = gain/green*centre_green/table[c, 6, 8]
            corrected[c] = channels[c]*gain
        result[mode] = metrics(corrected)
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('pairs', type=Path, nargs=2)
    parser.add_argument('--profile', type=Path, default=HERE/'imx363-sunny-shading.json')
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    if args.pairs[0].resolve() == args.pairs[1].resolve() or args.pairs[0].name == args.pairs[1].name:
        parser.error('Supply two distinct, distinctly named RAW pairs')
    report = validate(args.pairs, args.profile)
    args.output.write_text(json.dumps(report, indent=2)+'\n')
    print(json.dumps({mode: {c: v['normalizedRms'] for c, v in report[mode].items()}
                      for mode in ['before', 'colour', 'full']}, indent=2))
