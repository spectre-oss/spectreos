#!/usr/bin/env python3
"""Persistent remote_control CLI. One Wisp connection, a live stream, ordered commands and batches."""
import argparse
import getpass
import math
import json
import os
from pathlib import Path
import queue
import re
import signal
import socket
import stat
import struct
import subprocess
import sys
import threading
import time
from client import MAX_BINARY
from streaming import StreamingClient

IPC_LIMIT = 128 * 1024
NAME = re.compile(r"[a-zA-Z0-9_]{1,40}\Z")


def control_path(name):
    if not NAME.fullmatch(name):
        raise ValueError("session name must use letters, digits and underscores")
    root = Path(os.environ.get("XDG_RUNTIME_DIR", Path.home()/".cache"))/"wisp_remote_control"
    root.mkdir(mode=0o700, parents=True, exist_ok=True)
    metadata = root.lstat()
    if not stat.S_ISDIR(metadata.st_mode) or metadata.st_uid != os.getuid() or metadata.st_mode & 0o077:
        raise RuntimeError("session directory must be private and owned by this user")
    return root/(name+".sock")


def read_exact(stream, count):
    data = bytearray()
    while len(data) < count:
        part = stream.recv(count-len(data))
        if not part:
            raise ConnectionError("session IPC disconnected")
        data.extend(part)
    return bytes(data)


def read_message(stream, request=False):
    jlen, blen = struct.unpack(">II", read_exact(stream, 8))
    if not 0 < jlen <= IPC_LIMIT or blen > MAX_BINARY or (request and blen):
        raise ValueError("invalid IPC frame lengths")
    return json.loads(read_exact(stream, jlen)), read_exact(stream, blen)


def write_message(stream, metadata, binary=b""):
    data = json.dumps(metadata, allow_nan=False).encode()
    if len(data) > IPC_LIMIT or len(binary) > MAX_BINARY:
        raise ValueError("IPC message too large")
    stream.sendall(struct.pack(">II", len(data), len(binary))+data+binary)


def rpc(name, command, timeout=120):
    with socket.socket(socket.AF_UNIX) as stream:
        stream.settimeout(timeout)
        stream.connect(str(control_path(name)))
        write_message(stream, command)
        result, binary = read_message(stream)
        if not result.get("ok"):
            raise RuntimeError(result.get("error", "session command failed"))
        return result, binary


# US evdev text entry. Credentials are never returned in results or written to logs.
TEXT_KEYS = {}
for base, chars in [(2, "1234567890"), (16, "qwertyuiop"), (30, "asdfghjkl"), (44, "zxcvbnm")]:
    TEXT_KEYS.update({char: (base+index, False) for index, char in enumerate(chars)})
for base, chars in [(12, "-="), (26, "[]"), (39, ";'`"), (43, "\\"), (51, ",./")]:
    TEXT_KEYS.update({char: (base+index, False) for index, char in enumerate(chars)})
TEXT_KEYS.update({" ": (57, False), "\n": (28, False), "\t": (15, False), "\b": (14, False)})
for char in "abcdefghijklmnopqrstuvwxyz":
    TEXT_KEYS[char.upper()] = (TEXT_KEYS[char][0], True)
for shifted, normal in zip('!@#$%^&*()_+{}:"~|<>?', '1234567890-=[];\'`\\,./'):
    TEXT_KEYS[shifted] = (TEXT_KEYS[normal][0], True)


class Controller:
    def __init__(self, connection):
        self.connection = connection
        self.operations = threading.Lock()
        self.keys = set()
        self.started = time.monotonic()
        self.batches = 0
        self.commands = 0

    def status(self):
        c = self.connection
        if c.closed.is_set():
            return {"ok": True, "connected": False, "error": c.error, "frames_received": c.frame_sequence,
                    "commands": self.commands, "batches": self.batches}
        result = dict(c.request("status")[0], connected=True, frames_received=c.frame_sequence,
                      commands=self.commands, batches=self.batches,
                      video_codec=c.wire_codec, wire_packets=c.wire_sequence, wire_bytes=c.wire_bytes,
                      session_age_seconds=round(time.monotonic()-self.started, 2))
        if c.frame_received_at is not None:
            result["frame_age_ms"] = round((time.monotonic()-c.frame_received_at)*1000, 1)
        return result

    def request(self, kind, **fields):
        result = self.connection.request(kind, **fields)[0]
        if kind == "key":
            if fields["down"]:
                self.keys.add(fields["code"])
            else:
                self.keys.discard(fields["code"])
        return result

    def release(self):
        try:
            self.request("touch_cancel")
            for code in list(self.keys):
                self.request("key", code=code, down=False)
        except (OSError, RuntimeError):
            self.connection.close()

    def validate(self, command):
        if not isinstance(command, dict):
            raise ValueError("command must be an object")
        kind = command.get("type")
        allowed = {
            "status": set(), "wake": set(), "sleep": set(), "touch_cancel": set(),
            "action": {"name"}, "tap": {"x", "y", "hold_ms"},
            "swipe": {"x1", "y1", "x2", "y2", "duration"},
            "key": {"code", "down"}, "text": {"text", "submit", "delay_ms"},
            "touch_down": {"slot", "x", "y"}, "touch_move": {"slot", "x", "y"}, "touch_up": {"slot"},
            "wait": {"seconds"}, "frame": {"output", "after", "fresh"}, "video": {"after"}, "keyframe": set(), "screenshot": {"output"},
        }
        if kind not in allowed or set(command)-allowed[kind]-{"type"}:
            raise ValueError("unknown command or fields")
        def number(name, default=None):
            value = command.get(name, default)
            if isinstance(value, bool) or not isinstance(value, (int, float)) or not math.isfinite(value):
                raise ValueError("missing or invalid numeric field: "+name)
            return value
        if kind in ("tap", "touch_down", "touch_move"):
            width, height = self.connection.status["logical_width"], self.connection.status["logical_height"]
            if not (0 <= number("x") < width and 0 <= number("y") < height):
                raise ValueError("touch coordinates outside display")
        if kind in ("touch_down", "touch_move", "touch_up"):
            slot = number("slot")
            if not isinstance(slot, int) or not 0 <= slot <= 9:
                raise ValueError("slot must be 0..9")
        if kind == "tap" and not 10 <= number("hold_ms", 80) <= 1000:
            raise ValueError("hold_ms must be 10..1000")
        if kind == "swipe":
            for x, y in [("x1", "y1"), ("x2", "y2")]:
                if not 0 <= number(x) < self.connection.status["logical_width"] or not 0 <= number(y) < self.connection.status["logical_height"]:
                    raise ValueError("swipe coordinates outside display")
            if not .05 <= number("duration", .35) <= 5:
                raise ValueError("swipe duration must be .05..5 seconds")
        if kind == "key":
            code = number("code")
            if not isinstance(code, int) or not 1 <= code <= 767:
                raise ValueError("keycode must be 1..767")
            if "down" in command and not isinstance(command["down"], bool):
                raise ValueError("down must be boolean")
        if kind == "text":
            text = command.get("text")
            if not isinstance(text, str) or len(text) > 256 or any(c not in TEXT_KEYS for c in text):
                raise ValueError("text must contain at most 256 supported US keyboard characters")
            if not isinstance(command.get("submit", False), bool) or not 0 <= number("delay_ms", 30) <= 200:
                raise ValueError("invalid submit or delay_ms")
        if kind == "action" and command.get("name") not in ("home", "back", "overview"):
            raise ValueError("unknown navigation action")
        if kind == "wait" and not 0 <= number("seconds") <= 10:
            raise ValueError("wait must be 0..10 seconds")
        if kind in ("frame", "screenshot"):
            if "output" in command and not isinstance(command["output"], str):
                raise ValueError("frame output must be a path")
            if "fresh" in command and not isinstance(command["fresh"], bool):
                raise ValueError("fresh must be boolean")
            if "after" in command and (not isinstance(command["after"], int) or command["after"] < 0):
                raise ValueError("after must be a non-negative frame sequence")
        return command

    def execute(self, command):
        kind = command["type"]
        self.commands += 1
        fields = {k:v for k,v in command.items() if k != "type"}
        if kind == "status":
            return self.status(), b""
        if kind == "screenshot":
            metadata, binary = self.connection.request("screenshot")
            if "output" in command:
                target=Path(command["output"]).expanduser().resolve()
                target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(binary)
                return dict(metadata,output=str(target)),b""
            return metadata,binary
        if kind == "video":
            return self.connection.next_packet(after=command.get("after", 0))
        if kind == "frame":
            after = command.get("after", self.connection.frame_sequence if command.get("fresh", True) else None)
            metadata, binary = self.connection.next_frame(after=after)
            result = dict(metadata, ok=True)
            if "output" in command:
                target = Path(command["output"]).expanduser().resolve()
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(binary)
                result["output"] = str(target)
                return result, b""
            return result, binary
        if kind == "wait":
            if self.connection.closed.wait(command["seconds"]):
                raise ConnectionError("session disconnected during wait")
            return {"ok": True}, b""
        if kind == "tap":
            self.request("touch_down", slot=0, x=fields["x"], y=fields["y"])
            time.sleep(fields.get("hold_ms", 80)/1000)
            self.request("touch_up", slot=0)
        elif kind == "swipe":
            duration = fields.get("duration", .35)
            self.request("touch_down", slot=0, x=fields["x1"], y=fields["y1"])
            steps = max(2, int(duration*30))
            for index in range(1, steps+1):
                time.sleep(duration/steps)
                self.request("touch_move", slot=0, x=fields["x1"]+(fields["x2"]-fields["x1"])*index/steps,
                             y=fields["y1"]+(fields["y2"]-fields["y1"])*index/steps)
            self.request("touch_up", slot=0)
        elif kind == "key" and "down" not in fields:
            self.request("key", code=fields["code"], down=True)
            self.request("key", code=fields["code"], down=False)
        elif kind == "text":
            for char in fields["text"]:
                code, shift = TEXT_KEYS[char]
                if shift:
                    self.request("key", code=42, down=True)
                self.request("key", code=code, down=True)
                self.request("key", code=code, down=False)
                if shift:
                    self.request("key", code=42, down=False)
                time.sleep(fields.get("delay_ms", 30)/1000)
            if fields.get("submit"):
                self.request("key", code=28, down=True)
                self.request("key", code=28, down=False)
        else:
            return self.request(kind, **fields), b""
        result = {"ok": True, "type": kind}
        if kind == "text" and fields.get("submit"):
            # Report resulting lock state without echoing the entered text.
            result["locked"] = self.request("status").get("locked")
        return result, b""

    def dispatch(self, command):
        if command.get("type") == "batch":
            commands = command.get("commands")
            if set(command)-{"type", "commands"} or not isinstance(commands, list) or not 1 <= len(commands) <= 128:
                raise ValueError("batch must contain 1..128 commands")
            # Validate the entire structure before performing any UI operation.
            for item in commands:
                self.validate(item)
                if item["type"] in ("frame", "screenshot") and "output" not in item:
                    raise ValueError("a batch frame requires an output path")
            estimated = sum(item.get("seconds", 0) if item["type"] == "wait" else
                            item.get("duration", .35) if item["type"] == "swipe" else
                            item.get("hold_ms", 80)/1000 + .1 if item["type"] == "tap" else
                            len(item["text"])*(.15+item.get("delay_ms", 30)/1000) if item["type"] == "text" else .1
                            for item in commands)
            if estimated > 60:
                raise ValueError("batch exceeds the 60-second estimated work limit; split it")
            with self.operations:
                self.batches += 1
                results = []
                for index, item in enumerate(commands):
                    try:
                        result, _ = self.execute(item)
                        results.append(dict(result, index=index, type=item["type"]))
                    except Exception as error:
                        self.release()
                        return {"ok": True, "completed": False, "failed_index": index,
                                "error": str(error), "results": results}, b""
                return {"ok": True, "completed": True, "results": results}, b""
        self.validate(command)
        # Viewing the stream and querying status never wait behind a long input batch.
        if command["type"] in ("status", "frame", "video", "screenshot"):
            return self.execute(command)
        with self.operations:
            try:
                return self.execute(command)
            except Exception:
                self.release()
                raise


def run_daemon(args):
    path = control_path(args.name)
    if path.exists():
        try:
            rpc(args.name, {"type":"status"}, timeout=1)
        except (OSError, RuntimeError):
            # Do not unlink a reachable socket even if its controller is disconnected.
            with socket.socket(socket.AF_UNIX) as probe:
                try:
                    probe.connect(str(path))
                except ConnectionRefusedError:
                    path.unlink()
                else:
                    raise RuntimeError("session already exists; stop it explicitly")
        else:
            raise RuntimeError("session already exists; stop it explicitly")
    tunnel = None
    if args.ssh:
        with socket.socket() as probe:
            probe.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
            try:
                probe.bind(("127.0.0.1", args.port))
            except OSError:
                raise RuntimeError("local SSH port is already in use; choose another --port") from None
        remote_socket = args.remote_socket or "/run/user/1000/wisp_remote_control.sock"
        command = ["ssh", "-N", "-T", "-o", "BatchMode=yes", "-o", "ExitOnForwardFailure=yes",
                   "-o", "ServerAliveInterval=15", "-o", "ServerAliveCountMax=3",
                   "-L", f"127.0.0.1:{args.port}:{remote_socket}"]
        if args.identity:
            command += ["-i", str(Path(args.identity).expanduser()), "-o", "IdentitiesOnly=yes"]
        command += [args.ssh]
        tunnel = subprocess.Popen(command, stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL,
                                  stderr=subprocess.DEVNULL)
        for _ in range(100):
            if tunnel.poll() is not None:
                raise RuntimeError("SSH forwarding failed (check key, host, or local port)")
            try:
                with socket.create_connection(("127.0.0.1", args.port), timeout=.1):
                    break
            except OSError:
                time.sleep(.05)
        else:
            tunnel.terminate()
            raise TimeoutError("SSH tunnel did not become ready")
    connection = None
    listener = None
    stopping = threading.Event()
    own_inode = None
    def stop(*_):
        stopping.set()
    signal.signal(signal.SIGTERM, stop)
    signal.signal(signal.SIGINT, stop)
    try:
        # A tunnel readiness probe closes before hello; let the server release it.
        if tunnel:
            time.sleep(.1)
            if tunnel.poll() is not None:
                raise RuntimeError("SSH forwarding failed; check the selected port and host")
        connection = StreamingClient(args.host, args.port, args.socket)
        connection.request("stream", fps=args.fps, codec=args.codec)
        controller = Controller(connection)
        listener = socket.socket(socket.AF_UNIX)
        listener.bind(str(path))
        os.chmod(path, 0o600)
        own_inode = path.stat().st_ino
        listener.listen(8)
        listener.settimeout(.2)
        slots = threading.BoundedSemaphore(8)
        def handle(stream):
            with stream:
                try:
                    stream.settimeout(120)
                    command, _ = read_message(stream, request=True)
                    if not isinstance(command, dict):
                        raise ValueError("command must be an object")
                    if command == {"type":"stop"}:
                        result, binary = {"ok":True}, b""
                        stopping.set()
                    else:
                        result, binary = controller.dispatch(command)
                    write_message(stream, result, binary)
                except Exception as error:
                    try:
                        write_message(stream, {"ok":False, "error":str(error)})
                    except OSError:
                        pass
                finally:
                    slots.release()
        while not stopping.is_set():
            try:
                stream, _ = listener.accept()
            except socket.timeout:
                continue
            if not slots.acquire(blocking=False):
                stream.close()
                continue
            threading.Thread(target=handle, args=(stream,), daemon=True).start()
    finally:
        if listener:
            listener.close()
        if connection:
            connection.close()
        if tunnel:
            tunnel.terminate()
            try:
                tunnel.wait(timeout=3)
            except subprocess.TimeoutExpired:
                tunnel.kill()
                tunnel.wait(timeout=3)
        if own_inode is not None and path.exists() and path.stat().st_ino == own_inode:
            path.unlink()


def start(args):
    # An explicit start never replaces a live or locally revoked session.
    path = control_path(args.name)
    if path.exists():
        try:
            status = rpc(args.name, {"type":"status"}, timeout=2)[0]
        except OSError:
            pass
        else:
            if status.get("connected"):
                return status
            raise RuntimeError("session is disconnected; run stop, then start explicitly")
    if args.ssh:
        with socket.socket() as probe:
            probe.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
            try:
                probe.bind(("127.0.0.1", args.port))
            except OSError:
                raise RuntimeError("local SSH port is already in use; choose another --port") from None
    command = [sys.executable, str(Path(__file__).resolve()), "--name", args.name,
               "_daemon", "--host", args.host, "--port", str(args.port), "--fps", str(args.fps), "--codec", args.codec]
    for name in ["socket", "ssh", "identity", "remote_socket"]:
        value = getattr(args, name)
        if value:
            command += ["--"+name.replace("_", "-"), value]
    process = subprocess.Popen(command, stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL,
                               stderr=subprocess.DEVNULL, start_new_session=True)
    for _ in range(150):
        if process.poll() is not None:
            raise RuntimeError("session could not start; check SSH forwarding, Wisp service, and exclusive connection")
        try:
            return rpc(args.name, {"type":"status"}, timeout=1)[0]
        except (OSError, RuntimeError):
            time.sleep(.05)
    process.terminate()
    raise TimeoutError("session startup timed out")


def record(args):
    status = rpc(args.name, {"type":"status"})[0]
    if not status.get("connected"):
        raise RuntimeError("session disconnected")
    output = args.output.expanduser().resolve()
    output.parent.mkdir(parents=True, exist_ok=True)
    command = ["ffmpeg", "-v", "error", "-y", "-f", "image2pipe", "-framerate", str(args.fps),
               "-vcodec", "png", "-i", "-", "-an", "-c:v", "libx264", "-preset", "ultrafast",
               "-pix_fmt", "yuv420p", "-movflags", "+faststart", str(output)]
    process = subprocess.Popen(command, stdin=subprocess.PIPE)
    frames = 0
    unique = set()
    started = time.monotonic()
    deadline = started+args.seconds
    try:
        while time.monotonic() < deadline:
            # Sample the latest continuously received video frame on the output
            # clock. Duplicating a frame preserves wall-clock duration on slow links.
            meta, binary = rpc(args.name, {"type":"frame", "fresh":False})
            unique.add(meta["client_sequence"])
            process.stdin.write(binary)
            frames += 1
            time.sleep(max(0, started+frames/args.fps-time.monotonic()))
    finally:
        process.stdin.close()
        if process.wait(timeout=15):
            raise RuntimeError("video encoder failed")
    return {"ok":True, "output":str(output), "frames":frames, "unique_frames":len(unique), "source":"stream"}


class SessionViewer:
    """A view attached to an existing session; detaching never reconnects or owns Wisp."""
    background_reader = True
    persistent_session = True
    def __init__(self, name):
        self.name = name
        self.status = rpc(name, {"type":"status"})[0]
        if not self.status.get("connected"):
            raise RuntimeError("session disconnected; restart explicitly")
        self.closed = threading.Event()
        self.frames = queue.Queue(maxsize=1)
        self.error = None
        self.worker = threading.Thread(target=self._frames, daemon=True)
        self.worker.start()
    def _store(self, metadata, binary):
        if self.frames.full():
            try: self.frames.get_nowait()
            except queue.Empty: pass
        self.frames.put_nowait((metadata, binary))
    def _frames(self):
        after = 0
        while not self.closed.is_set():
            try:
                meta, binary = rpc(self.name, {"type":"frame", "after":after})
                after = meta["client_sequence"]
                self._store(meta, binary)
            except RuntimeError as error:
                try: connected = rpc(self.name, {"type":"status"})[0].get("connected")
                except (OSError, RuntimeError): connected = False
                if not connected:
                    self.error = str(error); self.closed.set(); break
                self._store({"ok":False, "error":str(error)}, b"")
                self.closed.wait(.2)
            except (OSError, ValueError) as error:
                self.error = str(error); self.closed.set(); break
    def request(self, kind, **fields):
        if kind in ("stream", "heartbeat"):
            return {"ok":True}, b""
        return rpc(self.name, dict(type=kind, **fields))
    def close(self):
        self.closed.set()
        if self.worker is not threading.current_thread():
            self.worker.join(timeout=1)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--name", default="poco")
    sub = parser.add_subparsers(dest="command", required=True)
    for name in ["start", "_daemon"]:
        s = sub.add_parser(name)
        s.add_argument("--host", default="127.0.0.1")
        s.add_argument("--port", type=int, default=8765)
        s.add_argument("--socket")
        s.add_argument("--ssh", help="SSH destination; this session owns its tunnel")
        s.add_argument("--identity")
        s.add_argument("--remote-socket")
        s.add_argument("--fps", type=int, choices=range(1,31), default=15)
        s.add_argument("--codec", choices=["h264", "png"], default="h264")
    for name in ["status", "stop", "wake", "sleep", "view", "keyframe"]:
        sub.add_parser(name)
    frame = sub.add_parser("frame")
    frame.add_argument("output", type=Path)
    screenshot = sub.add_parser("screenshot")
    screenshot.add_argument("output", type=Path)
    tap = sub.add_parser("tap")
    tap.add_argument("x", type=float);tap.add_argument("y", type=float)
    swipe = sub.add_parser("swipe")
    for name in ["x1", "y1", "x2", "y2"]:
        swipe.add_argument(name, type=float)
    swipe.add_argument("--duration", type=float, default=.35)
    key = sub.add_parser("key")
    key.add_argument("code", type=int)
    key.add_argument("--down", action="store_true")
    key.add_argument("--up", action="store_true")
    action = sub.add_parser("action")
    action.add_argument("action_name", choices=["home", "back", "overview"])
    text = sub.add_parser("type")
    text.add_argument("text", nargs="?")
    inputs = text.add_mutually_exclusive_group()
    inputs.add_argument("--stdin", action="store_true")
    inputs.add_argument("--password", action="store_true")
    text.add_argument("--submit", action="store_true")
    batch = sub.add_parser("batch")
    batch.add_argument("file", help="JSON array file, or - for stdin")
    video = sub.add_parser("record")
    video.add_argument("output", type=Path)
    video.add_argument("--seconds", type=float, default=5)
    video.add_argument("--fps", type=float, default=15)
    args = parser.parse_args()
    if args.command == "_daemon":
        run_daemon(args)
        return
    if args.command == "view":
        from client import view
        attached = SessionViewer(args.name)
        try: view(attached)
        finally: attached.close()
        return
    if args.command == "start":
        result = start(args)
    elif args.command == "record":
        if not 0 < args.seconds <= 300 or not 0 < args.fps <= 30:
            parser.error("record duration must be 0..300 seconds and fps 0..30")
        result = record(args)
    else:
        fields = vars(args).copy()
        for name in ["name", "command"]:
            fields.pop(name)
        kind = args.command
        if kind == "type":
            kind = "text"
            if args.password:
                value = getpass.getpass("Phone password (not stored): ")
            elif args.stdin:
                value = sys.stdin.read().removesuffix("\n")
            elif args.text is not None:
                value = args.text
            else:
                parser.error("provide text, --stdin or --password")
            fields = dict(text=value, submit=args.submit)
        elif kind == "action":
            fields = dict(name=args.action_name)
        elif kind == "key":
            if args.down and args.up:
                parser.error("choose --down or --up")
            fields = dict(code=args.code)
            if args.down or args.up:
                fields["down"] = args.down
        elif kind in ("frame", "screenshot"):
            fields = dict(output=str(args.output.expanduser().resolve()))
        elif kind == "batch":
            commands = json.loads(sys.stdin.read() if args.file == "-" else Path(args.file).read_text())
            if not isinstance(commands, list):
                parser.error("batch must be a JSON array")
            for item in commands:
                if isinstance(item, dict) and isinstance(item.get("output"), str):
                    item["output"] = str(Path(item["output"]).expanduser().resolve())
            fields = dict(commands=commands)
        result = rpc(args.name, dict(type=kind, **fields))[0]
        if kind == "stop":
            for _ in range(100):
                if not control_path(args.name).exists():
                    break
                time.sleep(.05)
            else:
                raise TimeoutError("session stop is still cleaning up")
    print(json.dumps(result, indent=2))
    if result.get("completed") is False:
        raise SystemExit(1)


if __name__ == "__main__":
    try:
        main()
    except (OSError, RuntimeError, ValueError, KeyError) as error:
        raise SystemExit(f"remote_control: {error}") from None
    except KeyboardInterrupt:
        raise SystemExit(130) from None
