#!/usr/bin/env python3
"""Decode static node/port graphs from the owner's Titan17x use-case container.

Run on the build host. Record layout is inferred from typed symbol references and
validated across the whole file. This is not a runtime trace or an ISP register
execution order. Unknown flags/properties are retained, not silently interpreted.
"""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path

spec = importlib.util.spec_from_file_location('profile', Path(__file__).with_name('decode-camera-profile.py'))
profile = importlib.util.module_from_spec(spec)
spec.loader.exec_module(profile)


class Graph(profile.Container):
    def symbol(self, ident, name):
        s = self.symbols[ident]
        if s['name'] != name:
            raise ValueError(f'Expected {name}, got {s}')
        return s

    def string(self, length, ident, name):
        s = self.symbol(ident, name)
        if s['size'] != length:
            raise ValueError('String length mismatch')
        raw = self.block(s['file_offset'], length)
        if raw and (raw[-1] != 0 or b'\0' in raw[:-1]):
            raise ValueError('Invalid string terminator')
        return raw.rstrip(b'\0').decode('ascii')

    def records(self, count, ident, name, words):
        s = self.symbol(ident, name)
        if count * words * 4 != s['size']:
            raise ValueError(f'{name} count/size mismatch')
        return [(pos, self.ints(pos, words)) for pos in
                range(s['file_offset'], s['file_offset'] + s['size'], words * 4)]

    def node(self, pos, w):
        props = []
        for offset, p in self.records(w[0], w[1], 'NodeProperty', 4):
            props.append(dict(id=p[0], type=p[1], value=self.string(p[2], p[3], 'NodePropertyValue'), file_offset=offset))
        return dict(name=self.string(w[2], w[3], 'NodeName'), node_id=w[4],
                    instance=self.string(w[5], w[6], 'NodeInstance'), instance_id=w[7],
                    properties=props, file_offset=pos)

    def port(self, w):
        # Count/ref arrays at words 3..6 are preserved in raw_words below.
        return dict(port=self.string(w[0], w[1], 'PortName'), port_id=w[2],
                    node=self.string(w[7], w[8], 'NodeName'), node_id=w[9],
                    instance=self.string(w[10], w[11], 'NodeInstance'), instance_id=w[12],
                    raw_words=w)

    def pipeline(self, pos, w):
        nodes = [self.node(p, n) for p, n in self.records(w[2], w[3], 'Node', 8)]
        links = []
        for offset, link in self.records(w[12], w[13], 'Link', 19):
            dst = [self.port(d) for _, d in self.records(link[15], link[16], 'DstPort', 13)]
            links.append(dict(source=self.port(link[2:15]), destinations=dst,
                              file_offset=offset, raw_words=link))
        known = {(n['node_id'], n['instance_id']): n['name'] for n in nodes}
        warnings = []
        if len(known) != len(nodes):
            warnings.append('Original definition repeats node IDs/instances; all rows retained')
        for link in links:
            for port in [link['source']] + link['destinations']:
                if port['node'] in ('SinkBuffer', 'SinkNoBuffer', 'SourceBuffer'):
                    continue
                if known.get((port['node_id'], port['instance_id'])) != port['node']:
                    warning = f'Endpoint absent or inconsistent in node list: {port["node"]}/{port["instance_id"]}'
                    if warning not in warnings:
                        warnings.append(warning)
        return dict(name=self.string(w[0], w[1], 'PipelineName'), file_offset=pos,
                    nodes=nodes, links=links, raw_words=w, warnings=warnings)

    def graph(self):
        root = self.symbol(1, 'UsecaseDef')
        if root['size'] != 20:
            raise ValueError('Unsupported root')
        header = self.ints(root['file_offset'], 5)
        usecases = []
        for pos, w in self.records(header[3], header[4], 'Usecase', 11):
            pipelines = [self.pipeline(p, v) for p, v in self.records(w[9], w[10], 'Pipeline', 14)]
            usecases.append(dict(name=self.string(w[0], w[1], 'UsecaseName'),
                                 file_offset=pos, raw_words=w, pipelines=pipelines))
        return dict(source=self.path.name, sha256=hashlib.sha256(self.data).hexdigest(),
                    scope='Static definitions, not proof of runtime selection; record layout inferred and validated against typed references',
                    usecases=usecases)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('binary', type=Path)
    args = parser.parse_args()
    print(json.dumps(Graph(args.binary).graph(), indent=2))
