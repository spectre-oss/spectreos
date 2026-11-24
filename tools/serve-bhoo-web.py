#!/usr/bin/env python3
"""Build and serve Bhoo's browser component explorer on loopback."""
import argparse
from functools import partial
from http.server import SimpleHTTPRequestHandler,ThreadingHTTPServer
from pathlib import Path
import subprocess

ROOT=Path(__file__).resolve().parents[1]
class Handler(SimpleHTTPRequestHandler):
    extensions_map={**SimpleHTTPRequestHandler.extensions_map,'.wasm':'application/wasm','.js':'text/javascript'}
    def end_headers(self):
        self.send_header('Cache-Control','no-cache')
        self.send_header('X-Content-Type-Options','nosniff')
        super().end_headers()

def main():
    parser=argparse.ArgumentParser();parser.add_argument('--port',type=int,default=8766);parser.add_argument('--no-build',action='store_true');parser.add_argument('--rust-only',action='store_true');args=parser.parse_args()
    if not args.no_build:subprocess.run(['python3',str(ROOT/'tools/build-bhoo-web.py')]+(['--rust-only'] if args.rust_only else []),cwd=ROOT,check=True)
    with ThreadingHTTPServer(('127.0.0.1',args.port),partial(Handler,directory=str(ROOT/'codes/apps/bhoo-web/site'))) as server:
        print(f'Bhoo component explorer: http://127.0.0.1:{server.server_port}',flush=True)
        if args.rust_only:print(f'Rust-only playground: http://127.0.0.1:{server.server_port}/playground.html',flush=True)
        try:server.serve_forever()
        except KeyboardInterrupt:pass
if __name__=='__main__':main()
