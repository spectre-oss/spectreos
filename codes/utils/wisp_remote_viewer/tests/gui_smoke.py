#!/usr/bin/env python3
"""Opt-in live GUI check. Requires a private Xvfb DISPLAY, XTest, FFmpeg and an
already connected/unlocked `poco` CLI session. Sends a blank-area tap and holds
Shift, then closes the Rust window to verify release without stopping the session.
Python is test instrumentation only, never a viewer runtime dependency.
"""
import argparse
import ctypes as c
import json
import os
from pathlib import Path
import socket
import struct
import subprocess
import time


def rpc(path, command):
    with socket.socket(socket.AF_UNIX) as stream:
        stream.settimeout(8)
        stream.connect(str(path))
        data=json.dumps(command).encode()
        stream.sendall(struct.pack('>II',len(data),0)+data)
        def read(count):
            data=bytearray()
            while len(data)<count:
                part=stream.recv(count-len(data))
                if not part: raise ConnectionError('IPC disconnected')
                data.extend(part)
            return data
        jlen,blen=struct.unpack('>II',read(8))
        result=json.loads(read(jlen));read(blen)
        if not result.get('ok'):raise RuntimeError(result.get('error'))
        return result


def wait_for(predicate, timeout=6):
    deadline=time.monotonic()+timeout
    while time.monotonic()<deadline:
        result=predicate()
        if result:return result
        time.sleep(.03)
    raise TimeoutError('live GUI assertion timed out')


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    package=Path(__file__).resolve().parents[1]
    parser.add_argument('--binary',type=Path,default=package/'target/debug/wisp_remote_viewer')
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument("--allow-locked",action="store_true",help="allow the Shift/blank-touch check on the lock screen")
    args=parser.parse_args()
    assert os.environ.get('DISPLAY'),'use an isolated Xvfb display'
    path=Path(os.environ.get('XDG_RUNTIME_DIR',Path.home()/'.cache'))/'wisp_remote_control/poco.sock'
    status=rpc(path,{'type':'status'})
    assert status['connected'] and (not status['locked'] or args.allow_locked) and status['keys']==status['contacts']==0
    x=c.CDLL('libX11.so.6');xt=c.CDLL('libXtst.so.6')
    x.XOpenDisplay.argtypes=[c.c_char_p];x.XOpenDisplay.restype=c.c_void_p
    x.XDefaultRootWindow.argtypes=[c.c_void_p];x.XDefaultRootWindow.restype=c.c_ulong
    x.XQueryTree.argtypes=[c.c_void_p,c.c_ulong,c.POINTER(c.c_ulong),c.POINTER(c.c_ulong),c.POINTER(c.POINTER(c.c_ulong)),c.POINTER(c.c_uint)]
    x.XFetchName.argtypes=[c.c_void_p,c.c_ulong,c.POINTER(c.c_char_p)]
    x.XFree.argtypes=[c.c_void_p]
    x.XFlush.argtypes=[c.c_void_p]
    x.XCloseDisplay.argtypes=[c.c_void_p]
    x.XSetInputFocus.argtypes=[c.c_void_p,c.c_ulong,c.c_int,c.c_ulong]
    x.XStringToKeysym.argtypes=[c.c_char_p];x.XStringToKeysym.restype=c.c_ulong
    x.XKeysymToKeycode.argtypes=[c.c_void_p,c.c_ulong];x.XKeysymToKeycode.restype=c.c_ubyte
    x.XInternAtom.argtypes=[c.c_void_p,c.c_char_p,c.c_int];x.XInternAtom.restype=c.c_ulong
    x.XSendEvent.argtypes=[c.c_void_p,c.c_ulong,c.c_int,c.c_long,c.c_void_p]
    xt.XTestFakeKeyEvent.argtypes=[c.c_void_p,c.c_uint,c.c_int,c.c_ulong]
    xt.XTestFakeButtonEvent.argtypes=[c.c_void_p,c.c_uint,c.c_int,c.c_ulong]
    xt.XTestFakeMotionEvent.argtypes=[c.c_void_p,c.c_int,c.c_int,c.c_int,c.c_ulong]
    display=x.XOpenDisplay(None);assert display
    root=x.XDefaultRootWindow(display)
    def find(window):
        name=c.c_char_p()
        if x.XFetchName(display,window,c.byref(name)):
            title=name.value or b'';x.XFree(name)
            if b'Wisp remote_control' in title:return window
        r=c.c_ulong();parent=c.c_ulong();children=c.POINTER(c.c_ulong)();count=c.c_uint()
        if x.XQueryTree(display,window,c.byref(r),c.byref(parent),c.byref(children),c.byref(count)):
            windows=[children[i] for i in range(count.value)]
            if children:x.XFree(children)
            for child in windows:
                found=find(child)
                if found:return found
        return None
    env=os.environ.copy();env['WINIT_UNIX_BACKEND']='x11'
    process=subprocess.Popen([str(args.binary),'--session','poco'],env=env)
    shift=x.XKeysymToKeycode(display,x.XStringToKeysym(b'Shift_L'))
    try:
        window=wait_for(lambda:find(root))
        time.sleep(.6)
        x.XSetInputFocus(display,window,2,0);x.XFlush(display)
        xt.XTestFakeMotionEvent(display,0,220,500,0)
        xt.XTestFakeButtonEvent(display,1,1,0);x.XFlush(display)
        wait_for(lambda:rpc(path,{'type':'status'})['contacts']==1)
        time.sleep(.4)
        args.output.parent.mkdir(parents=True,exist_ok=True)
        subprocess.run(['ffmpeg','-v','error','-y','-f','x11grab','-i',os.environ['DISPLAY'],'-frames:v','1',str(args.output)],check=True)
        xt.XTestFakeButtonEvent(display,1,0,0);x.XFlush(display)
        wait_for(lambda:rpc(path,{'type':'status'})['contacts']==0)
        xt.XTestFakeKeyEvent(display,shift,1,0);x.XFlush(display)
        wait_for(lambda:rpc(path,{'type':'status'})['keys']==1)
        class Message(c.Structure):
            _fields_=[('type',c.c_int),('serial',c.c_ulong),('send_event',c.c_int),('display',c.c_void_p),('window',c.c_ulong),('message_type',c.c_ulong),('format',c.c_int),('data',c.c_long*5)]
        class Event(c.Union):_fields_=[('message',Message),('pad',c.c_long*24)]
        event=Event();event.message.type=33;event.message.display=display;event.message.window=window
        event.message.message_type=x.XInternAtom(display,b'WM_PROTOCOLS',0);event.message.format=32
        event.message.data[0]=x.XInternAtom(display,b'WM_DELETE_WINDOW',0)
        x.XSendEvent(display,window,0,0,c.byref(event));x.XFlush(display)
        assert process.wait(timeout=8)==0
        final=wait_for(lambda:(s if (s:=rpc(path,{'type':'status'}))['keys']==0 else None))
        assert final['connected'] and final['contacts']==0
        print(json.dumps({'ok':True,'mouse_touch_verified':True,'physical_keyboard_verified':True,'held_shift_released_on_close':True,'session_connected_after_detach':True,'window_capture':str(args.output)}))
    finally:
        xt.XTestFakeButtonEvent(display,1,0,0);xt.XTestFakeKeyEvent(display,shift,0,0);x.XFlush(display)
        if process.poll() is None:
            process.terminate();process.wait(timeout=5)
            try:
                rpc(path,{'type':'touch_cancel'})
                rpc(path,{'type':'key','code':42,'down':False})
            except Exception:pass
        x.XCloseDisplay(display)


if __name__=='__main__':main()
