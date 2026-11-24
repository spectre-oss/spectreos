#!/usr/bin/env python3
"""Recover the pinned MIUI ARM32 CHI compiled ZSL graph. Run on build host.

Addresses/record sizes were traced with REA/Ghidra. This extractor deliberately
rejects other binaries. It does not execute Android code or trace live requests.
Ghidra addresses in the report use image base 0x10000; ELF VAs below do not.
"""
import argparse
import hashlib
import json
from pathlib import Path
import struct

SHA256 = '25643fc83f3a300e5901edb83cd20644bc360b23a87d5a647f7b9b6c6e5d2ad6'
NAMES = {0:'Sensor', 1:'StatsProcessing', 2:'SinkBuffer', 3:'SinkNoBuffer',
         5:'AutoFocus', 6:'JPEGAggregator', 8:'FDManager', 9:'StatsParse',
         255:'ChiNode', 65536:'IFE', 65537:'JPEG', 65538:'IPE', 65539:'BPS',
         65540:'FDHw', 65541:'LRME', 65542:'RANSAC'}


class Binary:
    def __init__(self, path):
        self.data = Path(path).read_bytes()
        if hashlib.sha256(self.data).hexdigest() != SHA256:
            raise ValueError('Only the verified original MIUI CHI binary is supported')
        phoff = struct.unpack_from('<I', self.data, 28)[0]
        size, count = struct.unpack_from('<HH', self.data, 42)
        self.segments = [struct.unpack_from('<8I', self.data, phoff + i*size) for i in range(count)]

    def offset(self, va, length):
        if not va:
            raise ValueError('Null pointer')
        for kind, offset, address, _, size, _, _, _ in self.segments:
            if kind == 1 and address <= va and va + length <= address + size:
                return offset + va - address
        raise ValueError(f'Unmapped VA {va:#x}, {length} bytes')

    def ints(self, va, count):
        return struct.unpack_from('<' + 'I'*count, self.data, self.offset(va, count*4))

    def string(self, va):
        result = bytearray()
        for i in range(256):
            value = self.data[self.offset(va+i, 1)]
            if not value:
                return result.decode('ascii')
            result.append(value)
        raise ValueError('Unterminated string')

    def endpoint(self, words):
        return dict(node_id=words[0], node=NAMES.get(words[0], 'Unknown'),
                    instance=words[1], port=words[2], source_type=words[3])

    def graph(self):
        # BuildUsecase's DAT_000ab01c at Ghidra image base 0x10000.
        root = self.ints(0x9b01c, 1)[0]
        usecase = self.ints(root, 6)
        if self.string(usecase[0]) != 'UsecaseZSL' or usecase[4] != 36:
            raise ValueError('Unexpected compiled usecase')
        pipelines = []
        for i in range(usecase[4]):
            address = usecase[5] + i*60
            p = self.ints(address, 15)
            if max(p[2],p[4],p[11],p[13]) > 128:
                raise ValueError('Unexpected graph count')
            nodes = []
            for j in range(p[2]):
                n = self.ints(p[3]+j*32, 8)
                if n[7] > 32:
                    raise ValueError('Unexpected property count')
                properties = []
                for k in range(n[7]):
                    key,value = self.ints(n[0]+k*8, 2)
                    properties.append(dict(id=key, value=self.string(value)))
                nodes.append(dict(node_id=n[1], node=NAMES.get(n[1], 'Unknown'), instance=n[2],
                                  properties=properties, raw_words=n))
            links = []
            for j in range(p[4]):
                link = self.ints(p[5]+j*52, 13)
                if link[4] > 64:
                    raise ValueError('Unexpected destination count')
                links.append(dict(source=self.endpoint(link[:4]),
                                  destinations=[self.endpoint(self.ints(link[5]+k*16,4)) for k in range(link[4])],
                                  elf_va=p[5]+j*52, raw_words=link))
            endpoints = {}
            for kind,count,pointer in [('sinks',p[11],p[12]),('sources',p[13],p[14])]:
                endpoints[kind] = []
                for j in range(count):
                    target = self.ints(pointer+j*24,6)
                    endpoints[kind].append(dict(name=self.string(target[0]),
                                                endpoint=self.endpoint(target[2:]), raw_words=target))
            known = {(n['node_id'],n['instance']) for n in nodes}
            for link in links:
                for endpoint in [link['source']] + link['destinations']:
                    if endpoint['node_id'] not in (2,3) and (endpoint['node_id'],endpoint['instance']) not in known:
                        raise ValueError('Unresolved compiled node endpoint')
            pipelines.append(dict(index=i, name=self.string(p[0]), elf_va=address,
                                  realtime=bool(p[6]), nodes=nodes, links=links, **endpoints))
        mapping = self.ints(0x3118c,36)
        if sorted(mapping) != list(range(36)):
            raise ValueError('Invalid advanced pipeline map')
        return dict(sha256=SHA256, usecase='UsecaseZSL', scope='Compiled definitions; runtime activation is conditional',
                    pipelines=pipelines,
                    advanced_types=[dict(type=i,index=index,name=pipelines[index]['name']) for i,index in enumerate(mapping)],
                    operation_modes=[dict(mode=self.ints(0x31128+i*8,2)[0], mask=self.ints(0x31128+i*8,2)[1]) for i in range(10)])


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('binary', type=Path)
    args = parser.parse_args()
    print(json.dumps(Binary(args.binary).graph(), indent=2))
