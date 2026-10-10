from pathlib import Path
import json
import math
import struct
import sys


def values(scene, data, index):
    accessor = scene['accessors'][index]
    view = scene['bufferViews'][accessor['bufferView']]
    components = {'SCALAR': 1, 'VEC2': 2, 'VEC3': 3}[accessor['type']]
    fmt = '<' + {5126: 'f', 5123: 'H', 5125: 'I'}[accessor['componentType']] * components
    stride = view.get('byteStride', struct.calcsize(fmt))
    offset = view.get('byteOffset', 0) + accessor.get('byteOffset', 0)
    return [struct.unpack_from(fmt, data, offset + i * stride) for i in range(accessor['count'])]


def bake(folder, placements):
    scene = json.loads((folder / 'television_02_1k.gltf').read_text())
    data = (folder / scene['buffers'][0]['uri']).read_bytes()
    result = bytearray()
    for origin, scale, yaw in placements:
        c, s = math.cos(yaw), math.sin(yaw)
        for primitive in scene['meshes'][0]['primitives']:
            positions = values(scene, data, primitive['attributes']['POSITION'])
            normals = values(scene, data, primitive['attributes']['NORMAL'])
            uvs = values(scene, data, primitive['attributes']['TEXCOORD_0'])
            for (index,) in values(scene, data, primitive['indices']):
                x, y, z = positions[index]
                nx, ny, nz = normals[index]
                p = [origin[0] + scale * (c*x+s*z), origin[1] + scale*y,
                     origin[2] + scale * (-s*x+c*z)]
                n = [c*nx+s*nz, ny, -s*nx+c*nz]
                result.extend(struct.pack('<12f', *p, 1, *n, 0, *uvs[index], 0, 1))
    return b'FBPROP01' + struct.pack('<I', len(result)//48) + result


if __name__ == '__main__':
    folder, output = map(Path, sys.argv[1:3])
    placements = [([12.1, -.82, 13.3], 2.0, math.pi)]
    placements += [([x, 1.01, z+.015], 1.3, 0)
                   for x, z in [(-10, 4), (-5.5, 9), (-1, 14), (3.5, 19)]]
    placements += [([-1.25, -.42, 23], 1.35, math.pi)]
    output.write_bytes(bake(folder, placements))
    print(f'Baked {len(placements)} textured CRTs')
