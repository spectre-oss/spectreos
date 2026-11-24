"""Recovery and queue bounds for the encoded-video host path."""
from collections import deque
import queue
import threading
import unittest
from codec_bridge import CodecBridge
from streaming import StreamingClient


class VideoRecoveryTests(unittest.TestCase):
    def client(self, packets):
        client=StreamingClient.__new__(StreamingClient)
        client.wire_codec='h264';client.wire_error=None
        client.closed=threading.Event();client.condition=threading.Condition()
        client.needs_keyframe=threading.Event();client.wire_packets=deque(packets)
        return client
    def test_join_and_lag_recovery_start_at_an_idr(self):
        def packet(n, key):return ({'client_packet_sequence':n,'keyframe':key},bytes([n]))
        client=self.client([packet(10,False),packet(11,True),packet(12,False),packet(13,True),packet(14,False)])
        self.assertEqual(client.next_packet(after=0)[0]['client_packet_sequence'],13)
        self.assertEqual(client.next_packet(after=2)[0]['client_packet_sequence'],13)
        self.assertEqual(client.next_packet(after=13)[0]['client_packet_sequence'],14)
    def test_missing_keyframe_requests_recovery_instead_of_decoding_delta(self):
        client=self.client([({'client_packet_sequence':5,'keyframe':False},b'delta')])
        with self.assertRaises(TimeoutError):client.next_packet(timeout=.01)
        self.assertTrue(client.needs_keyframe.is_set())
    def test_sleep_error_never_returns_a_cached_old_image(self):
        client=self.client([])
        client.latest=({'ok':True},b'old decoded PNG')
        client.frame_sequence=1;client.frame_received_at=0
        client.wire_error='screen is off; wake first'
        with self.assertRaisesRegex(RuntimeError,'screen is off'):
            client.next_frame()
    def test_decode_queue_overflow_discards_reference_chain(self):
        class Owner:
            needs_keyframe=threading.Event()
        bridge=CodecBridge.__new__(CodecBridge)
        bridge.owner=Owner();bridge.waiting_idr=False;bridge.queue=queue.Queue(maxsize=2)
        bridge.submit({'sequence':1,'keyframe':True},b'I')
        bridge.submit({'sequence':2,'keyframe':False},b'P')
        bridge.submit({'sequence':3,'keyframe':False},b'P')
        self.assertTrue(bridge.waiting_idr)
        self.assertTrue(bridge.owner.needs_keyframe.is_set())
        bridge.submit({'sequence':4,'keyframe':False},b'P')
        self.assertTrue(bridge.queue.empty())
        bridge.submit({'sequence':5,'keyframe':True},b'I')
        self.assertEqual(bridge.queue.get_nowait()[0]['sequence'],5)


if __name__=='__main__':unittest.main()
