from pathlib import Path
import bpy
import importlib.util
import json
import math
import struct
import sys
from mathutils import Matrix,Vector

source,folder=(Path(p).resolve() for p in sys.argv[sys.argv.index('--')+1:])
repo=Path(__file__).resolve().parent.parent
room=json.loads(Path(__file__).with_name('room-layout.json').read_text())
spec=importlib.util.spec_from_file_location('materials',Path(__file__).with_name('hardware-materials.py'))
helper=importlib.util.module_from_spec(spec)
spec.loader.exec_module(helper)
bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.import_scene.gltf(filepath=str(source/'sony-pvm/scene.gltf'))
objects=[o for o in bpy.context.scene.objects if o.type=='MESH' and all(m.name.startswith('pvm_') for m in o.data.materials)]
assert len(objects)>=4
for o in objects:
    for mat in o.data.materials:
        for node in mat.node_tree.nodes:
            if node.type=='NORMAL_MAP': node.uv_map='SourceUV'
        if mat.name.startswith('pvm_screen_and_details'):
            bsdf=next(n for n in mat.node_tree.nodes if n.type=='BSDF_PRINCIPLED')
            color=bsdf.inputs['Base Color']
            if color.is_linked:
                mix=mat.node_tree.nodes.new('ShaderNodeMixRGB')
                mix.blend_type='MULTIPLY'
                mix.inputs[0].default_value=1
                mix.inputs[2].default_value=(.035,.045,.055,1)
                mat.node_tree.links.new(color.links[0].from_socket,mix.inputs[1])
                mat.node_tree.links.new(mix.outputs[0],color)
            bsdf.inputs['Roughness'].default_value=.14
points=[o.matrix_world@v.co for o in objects for v in o.data.vertices]
lo=Vector([min(p[i] for p in points) for i in range(3)])
hi=Vector([max(p[i] for p in points) for i in range(3)])
center=Vector(((lo.x+hi.x)/2,(lo.y+hi.y)/2,lo.z))
scale=room['tvWidth']/(hi.x-lo.x)
origin=room['tv']
matrix=Matrix.Translation(Vector((origin[0],-origin[2],origin[1])))@Matrix.Rotation(math.pi,4,'Z')@Matrix.Scale(scale,4)@Matrix.Translation(-center)
for o in objects:
    world=o.matrix_world.copy()
    o.parent=None
    o.matrix_world=matrix@world
obj=helper.join_meshes(objects)
folder.mkdir(parents=True,exist_ok=True)
helper.bake_materials(obj,folder,1024)
mesh=obj.data
mesh.calc_loop_triangles()
vertices=bytearray()
for triangle in mesh.loop_triangles:
    for index in triangle.loops:
        loop=mesh.loops[index]
        p=mesh.vertices[loop.vertex_index].co
        n=mesh.corner_normals[index].vector
        u,v=mesh.uv_layers['BakeUV'].data[index].uv
        vertices.extend(struct.pack('<12f',p.x,p.z,-p.y,1,n.x,n.z,-n.y,0,u,1-v,0,1))
(repo/'native/matineevr/assets/lounge-tv-model.bin').write_bytes(b'FBPROP01'+struct.pack('<I',len(vertices)//48)+vertices)
bpy.ops.wm.save_as_mainfile(filepath=str(folder/'lounge-tv.blend'))
report={'triangles':len(vertices)//144,'sourceWidth':hi.x-lo.x,'scale':scale,'origin':origin,'width':room['tvWidth']}
(folder/'inspection.json').write_text(json.dumps(report,indent=2)+'\n')
print(report,flush=True)
