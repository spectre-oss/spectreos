"""Offline tests for framing and asynchronous PNG delivery: python -m unittest discover."""
import json
import socket
import struct
import threading
import unittest
from client import Client, MAX_BINARY


def frame(value, binary=b""):
    data = json.dumps(value).encode()
    return struct.pack(">II", len(data), len(binary)) + data + binary


class ClientTests(unittest.TestCase):
    def client_pair(self):
        local, remote = socket.socketpair()
        local.settimeout(1)
        remote.settimeout(1)
        client = Client.__new__(Client)
        client.socket = local
        client.next_id = 0
        import queue
        client.frames = queue.Queue(maxsize=1)
        self.addCleanup(local.close)
        self.addCleanup(remote.close)
        return client, remote

    def test_fragmented_frame(self):
        client, remote = self.client_pair()
        data = frame({"id": 1, "ok": True}, b"png bytes")
        def send():
            for byte in data:
                remote.sendall(bytes([byte]))
        worker = threading.Thread(target=send)
        worker.start()
        self.assertEqual(client.receive(), ({"id": 1, "ok": True}, b"png bytes"))
        worker.join()

    def test_stream_interleaves_command_reply(self):
        client, remote = self.client_pair()
        remote.sendall(frame({"id": 0, "event": "frame", "ok": True}, b"frame1") +
                       frame({"id": 0, "event": "frame", "ok": True}, b"frame2") +
                       frame({"id": 1, "ok": True}))
        self.assertEqual(client.request("heartbeat")[0]["id"], 1)
        self.assertEqual(client.frames.get_nowait()[1], b"frame2")

    def test_rejects_oversized_payload(self):
        client, remote = self.client_pair()
        remote.sendall(struct.pack(">II", 5, MAX_BINARY + 1))
        with self.assertRaises(ValueError):
            client.receive()

    def test_eof_is_reported(self):
        client, remote = self.client_pair()
        remote.close()
        with self.assertRaises(ConnectionError):
            client.receive()

    def test_server_error_is_reported(self):
        client, remote = self.client_pair()
        remote.sendall(frame({"id": 1, "ok": False, "error": "invalid contact"}))
        with self.assertRaisesRegex(RuntimeError, "invalid contact"):
            client.request("touch_up", slot=0)


if __name__ == "__main__":
    unittest.main()
