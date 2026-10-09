import json
import math
from pathlib import Path
import struct
import subprocess
import sys

SIZE = 512
SOURCE = Path('/opt/steamvr/drivers/frame_controller/resources/rendermodels')


def rotate(value, angles):
    x, y, z = value
    for axis, angle in reversed(list(enumerate(angles))):
        c, s = math.cos(-angle), math.sin(-angle)
        if axis == 0:
            y, z = c * y - s * z, s * y + c * z
        elif axis == 1:
            x, z = c * x + s * z, -s * x + c * z
        else:
            x, y = c * x - s * y, s * x + c * y
    return x, y, z


def srgb(value):
    value /= 255
    return value / 12.92 if value <= 0.04045 else ((value + 0.055) / 1.055) ** 2.4


def model(hand):
    name = 'frame_controller_' + hand
    folder = SOURCE / name
    info = json.loads((folder / (name + '.json')).read_text())
    grip = info['components']['grip']['component_local']
    origin = grip['origin']
    angles = [math.radians(angle) for angle in grip['rotate_xyz']]
    texture = subprocess.run([
        'ffmpeg', '-v', 'error', '-i', str(folder / (name + '_color.png')),
        '-vf', f'scale={SIZE}:{SIZE}', '-f', 'rawvideo', '-pix_fmt', 'rgba', '-',
    ], check=True, stdout=subprocess.PIPE).stdout
    if len(texture) != SIZE * SIZE * 4:
        raise ValueError('Unexpected controller texture size')
    positions, normals, coordinates, vertices = [], [], [], bytearray()

    def vertex(reference):
        indices = [int(part) for part in reference.split('/')]
        p = positions[indices[0] - 1]
        uv = coordinates[indices[1] - 1]
        n = normals[indices[2] - 1]
        p = rotate([p[i] - origin[i] for i in range(3)], angles)
        n = rotate(n, angles)
        x = min(SIZE - 1, max(0, int(uv[0] * SIZE)))
        y = min(SIZE - 1, max(0, int((1 - uv[1]) * SIZE)))
        offset = (y * SIZE + x) * 4
        color = [srgb(channel) for channel in texture[offset:offset + 3]]
        return struct.pack('<12f', *p, 1, *n, 0, *color, 1)

    for line in (folder / (name + '.obj')).read_text().splitlines():
        fields = line.split()
        if not fields:
            continue
        kind, values = fields[0], fields[1:]
        if kind == 'v':
            positions.append(tuple(map(float, values[:3])))
        elif kind == 'vn':
            normals.append(tuple(map(float, values[:3])))
        elif kind == 'vt':
            coordinates.append(tuple(map(float, values[:2])))
        elif kind == 'f':
            face = [vertex(reference) for reference in values]
            for i in range(1, len(face) - 1):
                vertices.extend(face[0] + face[i] + face[i + 1])
    if not vertices or len(vertices) > 32 * 1024 * 1024:
        raise ValueError('Unexpected controller mesh size')
    return vertices


if __name__ == '__main__':
    destination = Path(sys.argv[1])
    left, right = model('left'), model('right')
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_bytes(b'HFCM0001' + struct.pack('<2I', len(left) // 48,
                                                    len(right) // 48) + left + right)
    print(f'Prepared Steam controller models: {len(left) // 144} left triangles, '
          f'{len(right) // 144} right triangles')
