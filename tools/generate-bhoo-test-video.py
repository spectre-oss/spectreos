#!/usr/bin/env python3
"""Generate the bundled silent test pattern; pass the path to an FFmpeg binary."""
import argparse, subprocess
from pathlib import Path
parser=argparse.ArgumentParser();parser.add_argument('--ffmpeg',default='ffmpeg');args=parser.parse_args()
root=Path(__file__).resolve().parents[1];output=root/'codes/apps/bhoo-web/site/bhoo-test-video.webm'
subprocess.run([args.ffmpeg,'-hide_banner','-loglevel','error','-f','lavfi','-i','testsrc2=size=320x180:rate=24','-t','8','-an','-c:v','libvpx','-deadline','good','-b:v','280k','-pix_fmt','yuv420p','-y',str(output)],check=True)
