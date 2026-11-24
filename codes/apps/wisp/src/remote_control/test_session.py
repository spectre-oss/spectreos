"""Persistent CLI batching and continuous stream regression tests."""
import json
import select
import socket
import struct
import threading
import time
import unittest
from unittest.mock import patch
from streaming import StreamingClient
from session import Controller, TEXT_KEYS, read_message, write_message


class FakeConnection:
    def __init__(self):
        self.status = dict(logical_width=400, logical_height=832)
        self.closed = threading.Event()
        self.frame_sequence = 0
        self.frame_received_at = None
        self.calls = []
        self.failure = None
    def request(self, kind, **fields):
        self.calls.append((kind, fields))
        if self.failure and self.failure(kind, fields):
            raise RuntimeError("injected failure")
        return dict(id=len(self.calls), ok=True), b""
    def close(self):
        self.closed.set()


class BatchTests(unittest.TestCase):
    def setUp(self):
        self.connection = FakeConnection()
        self.controller = Controller(self.connection)
    def test_preflight_prevents_partial_input_for_invalid_batch(self):
        with self.assertRaises(ValueError):
            self.controller.dispatch(dict(type="batch", commands=[dict(type="tap", x=5, y=5),
                                      dict(type="tap", x=500, y=5)]))
        self.assertEqual(self.connection.calls, [])
    def test_failure_stops_batch_and_releases_input(self):
        self.connection.failure = lambda kind, fields: kind == "touch_move"
        result, _ = self.controller.dispatch(dict(type="batch", commands=[
            dict(type="key", code=42, down=True),
            dict(type="swipe", x1=50, y1=100, x2=50, y2=200, duration=.05),
            dict(type="key", code=30)]))
        self.assertFalse(result["completed"])
        self.assertEqual(result["failed_index"], 1)
        self.assertIn(("touch_cancel", {}), self.connection.calls)
        self.assertIn(("key", dict(code=42, down=False)), self.connection.calls)
        self.assertNotIn(("key", dict(code=30, down=True)), self.connection.calls)
    def test_batches_serialize_ui_input(self):
        def batch(code):
            self.controller.dispatch(dict(type="batch", commands=[
                dict(type="key", code=code, down=True), dict(type="wait", seconds=.02),
                dict(type="key", code=code, down=False)]))
        workers = [threading.Thread(target=batch, args=(code,)) for code in [30, 31]]
        for worker in workers: worker.start()
        for worker in workers: worker.join()
        codes = [fields["code"] for kind, fields in self.connection.calls if kind == "key"]
        self.assertIn(codes, [[30, 30, 31, 31], [31, 31, 30, 30]])
    def test_password_text_is_not_returned(self):
        result, _ = self.controller.dispatch(dict(type="text", text="pAss1!", submit=True, delay_ms=0))
        self.assertNotIn("pAss1!", json.dumps(result))
        self.assertEqual(self.connection.calls[-2], ("key", dict(code=28, down=False)))
        self.assertEqual(self.connection.calls[-1], ("status", {}))
        self.assertEqual(self.controller.keys, set())
        self.assertEqual(TEXT_KEYS['!'], (2, True))
    def test_oversized_batch_wait_is_rejected_before_input(self):
        with self.assertRaises(ValueError):
            self.controller.dispatch(dict(type="batch", commands=[dict(type="wait", seconds=10)]*7))
        self.assertEqual(self.connection.calls, [])


class CLITests(unittest.TestCase):
    def test_navigation_does_not_replace_session_name(self):
        import session
        with patch("sys.argv", ["session.py", "--name", "poco", "action", "home"]), \
             patch.object(session, "rpc", return_value=({"ok": True}, b"")) as rpc_call, \
             patch("builtins.print"):
            session.main()
        rpc_call.assert_called_once_with("poco", {"type": "action", "name": "home"})


class StreamingTests(unittest.TestCase):
    def test_frames_arrive_without_status_or_screenshot_polling(self):
        listener = socket.socket()
        listener.bind(("127.0.0.1", 0)); listener.listen(1)
        port = listener.getsockname()[1]
        calls, errors = [], []
        def server():
            try:
                with listener, listener.accept()[0] as stream:
                    streaming = False; sequence = 0; last_frame = 0
                    while True:
                        if select.select([stream], [], [], .01)[0]:
                            request, _ = read_message(stream, request=True)
                            calls.append(request["type"])
                            reply = dict(id=request["id"], ok=True)
                            if request["type"] == "hello":
                                reply.update(logical_width=400, logical_height=832, scale=2.7, width=1080, height=2246)
                            write_message(stream, reply)
                            if request["type"] == "disconnect": break
                            if request["type"] == "stream": streaming = request["fps"] > 0
                        if streaming and time.monotonic()-last_frame > .05:
                            sequence += 1; last_frame = time.monotonic()
                            write_message(stream, dict(id=0, ok=True, event="frame", sequence=sequence), b"\x89PNGtest")
            except Exception as error:
                errors.append(error)
        worker = threading.Thread(target=server)
        worker.start()
        with StreamingClient(port=port) as client:
            client.request("stream", fps=5)
            first, _ = client.next_frame()
            second, _ = client.next_frame(after=first["client_sequence"])
            self.assertGreater(second["client_sequence"], first["client_sequence"])
            time.sleep(2.1)
            self.assertIn("heartbeat", calls)
            self.assertNotIn("status", calls)
            self.assertNotIn("screenshot", calls)
            self.assertTrue(client.request("status")[0]["ok"])
        worker.join(timeout=2)
        self.assertFalse(worker.is_alive())
        self.assertEqual(errors, [])


if __name__ == "__main__":
    unittest.main()
