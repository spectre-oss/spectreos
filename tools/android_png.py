"""Decode bounded non-interlaced RGB, grayscale and indexed PNG drawables."""
import struct
import zlib


def decode_png(path):
    data = path.read_bytes()
    if not data.startswith(b'\x89PNG\r\n\x1a\n'):
        raise ValueError('Invalid PNG: ' + str(path))
    pos = 8
    parts = []
    header = None
    ended = False
    palette = None
    transparency = None
    while pos + 12 <= len(data):
        length = int.from_bytes(data[pos:pos+4], 'big')
        if length > len(data) - pos - 12:
            raise ValueError('Truncated PNG: ' + str(path))
        kind = data[pos+4:pos+8]
        payload = data[pos+8:pos+8+length]
        crc = int.from_bytes(data[pos+8+length:pos+12+length], 'big')
        if zlib.crc32(kind + payload) != crc:
            raise ValueError('PNG CRC mismatch: ' + str(path))
        pos += 12 + length
        if kind == b'IHDR':
            if header is not None or len(payload) != 13:
                raise ValueError('Invalid PNG header: ' + str(path))
            width, height, depth, color, compression, filtering, interlace = struct.unpack('>IIBBBBB', payload)
            depths = {0: (1, 2, 4, 8, 16), 2: (8, 16), 3: (1, 2, 4, 8), 4: (8, 16), 6: (8, 16)}
            if not (1 <= width <= 1024 and 1 <= height <= 1024) or color not in depths or depth not in depths[color] or compression or filtering or interlace:
                raise ValueError('Unsupported PNG format: ' + str(path))
            header = (width, height, depth, color)
        elif kind == b'PLTE':
            if not payload or len(payload) % 3 or len(payload) > 768:
                raise ValueError('Invalid PNG palette: ' + str(path))
            palette = payload
        elif kind == b'tRNS':
            transparency = payload
        elif kind == b'IDAT':
            parts.append(payload)
        elif kind == b'IEND':
            ended = True
            break
    if header is None or not parts or not ended:
        raise ValueError('Incomplete PNG: ' + str(path))
    width, height, depth, color = header
    channels = {0: 1, 2: 3, 3: 1, 4: 2, 6: 4}[color]
    if color == 3 and (palette is None or len(palette)//3 > 1 << depth):
        raise ValueError('Missing or invalid PNG palette: ' + str(path))
    if transparency is not None:
        valid = (color == 3 and len(transparency) <= len(palette)//3) or (color == 0 and len(transparency) == 2) or (color == 2 and len(transparency) == 6)
        if not valid:
            raise ValueError('Invalid PNG transparency: ' + str(path))
    stride = (width * channels * depth + 7) // 8
    filter_bytes = max(1, (channels * depth + 7) // 8)
    expected = (stride + 1) * height
    decoder = zlib.decompressobj()
    try:
        raw = decoder.decompress(b''.join(parts), expected + 1)
    except zlib.error as error:
        raise ValueError('Invalid PNG compressed data: ' + str(path)) from error
    if len(raw) != expected or not decoder.eof or decoder.unused_data:
        raise ValueError('Invalid PNG data length: ' + str(path))
    result = bytearray(width * height * 4)
    previous = bytearray(stride)
    for y in range(height):
        row = bytearray(raw[y*(stride+1)+1:(y+1)*(stride+1)])
        mode = raw[y*(stride+1)]
        if mode > 4:
            raise ValueError('Invalid PNG filter: ' + str(path))
        for x in range(stride):
            left = row[x-filter_bytes] if x >= filter_bytes else 0
            above = previous[x]
            upper_left = previous[x-filter_bytes] if x >= filter_bytes else 0
            p = left + above - upper_left
            distances = (abs(p-left), abs(p-above), abs(p-upper_left))
            paeth = left if distances[0] <= distances[1] and distances[0] <= distances[2] else above if distances[1] <= distances[2] else upper_left
            predictor = (0, left, above, (left+above)//2, paeth)[mode]
            row[x] = (row[x] + predictor) & 255
        for x in range(width):
            source = x * channels * (2 if depth == 16 else 1)
            target = (y * width + x) * 4
            if color in (0, 3):
                sample = int.from_bytes(row[source:source+2], 'big') if depth == 16 else (row[x * depth // 8] >> (8 - depth - (x * depth % 8))) & ((1 << depth) - 1)
                if color == 3:
                    if sample >= len(palette) // 3:
                        raise ValueError('PNG palette index out of range: ' + str(path))
                    rgb = palette[sample*3:sample*3+3]
                    alpha = transparency[sample] if transparency and sample < len(transparency) else 255
                else:
                    gray = sample * 255 // ((1 << depth) - 1)
                    rgb = bytes([gray, gray, gray])
                    alpha = 0 if transparency is not None and sample == int.from_bytes(transparency, 'big') else 255
            elif color == 4:
                gray, alpha = (row[source], row[source+2]) if depth == 16 else row[source:source+2]
                rgb = bytes([gray, gray, gray])
            else:
                samples = struct.unpack('>HHH', row[source:source+6]) if depth == 16 else tuple(row[source:source+3])
                rgb = bytes(value >> 8 if depth == 16 else value for value in samples)
                alpha = row[source+6] if color == 6 and depth == 16 else row[source+3] if color == 6 else 255
                if color == 2 and transparency is not None and samples == struct.unpack('>HHH', transparency):
                    alpha = 0
            result[target:target+4] = rgb + bytes([alpha])
        previous = row
    return width, height, result


def is_nine_patch(path):
    return str(path).endswith('.9.png')


def decode_nine_patch_bands(path):
    """Decode an Android .9.png, stripping its 1px guide border and extracting
    [left, top, right, bottom] content padding in pixels from the bottom and
    right 1px #FF000000 guide strips, plus all contiguous half-open (start, end)
    stretch band tuples in stripped-image pixel coordinates from the top and
    left #FF000000 strips."""
    width, height, pixels = decode_png(path)
    if width < 3 or height < 3:
        raise ValueError('Invalid 9-patch PNG dimensions: ' + str(path))
    inner_w = width - 2
    inner_h = height - 2
    def is_black(x, y):
        i = (y * width + x) * 4
        return pixels[i] == 0 and pixels[i + 1] == 0 and pixels[i + 2] == 0 and pixels[i + 3] == 255
    def extract_bands(marks, limit):
        if not marks:
            return [(0, limit)]
        bands = []
        start = prev = marks[0]
        for m in marks[1:]:
            if m == prev + 1:
                prev = m
            else:
                bands.append((start, prev + 1))
                start = prev = m
        bands.append((start, prev + 1))
        return bands
    top_marks = [x - 1 for x in range(1, width - 1) if is_black(x, 0)]
    left_marks = [y - 1 for y in range(1, height - 1) if is_black(0, y)]
    bottom_marks = [x - 1 for x in range(1, width - 1) if is_black(x, height - 1)]
    right_marks = [y - 1 for y in range(1, height - 1) if is_black(width - 1, y)]
    x_bands = extract_bands(top_marks, inner_w)
    y_bands = extract_bands(left_marks, inner_h)
    if bottom_marks:
        pad_left = bottom_marks[0]
        pad_right = (inner_w - 1) - bottom_marks[-1]
    else:
        pad_left = 0
        pad_right = 0
    if right_marks:
        pad_top = right_marks[0]
        pad_bottom = (inner_h - 1) - right_marks[-1]
    else:
        pad_top = 0
        pad_bottom = 0
    stripped = bytearray(inner_w * inner_h * 4)
    row_bytes = inner_w * 4
    for y in range(inner_h):
        src_off = ((y + 1) * width + 1) * 4
        dst_off = y * row_bytes
        stripped[dst_off:dst_off + row_bytes] = pixels[src_off:src_off + row_bytes]
    return inner_w, inner_h, stripped, [pad_left, pad_top, pad_right, pad_bottom], x_bands, y_bands


def decode_nine_patch_full(path):
    """Decode an Android .9.png, stripping its 1px guide border and extracting
    [left, top, right, bottom] content padding in pixels from the bottom and
    right 1px #FF000000 guide strips, plus half-open (start, end) stretch ranges
    in stripped-image pixel coordinates from the top and left #FF000000 strips."""
    inner_w, inner_h, stripped, padding_px, x_bands, y_bands = decode_nine_patch_bands(path)
    stretch_x = (x_bands[0][0], x_bands[-1][1])
    stretch_y = (y_bands[0][0], y_bands[-1][1])
    return inner_w, inner_h, stripped, padding_px, stretch_x, stretch_y


def decode_nine_patch(path):
    """Decode an Android .9.png, stripping its 1px guide border and extracting
    [left, top, right, bottom] content padding in pixels from the bottom and
    right 1px #FF000000 guide strips."""
    inner_w, inner_h, stripped, padding_px, _, _ = decode_nine_patch_full(path)
    return inner_w, inner_h, stripped, padding_px
