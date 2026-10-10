from pathlib import Path
import bpy
import math
import struct
import sys
from mathutils import Vector


root, output = map(Path, sys.argv[sys.argv.index('--')+1:])
placements = [
    ('sofa_02', (12.1, -1.5, 8.7), 0),
    ('modern_wooden_cabinet', (12.1, -1.5, 13.3), math.pi),
    ('modern_coffee_table_01', (12.1, -1.5, 10.5), 0),
    ('wooden_bookshelf_worn', (14.7, -1.5, 12.0), -math.pi/2),
    ('sofa_02', (9.8, -1.5, 10.5), math.pi/2),
    ('wooden_bookshelf_worn', (14.7, -1.5, 16.0), -math.pi/2),
    ('wooden_bookshelf_worn', (12.3, -1.5, 17.5), math.pi),
]
result = bytearray()
for tile, (name, origin, yaw) in enumerate(placements):
    tile={'sofa_02':0,'modern_wooden_cabinet':1,'modern_coffee_table_01':2,'wooden_bookshelf_worn':3}[name]
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=str(root / name / f'{name}_1k.gltf'))
    objects = [o for o in bpy.context.scene.objects if o.type == 'MESH']
    world = [(o.matrix_world @ v.co) for o in objects for v in o.data.vertices]
    center = [(min(p[i] for p in world)+max(p[i] for p in world))/2 for i in [0, 1]]
    floor = min(p.z for p in world)
    c, s = math.cos(yaw), math.sin(yaw)
    for obj in objects:
        mesh = obj.data
        mesh.calc_loop_triangles()
        transform = obj.matrix_world
        normal_transform = transform.to_3x3().inverted().transposed()
        uv = mesh.uv_layers.active.data
        for triangle in mesh.loop_triangles:
            for loop_index in triangle.loops:
                loop = mesh.loops[loop_index]
                p = transform @ mesh.vertices[loop.vertex_index].co
                n = (normal_transform @ loop.normal).normalized()
                x, y, z = p.x-center[0], p.z-floor, -(p.y-center[1])
                nx, ny, nz = n.x, n.z, -n.y
                position = [origin[0]+c*x+s*z, origin[1]+y, origin[2]-s*x+c*z]
                normal = [c*nx+s*nz, ny, -s*nx+c*nz]
                u, v = uv[loop_index].uv
                atlas_uv = [(tile % 2+u)/2, (tile//2+1-v)/2]
                result.extend(struct.pack('<12f', *position, 1, *normal, 0, *atlas_uv, 0, 1))
output.write_bytes(b'FBPROP01'+struct.pack('<I', len(result)//48)+result)
print(f'Baked lounge: {len(result)//144} triangles')
