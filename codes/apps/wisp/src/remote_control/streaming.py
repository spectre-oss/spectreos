"""One continuously received Wisp stream, with multiplexed commands and heartbeats."""
from collections import deque
import queue
import socket
import threading
import time
from client import Client


class StreamingClient(Client):
    background_reader = True

    def __init__(self, host="127.0.0.1", port=8765, unix_socket=None):
        self.running = False
        super().__init__(host, port, unix_socket)
        self.socket.settimeout(None)
        self.closed = threading.Event()
        self.send_lock = threading.Lock()
        self.pending_lock = threading.Lock()
        self.pending = {}
        self.condition = threading.Condition()
        self.latest = None
        self.frame_sequence = 0
        self.frame_received_at = None
        self.error = None
        self.needs_keyframe = threading.Event()
        self.bridge = None
        self.wire_packets = deque()
        self.wire_cache_bytes = 0
        self.wire_sequence = 0
        self.wire_bytes = 0
        self.wire_codec = "png"
        self.await_new_epoch = None
        self.wire_epoch = None
        self.wire_error = None
        self.running = True
        self.reader = threading.Thread(target=self._read_loop, daemon=True)
        self.heartbeat = threading.Thread(target=self._heartbeat_loop, daemon=True)
        self.reader.start()
        self.heartbeat.start()

    def _fail(self, error):
        self.error = str(error)
        self.closed.set()
        if self.bridge is not None: self.bridge.close()
        with self.pending_lock:
            for response in self.pending.values():
                try:
                    response.put_nowait(ConnectionError(self.error))
                except queue.Full:
                    pass
        with self.condition:
            self.condition.notify_all()
        try:
            self.socket.shutdown(socket.SHUT_RDWR)
        except OSError:
            pass

    def _read_loop(self):
        try:
            while not self.closed.is_set():
                metadata, binary = self.receive()
                if metadata.get("event") == "frame":
                    if metadata.get("ok") and metadata.get("codec") == "h264":
                        with self.condition:
                            if self.await_new_epoch == metadata.get("stream_id"):
                                continue
                            self.await_new_epoch = None
                            self.wire_sequence += 1
                            self.wire_bytes += len(binary)
                            self.wire_codec = "h264"
                            if self.wire_epoch != metadata.get("stream_id"):
                                self.wire_packets.clear(); self.wire_cache_bytes = 0
                                self.wire_epoch = metadata.get("stream_id")
                            self.wire_error = None
                            metadata = dict(metadata, client_packet_sequence=self.wire_sequence)
                            if len(binary)>4*1024*1024: raise ValueError("H.264 access unit exceeds limit")
                            self.wire_packets.append((metadata, binary))
                            self.wire_cache_bytes += len(binary)
                            while len(self.wire_packets)>90 or self.wire_cache_bytes>8*1024*1024:
                                self.wire_cache_bytes -= len(self.wire_packets.popleft()[1])
                            self.condition.notify_all()
                        if self.bridge is None:
                            from codec_bridge import CodecBridge
                            self.bridge = CodecBridge(self)
                        self.bridge.submit(metadata, binary)
                    else:
                        if metadata.get("ok") and metadata.get("codec") == "png": self.wire_codec = "png"
                        self.wire_error = metadata.get("error") if not metadata.get("ok") else None
                        self._publish_frame(metadata, binary)
                else:
                    with self.pending_lock:
                        response = self.pending.get(metadata.get("id"))
                        if response:
                            response.put_nowait((metadata, binary))
        except Exception as error:
            self._fail(error)

    def _publish_frame(self, metadata, binary):
        with self.condition:
            if metadata.get("ok") and self.wire_error: return
            if metadata.get("wire_codec") == "h264" and metadata.get("stream_id") == self.await_new_epoch: return
            if metadata.get("wire_codec") == "h264" and (self.wire_codec != "h264" or metadata.get("stream_id") != self.wire_epoch):
                return
            self.frame_sequence += 1
            self.latest = (metadata, binary)
            self.frame_received_at = time.monotonic()
            self.store_frame(metadata, binary)
            self.condition.notify_all()

    def next_packet(self, after=0, timeout=5):
        if self.wire_codec != "h264":
            meta, binary = self.next_frame(after=after or None, timeout=timeout)
            return dict(meta, client_packet_sequence=meta["client_sequence"]), binary
        deadline = time.monotonic()+timeout
        with self.condition:
            while True:
                if self.closed.is_set(): raise ConnectionError(self.error or "session disconnected")
                if self.wire_error: raise RuntimeError(self.wire_error)
                packets = list(self.wire_packets)
                if packets:
                    if after == 0 or after < packets[0][0]["client_packet_sequence"]-1:
                        candidates = [p for p in packets if p[0].get("keyframe")]
                        if candidates: return candidates[-1]
                        self.needs_keyframe.set()
                    else:
                        for packet in packets:
                            if packet[0]["client_packet_sequence"] > after: return packet
                remaining = deadline-time.monotonic()
                if remaining <= 0: raise TimeoutError("no new encoded video packet")
                self.condition.wait(remaining)

    def _heartbeat_loop(self):
        heartbeat = time.monotonic()
        while not self.closed.wait(.2):
            try:
                if self.needs_keyframe.is_set():
                    self.needs_keyframe.clear()
                    self.request("keyframe")
                if time.monotonic()-heartbeat >= 2:
                    self.request("heartbeat")
                    heartbeat = time.monotonic()
            except Exception as error:
                self._fail(error)

    def request(self, kind, **fields):
        if not self.running:
            return super().request(kind, **fields)
        if kind == "stream": fields.setdefault("codec", "h264")
        if self.closed.is_set():
            raise ConnectionError(self.error or "session disconnected; reconnect explicitly")
        response = queue.Queue(maxsize=1)
        with self.send_lock:
            identifier = self.next_id + 1
            with self.pending_lock:
                self.pending[identifier] = response
            try:
                self.send(kind, **fields)
            except Exception as error:
                with self.pending_lock:
                    self.pending.pop(identifier, None)
                self._fail(error)
                raise
        try:
            result = response.get(timeout=6)
            if isinstance(result, Exception):
                raise result
            metadata, binary = result
            if not metadata.get("ok"):
                raise RuntimeError(metadata.get("error", "command failed"))
            if kind == "sleep": self.wire_error = "screen is off; wake first"
            if kind == "wake":
                self.await_new_epoch = self.wire_epoch
                self.wire_error = None
            return metadata, binary
        except queue.Empty:
            error = TimeoutError("Wisp command timed out; session closed to release input")
            self._fail(error)
            raise error from None
        finally:
            with self.pending_lock:
                self.pending.pop(identifier, None)

    def next_frame(self, after=None, timeout=5):
        deadline = time.monotonic() + timeout
        with self.condition:
            if self.wire_error: raise RuntimeError(self.wire_error)
            while self.latest is None or (after is not None and self.frame_sequence <= after):
                if self.wire_error: raise RuntimeError(self.wire_error)
                if self.closed.is_set():
                    raise ConnectionError(self.error or "session disconnected")
                remaining = deadline - time.monotonic()
                if remaining <= 0:
                    raise TimeoutError("no new streamed frame")
                self.condition.wait(remaining)
            if self.closed.is_set():
                raise ConnectionError(self.error or "session disconnected")
            metadata, binary = self.latest
            if not metadata.get("ok"):
                raise RuntimeError(metadata.get("error", "stream unavailable"))
            metadata = dict(metadata, client_sequence=self.frame_sequence,
                            age_ms=round((time.monotonic() - self.frame_received_at)*1000, 1))
            return metadata, binary

    def close(self):
        if not getattr(self, "running", False):
            return super().close()
        if not self.closed.is_set():
            try:
                self.request("disconnect")
            except (OSError, RuntimeError, ValueError):
                pass
        self.closed.set()
        try:
            self.socket.shutdown(socket.SHUT_RDWR)
        except OSError:
            pass
        self.socket.close()
        if self.bridge is not None: self.bridge.close()
        with self.condition:
            self.condition.notify_all()
        for worker in [self.reader, self.heartbeat]:
            if worker is not threading.current_thread():
                worker.join(timeout=2)
