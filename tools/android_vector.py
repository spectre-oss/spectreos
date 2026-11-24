"""Rasterize a deliberately bounded, static Android VectorDrawable subset."""
import math, re, subprocess, tempfile, xml.etree.ElementTree as ET
from pathlib import Path
from android_png import decode_png

A = '{http://schemas.android.com/apk/res/android}'
AAPT = '{http://schemas.android.com/aapt}'
VALID_TINT_MODES = {'src_in', 'src_atop', 'multiply', 'screen', 'add', 'src_over'}


def vector_tint_mode(path):
    root = ET.parse(path).getroot()
    mode = root.get(A + 'tintMode', 'src_in')
    if mode not in VALID_TINT_MODES:
        raise ValueError('Unsupported vector tintMode ' + mode)
    return mode


def vector_auto_mirrored(path):
    root = ET.parse(path).getroot()
    am = root.get(A + 'autoMirrored')
    if am is None:
        return False
    if am not in {'true', 'false'}:
        raise ValueError('Invalid vector autoMirrored ' + am)
    return am == 'true'


def vector_tint(path):
    root = ET.parse(path).getroot()
    tint = root.get(A + 'tint')
    mode = root.get(A + 'tintMode', 'src_in')
    if mode not in VALID_TINT_MODES:
        raise ValueError('Unsupported vector tintMode ' + mode)
    if tint is None:
        return None
    if tint in ('?attr/colorControlNormal', '?android:attr/colorControlNormal'):
        return 'controlNormal'
    if tint in ('@android:color/black', '@android:color/white'):
        return '#ff000000' if tint.endswith('black') else '#ffffffff'
    if re.fullmatch(r'#[0-9a-fA-F]{6}', tint):
        return '#ff' + tint[1:]
    if re.fullmatch(r'#[0-9a-fA-F]{8}', tint):
        return tint.lower()
    raise ValueError('Unsupported vector tint ' + tint)


def _flatten_arc(x0, y0, rx, ry, phi_deg, fa, fs, x1, y1):
    """Flatten an SVG elliptical arc command into polyline points (excluding start)."""
    if abs(x0 - x1) < 1e-9 and abs(y0 - y1) < 1e-9:
        return []
    rx, ry = abs(rx), abs(ry)
    if rx < 1e-9 or ry < 1e-9:
        return [(x1, y1)]
    phi = math.radians(phi_deg % 360.0)
    cos_p, sin_p = math.cos(phi), math.sin(phi)
    dx2, dy2 = (x0 - x1) * 0.5, (y0 - y1) * 0.5
    x1p = cos_p * dx2 + sin_p * dy2
    y1p = -sin_p * dx2 + cos_p * dy2
    rx2, ry2 = rx * rx, ry * ry
    x1p2, y1p2 = x1p * x1p, y1p * y1p
    lam = x1p2 / rx2 + y1p2 / ry2
    if lam > 1.0:
        s = math.sqrt(lam)
        rx *= s
        ry *= s
        rx2, ry2 = rx * rx, ry * ry
    denom = rx2 * y1p2 + ry2 * x1p2
    num = max(0.0, rx2 * ry2 - denom)
    coef = math.sqrt(num / denom) if denom > 1e-12 else 0.0
    if fa == fs:
        coef = -coef
    cxp = coef * (rx * y1p / ry)
    cyp = coef * (-ry * x1p / rx)
    cx = cos_p * cxp - sin_p * cyp + (x0 + x1) * 0.5
    cy = sin_p * cxp + cos_p * cyp + (y0 + y1) * 0.5

    def vec_angle(ux, uy, vx, vy):
        dot = ux * vx + uy * vy
        mag = math.hypot(ux, uy) * math.hypot(vx, vy)
        if mag < 1e-12:
            return 0.0
        ang = math.acos(max(-1.0, min(1.0, dot / mag)))
        return -ang if (ux * vy - uy * vx) < 0 else ang

    vx0, vy0 = (x1p - cxp) / rx, (y1p - cyp) / ry
    vx1, vy1 = (-x1p - cxp) / rx, (-y1p - cyp) / ry
    theta1 = vec_angle(1.0, 0.0, vx0, vy0)
    dtheta = vec_angle(vx0, vy0, vx1, vy1)
    if not fs and dtheta > 0:
        dtheta -= 2.0 * math.pi
    elif fs and dtheta < 0:
        dtheta += 2.0 * math.pi
    steps = max(8, int(math.ceil(abs(dtheta) / (math.pi / 18.0))))
    pts = []
    for i in range(1, steps + 1):
        ang = theta1 + dtheta * (i / steps)
        ca, sa = math.cos(ang), math.sin(ang)
        px = cos_p * (rx * ca) - sin_p * (ry * sa) + cx
        py = sin_p * (rx * ca) + cos_p * (ry * sa) + cy
        pts.append((px, py))
    return pts


def _parse_path_polylines(data):
    """Parse SVG pathData into a list of polylines (each a list of (x, y) tuples)."""
    num_re = re.compile(r'[+-]?(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][+-]?\d+)?')
    n = len(data)
    i = 0

    def skip_ws_comma():
        nonlocal i
        while i < n and data[i] in ' \t\r\n,':
            i += 1

    def read_num():
        nonlocal i
        skip_ws_comma()
        m = num_re.match(data, i)
        if not m:
            raise ValueError('Invalid vector pathData')
        i = m.end()
        return float(m.group(0))

    def read_flag():
        nonlocal i
        skip_ws_comma()
        if i >= n or data[i] not in '01':
            raise ValueError('Invalid vector pathData')
        v = int(data[i])
        i += 1
        return v

    def has_more_args():
        skip_ws_comma()
        return i < n and not data[i].isalpha()

    polylines = []
    cur_poly = None
    cx = cy = 0.0
    sx = sy = 0.0
    prev_cmd = ''
    last_cubic_ctrl = None
    last_quad_ctrl = None

    while True:
        skip_ws_comma()
        if i >= n:
            break
        ch = data[i]
        if ch.isalpha():
            cmd = ch
            i += 1
        elif prev_cmd:
            cmd = 'L' if prev_cmd == 'M' else 'l' if prev_cmd == 'm' else prev_cmd
        else:
            raise ValueError('Invalid vector pathData')

        if cmd in ('M', 'm'):
            x, y = read_num(), read_num()
            if cmd == 'm':
                x += cx
                y += cy
            cx, cy = x, y
            sx, sy = x, y
            cur_poly = [(cx, cy)]
            polylines.append(cur_poly)
            last_cubic_ctrl = last_quad_ctrl = None
            prev_cmd = cmd
            while has_more_args():
                lx, ly = read_num(), read_num()
                if cmd == 'm':
                    lx += cx
                    ly += cy
                cx, cy = lx, ly
                cur_poly.append((cx, cy))
                prev_cmd = 'l' if cmd == 'm' else 'L'
        elif cmd in ('Z', 'z'):
            if cur_poly is not None:
                if abs(cx - sx) > 1e-9 or abs(cy - sy) > 1e-9:
                    cur_poly.append((sx, sy))
                cx, cy = sx, sy
            last_cubic_ctrl = last_quad_ctrl = None
            prev_cmd = cmd
        elif cmd in ('L', 'l'):
            if cur_poly is None:
                cur_poly = [(cx, cy)]
                polylines.append(cur_poly)
            while True:
                x, y = read_num(), read_num()
                if cmd == 'l':
                    x += cx
                    y += cy
                cx, cy = x, y
                cur_poly.append((cx, cy))
                if not has_more_args():
                    break
            last_cubic_ctrl = last_quad_ctrl = None
            prev_cmd = cmd
        elif cmd in ('H', 'h'):
            if cur_poly is None:
                cur_poly = [(cx, cy)]
                polylines.append(cur_poly)
            while True:
                x = read_num()
                if cmd == 'h':
                    x += cx
                cx = x
                cur_poly.append((cx, cy))
                if not has_more_args():
                    break
            last_cubic_ctrl = last_quad_ctrl = None
            prev_cmd = cmd
        elif cmd in ('V', 'v'):
            if cur_poly is None:
                cur_poly = [(cx, cy)]
                polylines.append(cur_poly)
            while True:
                y = read_num()
                if cmd == 'v':
                    y += cy
                cy = y
                cur_poly.append((cx, cy))
                if not has_more_args():
                    break
            last_cubic_ctrl = last_quad_ctrl = None
            prev_cmd = cmd
        elif cmd in ('C', 'c'):
            if cur_poly is None:
                cur_poly = [(cx, cy)]
                polylines.append(cur_poly)
            while True:
                x1, y1, x2, y2, x, y = read_num(), read_num(), read_num(), read_num(), read_num(), read_num()
                if cmd == 'c':
                    x1 += cx; y1 += cy; x2 += cx; y2 += cy; x += cx; y += cy
                steps = 24
                for s in range(1, steps + 1):
                    t = s / steps
                    mt = 1.0 - t
                    px = mt*mt*mt*cx + 3*mt*mt*t*x1 + 3*mt*t*t*x2 + t*t*t*x
                    py = mt*mt*mt*cy + 3*mt*mt*t*y1 + 3*mt*t*t*y2 + t*t*t*y
                    cur_poly.append((px, py))
                cx, cy = x, y
                last_cubic_ctrl = (x2, y2)
                last_quad_ctrl = None
                if not has_more_args():
                    break
            prev_cmd = cmd
        elif cmd in ('S', 's'):
            if cur_poly is None:
                cur_poly = [(cx, cy)]
                polylines.append(cur_poly)
            while True:
                x2, y2, x, y = read_num(), read_num(), read_num(), read_num()
                if cmd == 's':
                    x2 += cx; y2 += cy; x += cx; y += cy
                if last_cubic_ctrl is not None:
                    x1, y1 = 2.0 * cx - last_cubic_ctrl[0], 2.0 * cy - last_cubic_ctrl[1]
                else:
                    x1, y1 = cx, cy
                steps = 24
                for s in range(1, steps + 1):
                    t = s / steps
                    mt = 1.0 - t
                    px = mt*mt*mt*cx + 3*mt*mt*t*x1 + 3*mt*t*t*x2 + t*t*t*x
                    py = mt*mt*mt*cy + 3*mt*mt*t*y1 + 3*mt*t*t*y2 + t*t*t*y
                    cur_poly.append((px, py))
                cx, cy = x, y
                last_cubic_ctrl = (x2, y2)
                last_quad_ctrl = None
                if not has_more_args():
                    break
            prev_cmd = cmd
        elif cmd in ('Q', 'q'):
            if cur_poly is None:
                cur_poly = [(cx, cy)]
                polylines.append(cur_poly)
            while True:
                x1, y1, x, y = read_num(), read_num(), read_num(), read_num()
                if cmd == 'q':
                    x1 += cx; y1 += cy; x += cx; y += cy
                steps = 20
                for s in range(1, steps + 1):
                    t = s / steps
                    mt = 1.0 - t
                    px = mt*mt*cx + 2*mt*t*x1 + t*t*x
                    py = mt*mt*cy + 2*mt*t*y1 + t*t*y
                    cur_poly.append((px, py))
                cx, cy = x, y
                last_quad_ctrl = (x1, y1)
                last_cubic_ctrl = None
                if not has_more_args():
                    break
            prev_cmd = cmd
        elif cmd in ('T', 't'):
            if cur_poly is None:
                cur_poly = [(cx, cy)]
                polylines.append(cur_poly)
            while True:
                x, y = read_num(), read_num()
                if cmd == 't':
                    x += cx; y += cy
                if last_quad_ctrl is not None:
                    x1, y1 = 2.0 * cx - last_quad_ctrl[0], 2.0 * cy - last_quad_ctrl[1]
                else:
                    x1, y1 = cx, cy
                steps = 20
                for s in range(1, steps + 1):
                    t = s / steps
                    mt = 1.0 - t
                    px = mt*mt*cx + 2*mt*t*x1 + t*t*x
                    py = mt*mt*cy + 2*mt*t*y1 + t*t*y
                    cur_poly.append((px, py))
                cx, cy = x, y
                last_quad_ctrl = (x1, y1)
                last_cubic_ctrl = None
                if not has_more_args():
                    break
            prev_cmd = cmd
        elif cmd in ('A', 'a'):
            if cur_poly is None:
                cur_poly = [(cx, cy)]
                polylines.append(cur_poly)
            while True:
                rx, ry, rot = read_num(), read_num(), read_num()
                fa, fs = read_flag(), read_flag()
                x, y = read_num(), read_num()
                if cmd == 'a':
                    x += cx; y += cy
                cur_poly.extend(_flatten_arc(cx, cy, rx, ry, rot, fa, fs, x, y))
                cx, cy = x, y
                last_cubic_ctrl = last_quad_ctrl = None
                if not has_more_args():
                    break
            prev_cmd = cmd
        else:
            raise ValueError('Invalid vector pathData')
    return polylines


def _trim_path_data(data, trim_start, trim_end, trim_offset):
    """Trim an SVG path string using Android trimPathStart/End/Offset semantics."""
    if trim_start == 0.0 and trim_end == 1.0:
        return data
    start = (trim_start + trim_offset) % 1.0
    end = (trim_end + trim_offset) % 1.0
    if abs(start - end) < 1e-9:
        return ''
    if start < end:
        ranges = [(start, end)]
    else:
        ranges = []
        if start < 1.0 - 1e-9:
            ranges.append((start, 1.0))
        if end > 1e-9:
            ranges.append((0.0, end))
    polylines = _parse_path_polylines(data)
    segments = []
    total_len = 0.0
    for sub_idx, poly in enumerate(polylines):
        for idx in range(len(poly) - 1):
            p0, p1 = poly[idx], poly[idx + 1]
            seg_len = math.hypot(p1[0] - p0[0], p1[1] - p0[1])
            if seg_len > 1e-9:
                segments.append((total_len, total_len + seg_len, p0, p1, sub_idx))
                total_len += seg_len
    if total_len <= 1e-9:
        return ''
    cmds = []
    for r0, r1 in ranges:
        d0, d1 = r0 * total_len, r1 * total_len
        prev_pt = None
        prev_sub = -1
        for s0, s1, p0, p1, sub_idx in segments:
            if s1 <= d0 + 1e-9 or s0 >= d1 - 1e-9:
                continue
            seg_len = s1 - s0
            t0 = max(0.0, (d0 - s0) / seg_len)
            t1 = min(1.0, (d1 - s0) / seg_len)
            q0 = (p0[0] + (p1[0] - p0[0]) * t0, p0[1] + (p1[1] - p0[1]) * t0)
            q1 = (p0[0] + (p1[0] - p0[0]) * t1, p0[1] + (p1[1] - p0[1]) * t1)
            if prev_pt is None or sub_idx != prev_sub or abs(q0[0] - prev_pt[0]) > 1e-6 or abs(q0[1] - prev_pt[1]) > 1e-6:
                cmds.append(f'M{q0[0]:.4f} {q0[1]:.4f}')
            cmds.append(f'L{q1[0]:.4f} {q1[1]:.4f}')
            prev_pt = q1
            prev_sub = sub_idx
    return ' '.join(cmds)


def _sample_stops(stops, t):
    """Interpolate (r, g, b, alpha) at normalized offset t in [0, 1]."""
    if t <= stops[0][0]:
        return stops[0][1]
    if t >= stops[-1][0]:
        return stops[-1][1]
    for idx in range(len(stops) - 1):
        o0, c0 = stops[idx]
        o1, c1 = stops[idx + 1]
        if o0 <= t <= o1:
            span = o1 - o0
            f = 0.0 if span <= 1e-9 else (t - o0) / span
            return (
                int(round(c0[0] + (c1[0] - c0[0]) * f)),
                int(round(c0[1] + (c1[1] - c0[1]) * f)),
                int(round(c0[2] + (c1[2] - c0[2]) * f)),
                c0[3] + (c1[3] - c0[3]) * f,
            )
    return stops[-1][1]


def rasterize_vector(path, density=160):
    root = ET.parse(path).getroot()
    if root.tag != 'vector':
        raise ValueError('Expected vector drawable: ' + str(path))

    def attrs(element, allowed):
        if any(not key.startswith(A) or key[len(A):] not in allowed for key in element.attrib):
            raise ValueError('Unsupported vector attribute')

    attrs(root, {'width', 'height', 'viewportWidth', 'viewportHeight', 'alpha', 'tint', 'tintMode', 'autoMirrored'})
    vector_tint(path)
    vector_auto_mirrored(path)

    def number(value):
        if not re.fullmatch(r'(?:\d+(?:\.\d*)?|\.\d+)', value or ''):
            raise ValueError('Invalid vector dimension')
        return float(value)

    def signed_number(value):
        if not re.fullmatch(r'[+-]?(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][+-]?\d+)?', value or ''):
            raise ValueError('Invalid vector transform')
        return float(value)

    def dimension(value):
        match = re.fullmatch(r'(\d+(?:\.\d*)?|\.\d+)dp', value or '')
        if not match:
            raise ValueError('Vector dimensions must use dp')
        return float(match.group(1))

    w = dimension(root.get(A + 'width'))
    h = dimension(root.get(A + 'height'))
    vw = number(root.get(A + 'viewportWidth'))
    vh = number(root.get(A + 'viewportHeight'))
    width = round(w * density / 160)
    height = round(h * density / 160)
    if not (1 <= width <= 1024 and 1 <= height <= 1024 and vw > 0 and vh > 0):
        raise ValueError('Vector dimensions out of range')
    svg = ET.Element('svg', {'xmlns': 'http://www.w3.org/2000/svg', 'width': str(width), 'height': str(height), 'viewBox': f'0 0 {vw} {vh}'})

    def color(value):
        if value == '@android:color/transparent':
            return '#000000', 0.0
        if value in ('@android:color/black', '@android:color/white'):
            return ('#000000' if value.endswith('black') else '#ffffff', 1.0)
        if not re.fullmatch(r'#(?:[0-9a-fA-F]{3}|[0-9a-fA-F]{4}|[0-9a-fA-F]{6}|[0-9a-fA-F]{8})', value or ''):
            raise ValueError('Unsupported vector color ' + str(value))
        raw = value[1:]
        if len(raw) in (3, 4):
            raw = ''.join(ch * 2 for ch in raw)
        if len(raw) == 8:
            return '#' + raw[2:], int(raw[:2], 16) / 255.0
        return '#' + raw, 1.0

    root_alpha = number(root.get(A + 'alpha', '1'))
    if root_alpha > 1:
        raise ValueError('Invalid vector alpha')
    definitions = ET.SubElement(svg, 'defs')
    layer = ET.SubElement(svg, 'g', {'opacity': str(root_alpha)})
    path_count = 0
    clip_count = 0
    grad_count = 0

    def parse_gradient(grad):
        nonlocal grad_count
        attrs(grad, {'type', 'startX', 'startY', 'endX', 'endY', 'centerX', 'centerY', 'gradientRadius', 'startColor', 'centerColor', 'endColor', 'tileMode'})
        raw_type = grad.get(A + 'type', 'linear')
        gtype = {'0': 'linear', '1': 'radial', '2': 'sweep'}.get(raw_type, raw_type)
        if gtype not in {'linear', 'radial', 'sweep'}:
            raise ValueError('Unsupported vector gradient type ' + str(raw_type))
        raw_tile = grad.get(A + 'tileMode', 'clamp')
        spread = {'clamp': 'pad', 'repeat': 'repeat', 'mirror': 'reflect', '0': 'pad', '1': 'repeat', '2': 'reflect'}.get(raw_tile)
        if spread is None:
            raise ValueError('Unsupported vector gradient tileMode ' + str(raw_tile))
        stops = []
        if len(grad):
            for item in grad:
                if item.tag != 'item' or len(item):
                    raise ValueError('Unsupported vector gradient item')
                attrs(item, {'offset', 'color'})
                if A + 'offset' not in item.attrib or A + 'color' not in item.attrib:
                    raise ValueError('Vector gradient item requires offset and color')
                off = signed_number(item.get(A + 'offset'))
                if not 0.0 <= off <= 1.0:
                    raise ValueError('Invalid vector gradient offset')
                hex_rgb, alpha = color(item.get(A + 'color'))
                stops.append((off, hex_rgb, alpha))
            stops.sort(key=lambda s: s[0])
        else:
            if A + 'startColor' in grad.attrib:
                hex_rgb, alpha = color(grad.get(A + 'startColor'))
                stops.append((0.0, hex_rgb, alpha))
            if A + 'centerColor' in grad.attrib:
                hex_rgb, alpha = color(grad.get(A + 'centerColor'))
                stops.append((0.5, hex_rgb, alpha))
            if A + 'endColor' in grad.attrib:
                hex_rgb, alpha = color(grad.get(A + 'endColor'))
                stops.append((1.0, hex_rgb, alpha))
        if not stops:
            raise ValueError('Empty vector gradient')
        grad_count += 1
        gid = f'grad{grad_count}'
        if gtype == 'linear':
            x1 = signed_number(grad.get(A + 'startX', '0'))
            y1 = signed_number(grad.get(A + 'startY', '0'))
            x2 = signed_number(grad.get(A + 'endX', '0'))
            y2 = signed_number(grad.get(A + 'endY', '0'))
            gel = ET.SubElement(definitions, 'linearGradient', {
                'id': gid, 'gradientUnits': 'userSpaceOnUse',
                'x1': str(x1), 'y1': str(y1), 'x2': str(x2), 'y2': str(y2),
                'spreadMethod': spread,
            })
            for off, hex_rgb, alpha in stops:
                ET.SubElement(gel, 'stop', {'offset': str(off), 'stop-color': hex_rgb, 'stop-opacity': str(alpha)})
        elif gtype == 'radial':
            cx = signed_number(grad.get(A + 'centerX', '0'))
            cy = signed_number(grad.get(A + 'centerY', '0'))
            r = number(grad.get(A + 'gradientRadius', '0'))
            if r <= 0.0:
                raise ValueError('Invalid vector gradient radius')
            gel = ET.SubElement(definitions, 'radialGradient', {
                'id': gid, 'gradientUnits': 'userSpaceOnUse',
                'cx': str(cx), 'cy': str(cy), 'r': str(r),
                'spreadMethod': spread,
            })
            for off, hex_rgb, alpha in stops:
                ET.SubElement(gel, 'stop', {'offset': str(off), 'stop-color': hex_rgb, 'stop-opacity': str(alpha)})
        else:
            cx = signed_number(grad.get(A + 'centerX', '0'))
            cy = signed_number(grad.get(A + 'centerY', '0'))
            rgba_stops = [(off, (int(hx[1:3], 16), int(hx[3:5], 16), int(hx[5:7], 16), alpha)) for off, hx, alpha in stops]
            radius = math.hypot(vw, vh) * 2.0 + abs(cx) + abs(cy) + 1.0
            pat = ET.SubElement(definitions, 'pattern', {
                'id': gid, 'patternUnits': 'userSpaceOnUse',
                'x': '0', 'y': '0', 'width': str(vw), 'height': str(vh),
                'shape-rendering': 'crispEdges',
            })
            wedges = 360
            for idx in range(wedges):
                t = (idx + 0.5) / wedges
                a0 = (idx / wedges) * 2.0 * math.pi
                a1 = ((idx + 1) / wedges) * 2.0 * math.pi
                x0 = cx + radius * math.cos(a0)
                y0 = cy + radius * math.sin(a0)
                x1 = cx + radius * math.cos(a1)
                y1 = cy + radius * math.sin(a1)
                r, g, b, al = _sample_stops(rgba_stops, t)
                ET.SubElement(pat, 'path', {
                    'd': f'M{cx:.4f} {cy:.4f} L{x0:.4f} {y0:.4f} L{x1:.4f} {y1:.4f} Z',
                    'fill': f'#{r:02x}{g:02x}{b:02x}',
                    'fill-opacity': f'{al:.4f}',
                })
        return gid

    def path_data(element):
        data = element.get(A + 'pathData', '')
        if not data or not re.fullmatch(r'[MmZzLlHhVvCcSsQqTtAa0-9eE.,+\-\s]+', data):
            raise ValueError('Invalid vector pathData')
        return data

    def visit_children(parent, elements, depth):
        nonlocal clip_count
        current = parent
        for element in elements:
            if element.tag == 'clip-path':
                if len(element):
                    raise ValueError('Unsupported vector clip children')
                attrs(element, {'name', 'pathData'})
                clip_count += 1
                if clip_count > 1024:
                    raise ValueError('Too many vector clips')
                name = 'clip' + str(clip_count)
                clip = ET.SubElement(definitions, 'clipPath', {'id': name, 'clipPathUnits': 'userSpaceOnUse'})
                ET.SubElement(clip, 'path', {'d': path_data(element)})
                current = ET.SubElement(current, 'g', {'clip-path': 'url(#' + name + ')'})
            else:
                visit(current, element, depth)

    def visit(parent, element, depth=0):
        nonlocal path_count
        if depth > 32:
            raise ValueError('Vector nesting too deep')
        if element.tag == 'group':
            attrs(element, {'name', 'rotation', 'pivotX', 'pivotY', 'scaleX', 'scaleY', 'translateX', 'translateY'})
            def v(key, default):
                return signed_number(element.get(A + key, str(default)))
            px = v('pivotX', 0); py = v('pivotY', 0); rotation = v('rotation', 0)
            sx = v('scaleX', 1); sy = v('scaleY', 1); tx = v('translateX', 0); ty = v('translateY', 0)
            child = ET.SubElement(parent, 'g', {'transform': f'translate({tx+px} {ty+py}) rotate({rotation}) scale({sx} {sy}) translate({-px} {-py})'})
            visit_children(child, element, depth + 1)
            return
        if element.tag != 'path':
            raise ValueError('Unsupported vector element')
        path_count += 1
        if path_count > 1024:
            raise ValueError('Too many vector paths')
        path_node = element
        attrs(path_node, {
            'name', 'pathData', 'fillColor', 'fillAlpha', 'fillType',
            'strokeColor', 'strokeAlpha', 'strokeWidth', 'strokeLineCap',
            'strokeLineJoin', 'strokeMiterLimit',
            'trimPathStart', 'trimPathEnd', 'trimPathOffset',
        })
        aapt_grads = {}
        for child in path_node:
            if child.tag != AAPT + 'attr':
                raise ValueError('Unsupported vector element')
            if set(child.attrib) != {'name'} or len(child) != 1 or child[0].tag != 'gradient':
                raise ValueError('Unsupported vector aapt:attr')
            aname = child.attrib['name']
            if aname not in {'android:fillColor', 'android:strokeColor'}:
                raise ValueError('Unsupported vector aapt:attr ' + aname)
            if aname in aapt_grads:
                raise ValueError('Duplicate vector aapt:attr ' + aname)
            aapt_grads[aname] = parse_gradient(child[0])
        data = path_data(path_node)
        trim_start = signed_number(path_node.get(A + 'trimPathStart', '0'))
        trim_end = signed_number(path_node.get(A + 'trimPathEnd', '1'))
        trim_offset = signed_number(path_node.get(A + 'trimPathOffset', '0'))
        if not (0.0 <= trim_start <= 1.0 and 0.0 <= trim_end <= 1.0):
            raise ValueError('Invalid vector trimPath')
        if trim_start != 0.0 or trim_end != 1.0:
            data = _trim_path_data(data, trim_start, trim_end, trim_offset)
            if not data:
                return
        options = {'d': data}
        if 'android:fillColor' in aapt_grads:
            opacity = number(path_node.get(A + 'fillAlpha', '1'))
            if opacity > 1:
                raise ValueError('Invalid vector fill alpha')
            options['fill'] = 'url(#' + aapt_grads['android:fillColor'] + ')'
            options['fill-opacity'] = str(opacity)
        else:
            fill = path_node.get(A + 'fillColor')
            if fill is None:
                options['fill'] = 'none'
            else:
                value, alpha = color(fill)
                options['fill'] = value
                opacity = number(path_node.get(A + 'fillAlpha', '1')) * alpha
                if opacity > 1:
                    raise ValueError('Invalid vector fill alpha')
                options['fill-opacity'] = str(opacity)
        fill_type = path_node.get(A + 'fillType', 'nonZero')
        if fill_type not in ('nonZero', 'evenOdd'):
            raise ValueError('Unsupported vector fill type')
        options['fill-rule'] = 'evenodd' if fill_type == 'evenOdd' else 'nonzero'
        stroke = path_node.get(A + 'strokeColor')
        if 'android:strokeColor' in aapt_grads or stroke is not None:
            if 'android:strokeColor' in aapt_grads:
                options['stroke'] = 'url(#' + aapt_grads['android:strokeColor'] + ')'
                opacity = number(path_node.get(A + 'strokeAlpha', '1'))
            else:
                value, alpha = color(stroke)
                options['stroke'] = value
                opacity = number(path_node.get(A + 'strokeAlpha', '1')) * alpha
            if opacity > 1:
                raise ValueError('Invalid vector stroke alpha')
            options['stroke-opacity'] = str(opacity)
            options['stroke-width'] = str(number(path_node.get(A + 'strokeWidth', '0')))
            for android, svg_name, allowed in [('strokeLineCap', 'stroke-linecap', ('butt', 'round', 'square')), ('strokeLineJoin', 'stroke-linejoin', ('miter', 'round', 'bevel'))]:
                if A + android in path_node.attrib:
                    val = path_node.attrib[A + android]
                    if val not in allowed:
                        raise ValueError('Unsupported vector stroke option')
                    options[svg_name] = val
            if A + 'strokeMiterLimit' in path_node.attrib:
                options['stroke-miterlimit'] = str(number(path_node.attrib[A + 'strokeMiterLimit']))
        ET.SubElement(parent, 'path', options)

    visit_children(layer, root, 0)
    if not path_count:
        raise ValueError('Vector must contain a path')
    try:
        with tempfile.TemporaryDirectory() as tmp:
            out = Path(tmp) / 'vector.png'
            subprocess.run(['rsvg-convert', '--format=png', '--output', str(out)], input=ET.tostring(svg), check=True, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE)
            actual_w, actual_h, pixels = decode_png(out)
    except FileNotFoundError as error:
        raise ValueError('Vector import requires rsvg-convert') from error
    except subprocess.CalledProcessError as error:
        raise ValueError('Invalid vector path') from error
    if (actual_w, actual_h) != (width, height):
        raise ValueError('Unexpected vector raster size')
    return actual_w, actual_h, pixels, 160.0 / density
