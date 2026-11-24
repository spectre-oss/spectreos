#!/usr/bin/env python3
"""Run Bhoo's GLES smoke test in an isolated headless Weston compositor.

Requires cargo, weston, an EGL/GLES implementation, and a font (see BHOO_FONT).
Uses only Python's standard library. Does not alter the desktop session.
"""
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]
CRATE = ROOT / "codes/libs/ui/libbhoo"


def check_capture(path, scale, example):
    with path.open("rb") as image:
        assert image.readline() == b"P6\n", "Expected a PPM capture"
        width, height = map(int, image.readline().split())
        assert image.readline() == b"255\n"
        pixels = image.read()
    assert len(pixels) == width * height * 3
    expected = (300, 180) if example in {"split_outline", "checkbox_native", "image_reflection"} else (440, 360) if example in {"launcher", "effects"} else (720, 640)
    assert (width, height) == (expected[0] * scale, expected[1] * scale), (width, height)

    def pixel(x, y):
        offset = ((y * scale) * width + x * scale) * 3
        return tuple(pixels[offset:offset + 3])

    if example == "image_reflection":
        assert pixel(9,9)[1]>200 and pixel(9,9)[0]<30, "Reflected top-left should be green"
        assert pixel(86,9)[0]>200 and pixel(86,9)[1]<30, "Reflected top-right should be red"
        assert min(pixel(9,86)[:2])>200 and pixel(9,86)[2]<30, "Reflected bottom-left should be yellow"
        assert pixel(86,86)[2]>200 and pixel(86,86)[0]<30, "Reflected bottom-right should be blue"
        print(f"PASS: ImageView reflection, Wayland + GLES, scale {scale}")
        return
    if example == "checkbox_native":
        assert pixel(0, 0) == (18, 18, 22)
        assert pixel(36, 40) == (208, 188, 255), "Checked box fill missing"
        assert pixel(44, 45) != (208, 188, 255), "Checked mark missing"
        assert pixel(44, 96) != (208, 188, 255), "Mixed mark missing"
        assert pixel(36, 91) == (208, 188, 255), "Mixed box fill missing"
        # Examine physical pixels inside the fill, away from the rounded outer
        # edge. Intermediate colors must come from the checkmark's AA fringe.
        interior = []
        for y in range(38 * scale, 51 * scale):
            for x in range(38 * scale, 51 * scale):
                offset = (y * width + x) * 3
                interior.append(tuple(pixels[offset:offset + 3]))
        assert any(58 < r < 206 and 32 < g < 186 and 116 < b < 253
                   for r, g, b in interior), "Checkmark has no antialiased edge pixels"
        print(f"PASS: checkbox marks, Wayland + GLES, scale {scale}")
        return
    if example == "split_outline":
        background = (41, 110, 75)
        assert pixel(0, 0) == background
        assert pixel(70, 75) == background, "Outlined interior hid the underlying surface"
        assert pixel(30, 60) == background, "Outlined outer corner was filled"
        assert pixel(100, 60) != background, "Outlined top edge was not rendered"
        assert pixel(30, 88) != background, "Outlined left edge was not rendered"
        assert pixel(232, 88) == background, "Outline painted outside its bounds"
        print(f"PASS: split outline, Wayland + GLES, scale {scale}, transparent interior and border pixels")
        return
    assert pixel(0, 0) == (18, 18, 22), "Background was not rendered"
    if example == "effects":
        assert pixel(24,72)==(18,18,22), "Ripple escaped the rounded corner"
        assert pixel(220,108)==(42,42,50), "Ripple escaped its radial footprint"
        assert min(pixel(100,108))>100, "Radial ink was not rendered"
        assert max(pixel(48,224))>180, "Determinate morph path was not rendered"
        assert max(pixel(144,224))>180, "Indeterminate morph path was not rendered"
        assert max(pixel(240,202))>180, "Circular progress arc was not rendered"
        assert pixel(240,220)==(18,18,22), "Circular progress filled its hollow center"
        assert pixel(310,82)[0]>150 and pixel(310,82)[1]<100, "Image red quadrant missing"
        assert pixel(354,82)[1]>150 and pixel(354,82)[0]<100, "Image green quadrant missing"
        assert pixel(310,126)[2]>150, "Image blue quadrant missing"
        assert max(pixel(324,220))>180, "Beer CSS ghost shape was not rendered"
        assert pixel(394,224)==(18,18,22), "Beer CSS shape hole was filled"
    elif example == "launcher":
        assert pixel(30, 110) == (18, 18, 22), "Text button unexpectedly paints a filled container"
    else:
        assert pixel(30, 110) == (208, 188, 255), "Filled button was not rendered"
        assert pixel(380, 530) == (18, 18, 22), "Disabled text button unexpectedly paints a filled container"
        assert pixel(360, 220) == (18, 18, 22), "List content escaped its clipping region"
    if example!="effects":assert pixel(24, 96) == (18, 18, 22), "Rounded corner was not clipped"
    # A label must contain light glyph pixels, not just background.
    text_pixels = []
    for y in range(24 * scale, 48 * scale):
        start = (y * width + 36 * scale) * 3
        text_pixels.extend(pixels[start:start + 180 * scale * 3])
    assert max(text_pixels) > 180, "Text was not rendered"
    print(f"PASS: {example}, Wayland + GLES, scale {scale}, {width}x{height}, component/text/clipping pixels")


def main():
    if not shutil.which("weston"):
        raise SystemExit("weston is required for this smoke test")
    subprocess.run(["cargo", "build", "--locked", "--features", "wayland", "--examples"],
                   cwd=CRATE, check=True)
    for scale in (1, 2):
        with tempfile.TemporaryDirectory(prefix="bhoo-wayland-") as directory:
            runtime = Path(directory)
            runtime.chmod(0o700)
            log = runtime / "weston.log"
            capture = runtime / "frame.ppm"
            env = dict(os.environ, XDG_RUNTIME_DIR=str(runtime), WAYLAND_DISPLAY="bhoo-test",
                       BHOO_CAPTURE=str(capture), LIBGL_ALWAYS_SOFTWARE="1")
            compositor = subprocess.Popen([
                "weston", "--backend=headless", "--renderer=pixman", "--no-config",
                "--shell=desktop", "--socket=bhoo-test", "--idle-time=0",
                "--width=2000", "--height=1600", f"--scale={scale}", f"--log={log}",
            ], env=env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            try:
                deadline = time.monotonic() + 10
                while not (runtime / "bhoo-test").exists():
                    if compositor.poll() is not None or time.monotonic() >= deadline:
                        raise RuntimeError(log.read_text() if log.exists() else "Weston did not start")
                    time.sleep(0.05)
                for example, extra in [("image_reflection", []), ("split_outline", []), ("checkbox_native", []), ("launcher", []), ("components", []), ("components", ["--reduced-motion"]), ("effects", []), ("effects", ["--reduced-motion"])]:
                    subprocess.run([
                        "cargo", "run", "--locked", "--features", "wayland", "--example", example,
                        "--", "--smoke-test", *extra,
                    ], cwd=CRATE, env=env, check=True, timeout=30)
                    check_capture(capture, scale, example)
            finally:
                compositor.terminate()
                try:
                    compositor.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    compositor.kill()
                    compositor.wait()


if __name__ == "__main__":
    main()
