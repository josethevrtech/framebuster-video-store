from pathlib import Path
import bpy
import importlib.util
import math
import struct
import sys
from mathutils import Matrix, Vector


spec=importlib.util.spec_from_file_location('hardware_materials',Path(__file__).with_name('hardware-materials.py'))
helper=importlib.util.module_from_spec(spec)
spec.loader.exec_module(helper)
root,output=map(Path,sys.argv[sys.argv.index('--')+1:])
families=[('nes-cartridge',.133),('snes-cartridge',.156),('genesis-cartridge',.100),
    ('psx-case',.142),('n64-cartridge',.115),('gb-cartridge',.057)]
payload=bytearray()
counts=[]
dimensions=[]
for tile,(name,width) in enumerate(families):
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=str(root/name/'scene.gltf'))
    objects=[o for o in bpy.context.scene.objects if o.type=='MESH']
    if name=='genesis-cartridge':
        materials={'Bottom_ncl1_3','Top_ncl1_1','FL_Lable_ncl1_1','screws_and_plata_ncl2__ncl1_2'}
        objects=[o for o in objects if any(m.name in materials for m in o.data.materials)]
    if name=='psx-case':
        objects=[o for o in objects if o.name not in ['Object_13','Object_14']]
        for obj in objects:
            world=obj.matrix_world.copy()
            points=[world@v.co for v in obj.data.vertices]
            if obj.name in ['Object_10','Object_11']:
                center=sum(points,Vector())/len(points)
                xx=sum((p.x-center.x)**2 for p in points)
                yy=sum((p.y-center.y)**2 for p in points)
                xy=sum((p.x-center.x)*(p.y-center.y) for p in points)
                angle=math.pi/2-.5*math.atan2(2*xy,xx-yy)
                rotation=Matrix.Rotation(angle,3,'Z')
                points=[rotation@(p-center) for p in points]
                lo=min(p.y for p in points)
                front=max(p.x for p in points)
                for p in points:
                    p.x-=front+.16
                    p.y+=-1.54-lo
                    p.z+=center.z
            for vertex,p in zip(obj.data.vertices,points): vertex.co=p
            obj.matrix_world=Matrix.Identity(4)
    if name in ['genesis-cartridge','psx-case']:
        rotation=Matrix(((0,0,1,0),(1,0,0,0),(0,1,0,0),(0,0,0,1))) if name=='genesis-cartridge' else Matrix.Rotation(math.pi/2,4,'Z')
        for obj in objects: obj.matrix_world=rotation@obj.matrix_world
    if name=='n64-cartridge':
        for obj in objects:
            obj.parent=None
            obj.matrix_world=Matrix.Identity(4)
    for other in bpy.context.scene.objects:
        if other.type=='MESH' and other not in objects: other.hide_render=True
    obj=helper.join_meshes(objects)
    if len(obj.data.polygons)>6000:
        modifier=obj.modifiers.new('Rental display topology','DECIMATE')
        modifier.ratio=4500/len(obj.data.polygons)
        bpy.ops.object.modifier_apply(modifier=modifier.name)
    folder=root/name/'baked'
    folder.mkdir(exist_ok=True)
    helper.bake_materials(obj,folder)
    mesh=obj.data
    mesh.calc_loop_triangles()
    lo=[min(v.co[a] for v in mesh.vertices) for a in range(3)]
    hi=[max(v.co[a] for v in mesh.vertices) for a in range(3)]
    scale=width/(hi[0]-lo[0])
    center=[(lo[a]+hi[a])/2 for a in range(2)]
    start=len(payload)
    for triangle in mesh.loop_triangles:
        for index in triangle.loops:
            loop=mesh.loops[index]
            p=mesh.vertices[loop.vertex_index].co
            n=mesh.corner_normals[index].vector
            u,v=mesh.uv_layers['BakeUV'].data[index].uv
            position=[(p.x-center[0])*scale,(p.z-lo[2])*scale,(p.y-center[1])*scale]
            normal=[n.x,n.z,n.y]
            if name=='snes-cartridge':
                position[0]*=-1
                position[2]*=-1
                normal[0]*=-1
                normal[2]*=-1
            uv=[(tile%2+u)/2,(tile//2+1-v)/3]
            payload.extend(struct.pack('<12f',*position,1,*normal,0,*uv,0,1))
    counts.append((len(payload)-start)//48)
    dimensions.append([width,(hi[2]-lo[2])*scale,(hi[1]-lo[1])*scale])
    print(name,dimensions[-1],flush=True)
output.write_bytes(b'FBCART01'+struct.pack('<6I',*counts)+struct.pack('<18f',*[v for d in dimensions for v in d])+payload)
