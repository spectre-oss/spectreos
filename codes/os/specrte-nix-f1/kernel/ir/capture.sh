#!/bin/sh
# RAM diagnostic only: configure the isolated IR pipeline and print aggregates.
set -eu
frames=${1:-30}
pattern=${2:-0}
case "$frames" in ''|*[!0-9]*) exit 2;; esac
[ "$frames" -ge 1 ] && [ "$frames" -le 150 ] || exit 2
case "$pattern" in 0|1) ;; *) exit 2;; esac
sensor=$(media-ctl -e 'ov7251 17-0060')
video=$(media-ctl -e 'msm_vfe0_video0')
raw=$(mktemp /run/ir-capture.XXXXXX)
cleanup() {
 rm -f "$raw"
 v4l2-ctl -d "$sensor" --set-ctrl=test_pattern=0 >/dev/null 2>&1 || :
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
media-ctl -r
media-ctl -l '"msm_csiphy1":1 -> "msm_csid0":0 [1]'
media-ctl -l '"msm_csid0":1 -> "msm_vfe0_rdi0":0 [1]'
for entity in 'ov7251 17-0060' msm_csiphy1 msm_csid0 msm_vfe0_rdi0; do
 media-ctl -V "\"$entity\":0 [fmt:Y10_1X10/640x480]"
done
v4l2-ctl -d "$sensor" --set-ctrl="test_pattern=$pattern"
v4l2-ctl -d "$video" --set-fmt-video=width=640,height=480,pixelformat=Y10P
timeout 8 v4l2-ctl -d "$video" --stream-mmap=4 --stream-count="$frames" --stream-to="$raw"
bytes=$(wc -c < "$raw")
[ "$bytes" -eq "$((frames * 384000))" ] || {
 echo "Incomplete capture: $bytes bytes for $frames requested frames" >&2
 exit 1
}
/opt/pixel-stats --packed --frame-pixels 307200 < "$raw"
