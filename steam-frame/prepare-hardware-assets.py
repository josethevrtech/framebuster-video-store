from pathlib import Path
import bpy
import importlib.util
import math
import json
import struct
import sys


spec=importlib.util.spec_from_file_location('hardware_materials',Path(__file__).with_name('hardware-materials.py'))
helper=importlib.util.module_from_spec(spec)
spec.loader.exec_module(helper)
root,output=map(Path,sys.argv[sys.argv.index('--')+1:])
origin=json.loads(Path(__file__).with_name('room-layout.json').read_text())['console']
systems=[('nes',.26),('snes',.23),('genesis',.27),('psx',.27),('n64',.26)]
data=bytearray()
counts=[]
for tile,(name,width) in enumerate(systems):
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=str(root/name/'scene.gltf'))
    objects=[o for o in bpy.context.scene.objects if o.type=='MESH']
    if name=='n64':
        objects=[o for o in objects if any(m.name in ['console_good_main_mat','console_good_legs_mat',
            'console_good_connecter_mat','on_off_mat'] for m in o.data.materials)]
    obj=helper.join_meshes(objects)
    folder=root/name/'baked'
    folder.mkdir(exist_ok=True)
    helper.bake_materials(obj,folder)
    mesh=obj.data
    mesh.calc_loop_triangles()
    lo=[min(v.co[a] for v in mesh.vertices) for a in range(3)]
    hi=[max(v.co[a] for v in mesh.vertices) for a in range(3)]
    center=[(lo[a]+hi[a])/2 for a in range(2)]
    scale=width/(hi[0]-lo[0])
    start=len(data)
    for triangle in mesh.loop_triangles:
        for index in triangle.loops:
            loop=mesh.loops[index]
            p=mesh.vertices[loop.vertex_index].co
            n=mesh.corner_normals[index].vector
            u,v=mesh.uv_layers['BakeUV'].data[index].uv
            position=[origin[0]-(p.x-center[0])*scale,origin[1]+(p.z-lo[2])*scale,origin[2]+(p.y-center[1])*scale]
            normal=[-n.x,n.z,n.y]
            uv=[(tile%2+u)/2,(tile//2+1-v)/3]
            data.extend(struct.pack('<12f',*position,1,*normal,0,*uv,0,1))
    counts.append((len(data)-start)//48)
    print(name,'textured console ready',flush=True)
output.write_bytes(b'FBCONS01'+struct.pack('<5I',*counts)+data)
