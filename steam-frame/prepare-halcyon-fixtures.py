from pathlib import Path
import json
import math
import struct
import sys


def load(path):
    data = path.read_bytes()
    assert data[:4] == b'glTF'
    size = struct.unpack_from('<I', data, 12)[0]
    scene = json.loads(data[20:20 + size])
    offset = 20 + size
    length = struct.unpack_from('<I', data, offset)[0]
    return scene, data[offset + 8:offset + 8 + length]


def values(scene, data, index):
    accessor = scene['accessors'][index]
    view = scene['bufferViews'][accessor['bufferView']]
    assert 'sparse' not in accessor
    count = {'SCALAR': 1, 'VEC3': 3}[accessor['type']]
    kind = {5126: 'f', 5125: 'I', 5123: 'H', 5121: 'B'}[accessor['componentType']]
    fmt = '<' + kind * count
    stride = view.get('byteStride', struct.calcsize(fmt))
    offset = view.get('byteOffset', 0) + accessor.get('byteOffset', 0)
    return [struct.unpack_from(fmt, data, offset + i * stride) for i in range(accessor['count'])]


def bake(path, origin, scale, selected=None, black=False):
    scene, data = load(path)
    result = bytearray()
    for node in scene['nodes']:
        if 'mesh' not in node or selected and node.get('name') != selected:
            continue
        assert not any(key in node for key in ['matrix', 'translation', 'rotation', 'scale'])
        for primitive in scene['meshes'][node['mesh']]['primitives']:
            assert primitive.get('mode', 4) == 4
            positions = values(scene, data, primitive['attributes']['POSITION'])
            normals = values(scene, data, primitive['attributes']['NORMAL'])
            indices = values(scene, data, primitive['indices'])
            material = scene['materials'][primitive['material']]
            color = material.get('pbrMetallicRoughness', {}).get('baseColorFactor', [.5, .5, .5, 1])
            if black:
                color = [.045, .045, .055, 1]
            for (index,) in indices:
                p = positions[index]
                n = normals[index]
                if selected == 'Deck':
                    p = [p[2] * scale[0], p[1] * scale[1], p[0] * scale[2]]
                    n = [n[2] / scale[0], n[1] / scale[1], n[0] / scale[2]]
                else:
                    p = [p[i] * scale[i] for i in range(3)]
                    n = [n[i] / scale[i] for i in range(3)]
                length = math.sqrt(sum(x*x for x in n))
                result.extend(struct.pack('<12f', *[p[i] + origin[i] for i in range(3)], 1,
                    *[x / length for x in n], 0, *color))
    return result


folder, output = map(Path, sys.argv[1:3])
result = bake(folder/'checkout-counter-shield-rounded.glb', [0, -1.5, 15.3], [.3048]*3)
for x in [-5.2, 5.2]:
    for z in [1.0, 5.5]:
        for y in [-1.15, -.65, -.15, .25]:
            result.extend(bake(folder/'shelf-components.glb', [x, y, z], [1.15, .56, 2.65], 'Deck', True))
assert len(result) < 4 * 1024 * 1024
output.write_bytes(b'FBPROP01' + struct.pack('<I', len(result)//48) + result)
print(f'Baked {len(result)//144} upstream fixture triangles')
