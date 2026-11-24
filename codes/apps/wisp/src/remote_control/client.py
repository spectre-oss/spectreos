#!/usr/bin/env python3
"""Wisp remote_control v1 client. Standard library only; viewer uses Tk's PNG decoder."""
import argparse
import base64
import json
import math
import queue
import socket
import struct
import threading
import time
from pathlib import Path

MAX_JSON = 65536
MAX_BINARY = 32 * 1024 * 1024


class Client:
    def __init__(self, host="127.0.0.1", port=8765, unix_socket=None):
        self.socket = socket.socket(socket.AF_UNIX if unix_socket else socket.AF_INET)
        self.socket.settimeout(5)
        self.socket.connect(unix_socket if unix_socket else (host, port))
        self.next_id = 0
        self.frames = queue.Queue(maxsize=1)
        self.status = self.request("hello", version=1)[0]

    def read_exact(self, length):
        data = bytearray()
        while len(data) < length:
            part = self.socket.recv(length - len(data))
            if not part:
                raise ConnectionError("Wisp disconnected (local takeover, timeout, or server restart)")
            data.extend(part)
        return bytes(data)

    def receive(self):
        jlen, blen = struct.unpack(">II", self.read_exact(8))
        if not 0 < jlen <= MAX_JSON or blen > MAX_BINARY:
            raise ValueError("invalid server frame lengths")
        return json.loads(self.read_exact(jlen)), self.read_exact(blen)

    def send(self, kind, **fields):
        self.next_id += 1
        payload = json.dumps(dict(id=self.next_id, type=kind, **fields), allow_nan=False).encode()
        self.socket.sendall(struct.pack(">II", len(payload), 0) + payload)
        return self.next_id

    def request(self, kind, **fields):
        identifier = self.send(kind, **fields)
        while True:
            response, binary = self.receive()
            if response.get("event") == "frame":
                self.store_frame(response, binary)
                continue
            if response.get("id") != identifier:
                raise ValueError("unexpected response id")
            if not response.get("ok"):
                raise RuntimeError(response.get("error", "request failed"))
            return response, binary

    def store_frame(self, response, binary):
        if self.frames.full():
            try:
                self.frames.get_nowait()
            except queue.Empty:
                pass
        self.frames.put_nowait((response, binary))

    def close(self):
        try:
            self.request("disconnect")
        except (OSError, RuntimeError, ValueError):
            pass
        self.socket.close()

    def __enter__(self):
        return self

    def __exit__(self, *_):
        self.close()


def view(client):
    import tkinter as tk
    from tkinter import ttk
    root = tk.Tk()
    root.title("Wisp remote_control — physical touch on phone disconnects")
    events = queue.Queue(maxsize=128)
    stop = threading.Event()
    failures = queue.Queue()
    scale = client.status["scale"]
    factor = max(1, math.ceil(client.status["height"] / 850))
    width = math.ceil(client.status["width"] / factor)
    height = math.ceil(client.status["height"] / factor)
    canvas = tk.Canvas(root, width=width, height=height, background="black", highlightthickness=0)
    bar = ttk.Frame(root)
    bar.pack(fill="x")
    message = tk.StringVar(value="REMOTE CONTROL connected — touch phone to stop")
    ttk.Label(root, textvariable=message).pack(fill="x")
    canvas.pack()
    picture = canvas.create_image(0, 0, anchor="nw")
    slots = set()
    keys = set()

    def enqueue(kind, **fields):
        if stop.is_set():
            return
        try:
            events.put_nowait((kind, fields))
        except queue.Full:
            failures.put("Input queue full; disconnected to release held input")
            stop.set()

    for label, kind, fields in [
        ("Wake", "wake", {}), ("Sleep", "sleep", {}),
        ("Back", "action", {"name": "back"}), ("Home", "action", {"name": "home"}),
        ("Overview", "action", {"name": "overview"}),
    ]:
        ttk.Button(bar, text=label, command=lambda k=kind, f=fields: enqueue(k, **f)).pack(side="left")
    ttk.Button(bar, text="Detach viewer" if getattr(client, "persistent_session", False) else "Disconnect", command=stop.set).pack(side="left")

    def point(event):
        return dict(x=max(0, min(client.status["logical_width"] - .01, event.x * factor / scale)),
                    y=max(0, min(client.status["logical_height"] - .01, event.y * factor / scale)))

    def down(event, slot):
        canvas.focus_set()
        if slot not in slots:
            slots.add(slot)
            enqueue("touch_down", slot=slot, **point(event))

    def motion(event, slot):
        if slot in slots:
            enqueue("touch_move", slot=slot, **point(event))

    def up(_, slot):
        if slot in slots:
            slots.remove(slot)
            enqueue("touch_up", slot=slot)

    for button, slot in [(1, 0), (3, 1)]:
        canvas.bind(f"<ButtonPress-{button}>", lambda e, s=slot: down(e, s))
        canvas.bind(f"<B{button}-Motion>", lambda e, s=slot: motion(e, s))
        canvas.bind(f"<ButtonRelease-{button}>", lambda e, s=slot: up(e, s))

    # Tk keysyms -> Linux evdev. No dependency on host X11 keycode offsets.
    keymap = dict(Escape=1, BackSpace=14, Tab=15, Return=28, space=57,
                  Shift_L=42, Shift_R=54, Control_L=29, Control_R=97,
                  Alt_L=56, Alt_R=100, Super_L=125, Super_R=126,
                  Home=102, Up=103, Prior=104, Left=105, Right=106,
                  End=107, Down=108, Next=109, Insert=110, Delete=111,
                  minus=12, equal=13, bracketleft=26, bracketright=27,
                  semicolon=39, apostrophe=40, grave=41, backslash=43,
                  comma=51, period=52, slash=53)
    for base, chars in [(2, "1234567890"), (16, "qwertyuiop"), (30, "asdfghjkl"), (44, "zxcvbnm")]:
        keymap.update({char: base + index for index, char in enumerate(chars)})
    keymap.update({f"F{n}": 58 + n for n in range(1, 11)})
    keymap.update(F11=87, F12=88)
    for alias, normal in dict(exclam="1", at="2", numbersign="3", dollar="4", percent="5",
                             asciicircum="6", ampersand="7", asterisk="8", parenleft="9", parenright="0",
                             underscore="minus", plus="equal", braceleft="bracketleft", braceright="bracketright",
                             colon="semicolon", quotedbl="apostrophe", asciitilde="grave", bar="backslash",
                             less="comma", greater="period", question="slash").items():
        keymap[alias] = keymap[normal]

    def key(event, pressed):
        code = keymap.get(event.keysym, keymap.get(event.keysym.lower()))
        if code is None:
            return
        if pressed and code not in keys:
            keys.add(code)
            enqueue("key", code=code, down=True)
        elif not pressed and code in keys:
            keys.remove(code)
            enqueue("key", code=code, down=False)
        return "break"

    canvas.bind("<KeyPress>", lambda e: key(e, True))
    canvas.bind("<KeyRelease>", lambda e: key(e, False))

    def unfocus(_):
        for code in list(keys):
            enqueue("key", code=code, down=False)
        keys.clear()
        if slots:
            enqueue("touch_cancel")
            slots.clear()
    canvas.bind("<FocusOut>", unfocus)

    def network():
        try:
            client.request("stream", fps=5)
            heartbeat = time.monotonic()
            while not stop.is_set():
                # Bound each batch so continuous pointer motion cannot starve heartbeats.
                for _ in range(16):
                    try:
                        kind, fields = events.get_nowait()
                    except queue.Empty:
                        break
                    try:
                        client.request(kind, **fields)
                    except RuntimeError as error:
                        failures.put(str(error))
                if time.monotonic() - heartbeat >= 2:
                    client.request("heartbeat")
                    heartbeat = time.monotonic()
                if getattr(client, "background_reader", False):
                    if client.closed.wait(.02):
                        raise ConnectionError(client.error or "session disconnected")
                else:
                    metadata, binary = client.receive()
                    client.store_frame(metadata, binary)
        except Exception as error:
            failures.put(str(error))
            stop.set()
        finally:
            client.close()

    worker = threading.Thread(target=network, daemon=True)
    worker.start()

    def update():
        try:
            response, png = client.frames.get_nowait()
            if response.get("ok") and png:
                image = tk.PhotoImage(data=base64.b64encode(png)).subsample(factor)
                canvas.image = image
                canvas.itemconfigure(picture, image=image)
            elif response.get("error"):
                canvas.itemconfigure(picture, image="")
                canvas.image = None
                message.set(response["error"])
        except queue.Empty:
            pass
        try:
            message.set(failures.get_nowait())
        except queue.Empty:
            pass
        if stop.is_set() and not worker.is_alive():
            message.set("Disconnected — close this window to exit")
        root.after(30, update)

    def close():
        stop.set()
        root.destroy()
    root.protocol("WM_DELETE_WINDOW", close)
    update()
    root.mainloop()
    stop.set()
    worker.join(timeout=6)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=8765)
    parser.add_argument("--socket", help="local Unix socket instead of SSH-forwarded TCP")
    sub = parser.add_subparsers(dest="command", required=True)
    sub.add_parser("status")
    sub.add_parser("view")
    sub.add_parser("wake")
    sub.add_parser("sleep")
    shot = sub.add_parser("screenshot")
    shot.add_argument("output", type=Path)
    tap = sub.add_parser("tap")
    tap.add_argument("x", type=float)
    tap.add_argument("y", type=float)
    swipe = sub.add_parser("swipe")
    for name in ["x1", "y1", "x2", "y2"]:
        swipe.add_argument(name, type=float)
    swipe.add_argument("--duration", type=float, default=.35)
    action = sub.add_parser("action")
    action.add_argument("name", choices=["back", "home", "overview"])
    key = sub.add_parser("key")
    key.add_argument("code", type=int)
    args = parser.parse_args()
    if args.command == "view":
        from streaming import StreamingClient
        connection = StreamingClient
    else:
        connection = Client
    with connection(args.host, args.port, args.socket) as client:
        if args.command == "view":
            view(client)
        elif args.command == "screenshot":
            _, data = client.request("screenshot")
            args.output.write_bytes(data)
            print(args.output)
        elif args.command == "tap":
            client.request("touch_down", slot=0, x=args.x, y=args.y)
            time.sleep(.08)
            client.request("touch_up", slot=0)
        elif args.command == "swipe":
            if not .05 <= args.duration <= 5:
                parser.error("duration must be .05..5 seconds")
            client.request("touch_down", slot=0, x=args.x1, y=args.y1)
            steps = max(2, int(args.duration * 30))
            for n in range(1, steps + 1):
                time.sleep(args.duration / steps)
                client.request("touch_move", slot=0, x=args.x1 + (args.x2-args.x1)*n/steps,
                               y=args.y1 + (args.y2-args.y1)*n/steps)
            client.request("touch_up", slot=0)
        elif args.command == "key":
            client.request("key", code=args.code, down=True)
            client.request("key", code=args.code, down=False)
        elif args.command == "action":
            client.request("action", name=args.name)
        else:
            print(json.dumps(client.request(args.command)[0], indent=2))


if __name__ == "__main__":
    try:
        main()
    except (OSError, RuntimeError, ValueError, ImportError) as error:
        raise SystemExit(f"remote_control: {error}") from None
    except KeyboardInterrupt:
        raise SystemExit(130) from None
