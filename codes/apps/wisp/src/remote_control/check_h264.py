#!/usr/bin/env python3
"""Explicit live H.264 checks against an existing stateful session.
Sleeps/wakes the phone, moves the lock screen, and sends Shift (no credential).
"""
import argparse
import json
from pathlib import Path
import re
import struct
import time
from session import rpc


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--name',default='poco')
    parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args();args.output.mkdir(parents=True,exist_ok=True)
    def call(kind,**fields):return rpc(args.name,dict(type=kind,**fields))
    def status():return call('status')[0]
    def nals(data):return [data[m.end()]&31 for m in re.finditer(b'\x00\x00\x00\x01|\x00\x00\x01',data)]
    call('wake');time.sleep(.3)
    first,raw=call('video',after=0)
    assert first['codec']=='h264' and first['keyframe']
    assert {5,7,8}.issubset(nals(raw)),nals(raw)
    before=status();start=time.monotonic();after=first['client_packet_sequence']
    packets=[first];bitstream=bytearray(raw)
    while time.monotonic()-start<5:
        packet,data=call('video',after=after);after=packet['client_packet_sequence']
        packets.append(packet);bitstream.extend(data)
    elapsed=time.monotonic()-start;end=status()
    results={'codec':'h264','encoded_size':[first['width'],first['height']],
             'fps':round((end['wire_packets']-before['wire_packets'])/elapsed,2),
             'encoded_payload_kbit_s':round((end['wire_bytes']-before['wire_bytes'])*8/elapsed/1000,2),
             'sample_seconds':round(elapsed,3),'keyframes':sum(bool(p['keyframe']) for p in packets)}
    (args.output/'live.h264').write_bytes(bitstream)
    # Force a new decodable access unit and measure time to recovery point.
    after=status()['wire_packets'];start=time.monotonic();call('keyframe')
    while True:
        packet,data=call('video',after=after);after=packet['client_packet_sequence']
        if packet.get('keyframe'):break
        assert time.monotonic()-start<3
    assert {5,7,8}.issubset(nals(data))
    results['requested_idr_ms']=round((time.monotonic()-start)*1000,1)
    latency=[]
    for _ in range(5):
        for down in [True,False]:
            start=time.monotonic();call('key',code=42,down=down);latency.append((time.monotonic()-start)*1000)
    results['key_ack_ms']={'mean':round(sum(latency)/len(latency),1),'max':round(max(latency),1)}
    call('tap',x=100,y=300)
    call('frame',output=str((args.output/'tap.png').resolve()))
    # Sleep is a video pause, not a lost control connection. Old cache must fail.
    old_epoch=packet['stream_id'];call('sleep')
    try:call('frame');raise AssertionError('sleep returned a stale frame')
    except RuntimeError as error:assert 'screen is off' in str(error)
    assert status()['connected'] and not status()['screen_on']
    results['pause_rejects_stale_frame']=True
    start=time.monotonic();call('wake')
    deadline=time.monotonic()+4
    while time.monotonic()<deadline:
        try:packet,data=call('video',after=after)
        except RuntimeError:time.sleep(.05);continue
        after=packet['client_packet_sequence']
        if packet['stream_id']!=old_epoch:break
    assert packet['stream_id']!=old_epoch and packet['keyframe']
    results['wake_new_epoch_idr_ms']=round((time.monotonic()-start)*1000,1)
    call('frame',output=str((args.output/'resumed.png').resolve()))
    call('screenshot',output=str((args.output/'full_resolution.png').resolve()))
    png=(args.output/'full_resolution.png').read_bytes()
    assert struct.unpack('>II',png[16:24])==(1080,2246)
    results['full_resolution_screenshot']=[1080,2246]
    # A reader older than the bounded GOP cache must rejoin at a keyframe.
    packet,_=call('video',after=0);assert packet['keyframe']
    results['new_viewer_starts_at_idr']=True
    final=status();assert final['connected'] and final['keys']==final['contacts']==0
    results['final_status']=final
    (args.output/'checks.json').write_text(json.dumps(results,indent=2)+'\n')
    print(json.dumps(results,indent=2))


if __name__=='__main__':main()
