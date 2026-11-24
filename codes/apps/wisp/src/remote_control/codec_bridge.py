"""Bounded host decoder bridge; the codec itself lives in the Rust viewer."""
import json
import os
from pathlib import Path
import queue
import struct
import subprocess
import threading
from client import MAX_BINARY, MAX_JSON


class CodecBridge:
    def __init__(self, owner):
        self.owner = owner
        binary = Path(os.environ.get('WISP_REMOTE_CODEC', Path(__file__).resolve().parents[4] /
                       'utils/wisp_remote_viewer/target/release/wisp_remote_viewer'))
        if not binary.is_file():
            raise RuntimeError('Build the Rust viewer codec first, or start with --codec png')
        self.process = subprocess.Popen([str(binary), '--decode-video'], stdin=subprocess.PIPE,
                                        stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
        self.queue = queue.Queue(maxsize=4)
        self.closing = threading.Event()
        self.waiting_idr = True
        self.writer = threading.Thread(target=self._write, daemon=True)
        self.reader = threading.Thread(target=self._read, daemon=True)
        self.writer.start(); self.reader.start()

    def submit(self, metadata, binary):
        if self.waiting_idr and not metadata.get('keyframe'):
            self.owner.needs_keyframe.set()
            return
        try:
            self.queue.put_nowait((metadata, binary))
            self.waiting_idr = False
        except queue.Full:
            # Never decode a damaged reference chain after dropping queued packets.
            while True:
                try: self.queue.get_nowait()
                except queue.Empty: break
            self.waiting_idr = True
            self.owner.needs_keyframe.set()
            if metadata.get('keyframe'):
                self.queue.put_nowait((metadata, binary)); self.waiting_idr = False

    def _write(self):
        try:
            while not self.owner.closed.is_set():
                item = self.queue.get()
                if item is None: break
                metadata, binary = item
                data = json.dumps(metadata).encode()
                self.process.stdin.write(struct.pack('>II', len(data), len(binary))+data+binary)
                self.process.stdin.flush()
        except Exception as error:
            if not self.owner.closed.is_set(): self.owner._fail(error)

    def _read(self):
        def exact(count):
            result = bytearray()
            while len(result) < count:
                part = self.process.stdout.read(count-len(result))
                if not part: raise ConnectionError('Rust H.264 decoder exited')
                result.extend(part)
            return result
        try:
            while not self.owner.closed.is_set():
                jlen, blen = struct.unpack('>II', exact(8))
                if not 0 < jlen <= MAX_JSON or blen > MAX_BINARY: raise ValueError('invalid decoder frame')
                metadata, binary = json.loads(exact(jlen)), bytes(exact(blen))
                if metadata.get('event') == 'need_keyframe': self.owner.needs_keyframe.set()
                else: self.owner._publish_frame(metadata, binary)
        except Exception as error:
            if not self.owner.closed.is_set(): self.owner._fail(error)

    def close(self):
        if self.closing.is_set(): return
        self.closing.set()
        self.process.terminate()
        try: self.process.wait(timeout=2)
        except subprocess.TimeoutExpired: self.process.kill(); self.process.wait()
        try: self.queue.put_nowait(None)
        except queue.Full: pass
