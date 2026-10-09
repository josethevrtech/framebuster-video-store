from pathlib import Path
import math
import struct
import sys


def model(folder, name, origin, scale, yaw):
    materials = {}
    material = None
    for line in (folder / (name + '.mtl')).read_text().splitlines():
        fields = line.split()
        if not fields:
            continue
        if fields[0] == 'newmtl':
            material = fields[1]
        elif fields[0] == 'Kd':
            materials[material] = tuple(map(float, fields[1:4]))
    positions, normals, faces = [], [], []
    color = (0.5, 0.5, 0.5)
    for line in (folder / (name + '.obj')).read_text().splitlines():
        fields = line.split()
        if not fields:
            continue
        if fields[0] == 'v':
            positions.append(tuple(map(float, fields[1:4])))
        elif fields[0] == 'vn':
            normals.append(tuple(map(float, fields[1:4])))
        elif fields[0] == 'usemtl':
            color = materials[fields[1]]
        elif fields[0] == 'f':
            face = [tuple(int(i) if i else 0 for i in ref.split('/')) for ref in fields[1:]]
            for i in range(1, len(face) - 1):
                faces.append(([face[0], face[i], face[i + 1]], color))
    floor = min(p[1] for p in positions)
    c, s = math.cos(yaw), math.sin(yaw)

    def rotate(p):
        return c * p[0] + s * p[2], p[1], -s * p[0] + c * p[2]

    result = bytearray()
    for face, color in faces:
        for ref in face:
            p = positions[ref[0] - 1]
            p = rotate((p[0] * scale, (p[1] - floor) * scale, p[2] * scale))
            n = rotate(normals[ref[-1] - 1])
            p = [p[i] + origin[i] for i in range(3)]
            values = (*p, 1, *n, 0, *color, 1)
            if not all(math.isfinite(x) for x in values):
                raise ValueError('Invalid asset vertex')
            result.extend(struct.pack('<12f', *values))
    return result


if __name__ == '__main__':
    folder, output = Path(sys.argv[1]), Path(sys.argv[2])
    data = bytearray()
    for name, position, scale, yaw in [
        ('televisionVintage', (-1.25, -0.42, 23.0), 1.15, math.pi),
        ('pottedPlant', (-4.0, -1.5, 23.7), 1.3, 0),
        ('pottedPlant', (4.0, -1.5, 23.7), 1.3, 0),
        ('chairCushion', (2.9, -1.5, 21.8), 1.2, math.pi / 2),
    ]:
        data.extend(model(folder, name, position, scale, yaw))
    if not data or len(data) > 4 * 1024 * 1024:
        raise ValueError('Unexpected prop mesh size')
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_bytes(b'FBPROP01' + struct.pack('<I', len(data) // 48) + data)
    print(f'Prepared offline furniture: {len(data) // 144} triangles')
