#!/usr/bin/env python3
"""JSON-lines debugging agent. Run as the graphical user over SSH, never as root."""
import base64
import fcntl
import json
import os
from pathlib import Path
import struct
import subprocess
import sys
import time

# Linux uinput ABI: both build host and phone use the generic ioctl encoding.
def ioctl_write(number, size):
    return (1 << 30) | (size << 16) | (ord('U') << 8) | number


def integer(value, low, high):
    if type(value) is not int or not low <= value <= high:
        raise ValueError(f'Expected integer in [{low}, {high}]')
    return value


def coordinate(value):
    if type(value) not in (int, float) or not 0 <= value <= 1:
        raise ValueError('Touch coordinates must be normalized to 0..1')
    return round(value * 65535)


class Input:
    """Separate mouse and direct multitouch devices; no fake hardware keyboard."""
    def __init__(self, touch=False):
        self.fd = os.open('/dev/uinput', os.O_WRONLY | os.O_NONBLOCK)
        try:
            self.bit(100, 1)  # EV_KEY
            if touch:
                self.bit(100, 3)  # EV_ABS
                self.bit(101, 330)  # BTN_TOUCH
                self.bit(101, 325)  # BTN_TOOL_FINGER
                self.bit(110, 1)  # INPUT_PROP_DIRECT
                for code, maximum in [(0, 65535), (1, 65535), (47, 9),
                                      (53, 65535), (54, 65535), (57, 65535)]:
                    self.bit(103, code)
                    fcntl.ioctl(self.fd, ioctl_write(4, 28),
                                struct.pack('=H2xiiiiii', code, 0, 0, maximum, 0, 0, 0))
            else:
                self.bit(100, 2)  # EV_REL
                for code in (0, 1, 8):
                    self.bit(102, code)
                for code in (272, 273, 274):
                    self.bit(101, code)
            name = b'Spectre AI Touch' if touch else b'Spectre AI Mouse'
            setup = struct.pack('=HHHH80sI', 3, 0x1209, 0xF101, 1, name, 0)
            fcntl.ioctl(self.fd, ioctl_write(3, len(setup)), setup)
            fcntl.ioctl(self.fd, 0x5501)  # UI_DEV_CREATE
            time.sleep(0.8)  # Allow udev/libinput to discover the device.
        except BaseException:
            os.close(self.fd)
            raise

    def bit(self, number, value):
        fcntl.ioctl(self.fd, ioctl_write(number, 4), value)

    def event(self, kind, code, value):
        os.write(self.fd, struct.pack('@llHHi', 0, 0, kind, code, value))

    def sync(self):
        self.event(0, 0, 0)

    def close(self):
        try:
            fcntl.ioctl(self.fd, 0x5502)
        finally:
            os.close(self.fd)


def validate_frames(frames):
    if not isinstance(frames, list) or not 1 <= len(frames) <= 500:
        raise ValueError('Provide 1..500 touch frames')
    total = 0
    for frame in frames:
        total += integer(frame.get('delay_ms', 16), 0, 1000)
        points = frame['points']
        if not isinstance(points, list) or len(points) > 10:
            raise ValueError('At most ten contacts per frame')
        ids = set()
        for p in points:
            slot = integer(p['id'], 0, 9)
            if slot in ids:
                raise ValueError('Duplicate touch slot')
            ids.add(slot)
            coordinate(p['x']); coordinate(p['y'])
    if total > 30000:
        raise ValueError('Gesture exceeds 30 seconds')


class Agent:
    def __init__(self):
        self.mouse = self.touch = None
        self.restore_keyboard = None
        self.env = os.environ.copy()
        runtime = f'/run/user/{os.getuid()}'
        self.env.update(XDG_RUNTIME_DIR=runtime,
                        DBUS_SESSION_BUS_ADDRESS=f'unix:path={runtime}/bus')
        displays = sorted(Path(runtime).glob('wayland-*'))
        display = next((p.name for p in displays if not p.name.endswith('.lock')), None)
        if display:
            self.env['WAYLAND_DISPLAY'] = display

    def dbus(self, method, *args):
        return subprocess.check_output([
            'gdbus', 'call', '--session', '--dest', 'sm.puri.OSK0',
            '--object-path', '/sm/puri/OSK0', '--method', method, *args],
            env=self.env, timeout=10, text=True).strip()

    def screen_locked(self):
        # ScreenSaver.GetActive means blanked, not locked. Use logind's hint.
        session = subprocess.check_output(
            ['loginctl', 'show-user', str(os.getuid()), '-p', 'Display', '--value'],
            text=True, timeout=5).strip()
        if not session:
            raise RuntimeError('No graphical session found; cannot establish lock state')
        value = subprocess.check_output(
            ['loginctl', 'show-session', session, '-p', 'LockedHint', '--value'],
            text=True, timeout=5).strip()
        if value not in ('yes', 'no'):
            raise RuntimeError('Unknown graphical session lock state')
        return value == 'yes'

    def perform(self, request):
        action = request['action']
        if action == 'status':
            return {'keyboard_visible': self.dbus('org.freedesktop.DBus.Properties.Get',
                                         'sm.puri.OSK0', 'Visible') == '(<true>,)',
                    'screen_locked': self.screen_locked(),
                    'wayland': self.env.get('WAYLAND_DISPLAY'),
                    'uinput_access': os.access('/dev/uinput', os.W_OK)}
        if action == 'keyboard':
            visible = request['visible']
            if type(visible) is not bool:
                raise ValueError('visible must be boolean')
            if visible:
                if self.screen_locked():
                    raise ValueError('Phone is locked: use its PIN keypad; forcing the OSK can hide it')
                if self.restore_keyboard is None:
                    self.restore_keyboard = self.dbus(
                        'org.freedesktop.DBus.Properties.Get', 'sm.puri.OSK0', 'Visible') == '(<true>,)'
            else:
                self.restore_keyboard = None
            self.dbus('sm.puri.OSK0.SetVisible', str(visible).lower())
            time.sleep(0.3)
            return self.perform({'action': 'status'})
        if action in ('wake', 'screenshot'):
            subprocess.run(['gdbus', 'call', '--session', '--dest',
                            'org.gnome.Mutter.DisplayConfig', '--object-path',
                            '/org/gnome/Mutter/DisplayConfig', '--method',
                            'org.freedesktop.DBus.Properties.Set',
                            'org.gnome.Mutter.DisplayConfig', 'PowerSaveMode',
                            '<int32 0>'], env=self.env, timeout=10,
                           check=True, stdout=subprocess.DEVNULL)
            time.sleep(0.3)
            if action == 'wake':
                return {'awake': True}
        if action == 'screenshot':
            data = subprocess.check_output(['grim', '-'], env=self.env, timeout=15)
            if not data.startswith(b'\x89PNG\r\n\x1a\n'):
                raise RuntimeError('Capture did not return PNG')
            width, height = struct.unpack('>II', data[16:24])
            return {'width': width, 'height': height,
                    'png_base64': base64.b64encode(data).decode()}
        if action == 'mouse':
            dx = integer(request.get('dx', 0), -65535, 65535)
            dy = integer(request.get('dy', 0), -65535, 65535)
            wheel = integer(request.get('wheel', 0), -100, 100)
            button = request.get('click')
            if button not in (None, 'left', 'right', 'middle'):
                raise ValueError('Unknown mouse button')
            if self.mouse is None:
                self.mouse = Input()
            m = self.mouse
            m.event(2, 0, dx); m.event(2, 1, dy); m.event(2, 8, wheel); m.sync()
            if button:
                code = {'left': 272, 'right': 273, 'middle': 274}[button]
                try:
                    m.event(1, code, 1); m.sync(); time.sleep(0.05)
                finally:
                    m.event(1, code, 0); m.sync()
            return {'injected': True}
        if action == 'touch':
            frames = request['frames']
            validate_frames(frames)
            if self.touch is None:
                self.touch = Input(touch=True)
            t, active = self.touch, set()
            try:
                for frame in frames:
                    points = {p['id']: p for p in frame['points']}
                    for slot in active - points.keys():
                        t.event(3, 47, slot); t.event(3, 57, -1)
                    for slot, p in points.items():
                        t.event(3, 47, slot)
                        if slot not in active:
                            t.event(3, 57, slot + 1)
                        t.event(3, 53, coordinate(p['x']))
                        t.event(3, 54, coordinate(p['y']))
                    if points:
                        p = next(iter(points.values()))
                        t.event(3, 0, coordinate(p['x'])); t.event(3, 1, coordinate(p['y']))
                    t.event(1, 330, bool(points)); t.event(1, 325, bool(points)); t.sync()
                    active = set(points)
                    time.sleep(frame.get('delay_ms', 16) / 1000)
            finally:
                for slot in active:
                    t.event(3, 47, slot); t.event(3, 57, -1)
                t.event(1, 330, 0); t.event(1, 325, 0); t.sync()
            return {'injected': True, 'frames': len(frames)}
        raise ValueError('Unknown action')

    def close(self):
        if self.restore_keyboard is not None:
            try:
                self.dbus('sm.puri.OSK0.SetVisible', str(self.restore_keyboard).lower())
            except (OSError, subprocess.SubprocessError):
                pass  # Still release all virtual input devices on disconnect.
        for device in (self.mouse, self.touch):
            if device:
                device.close()


def main():
    agent = Agent()
    try:
        for line in sys.stdin:
            try:
                if len(line) > 256000:
                    raise ValueError('Request too large')
                result = {'ok': True, 'result': agent.perform(json.loads(line))}
            except (OSError, ValueError, KeyError, TypeError, RuntimeError, subprocess.SubprocessError) as error:
                result = {'ok': False, 'error': str(error)}
            print(json.dumps(result), flush=True)
    finally:
        agent.close()


if __name__ == '__main__':
    main()
