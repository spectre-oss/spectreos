# Spectre Camera

Qt 6 Quick camera for the Poco F1. Version 0.3 uses **libcamera** and its
software ISP for a small live preview and full-resolution processed photos,
automatic exposure/gain and white balance. Preview rotation runs in Qt Quick;
photo encoding runs separately from real-camera preview processing. Qt presents the image and saves JPEG quality 100 or lossless PNG
with capture metadata. The previous custom RAW demosaicer is superseded.

Picture quality takes priority. Do not claim calibrated colour, focus or noise
performance from successful frame capture or demo tests. See the project-root
`docs/qt-camera.md` for the local IMX363/ISP patches, evidence, limitations,
server-only builds and installation. Demo frames do not validate real quality.

This app's original source is GPL-3.0-or-later. It may be redistributed and
modified under GNU GPL version 3 or any later version, without warranty; see
[the GNU GPL](https://www.gnu.org/licenses/gpl-3.0.html). Qt and libcamera retain
their respective licenses. Backported code retains its original attribution.
