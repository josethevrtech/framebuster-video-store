from pathlib import Path
import bpy
import importlib.util
import json
import math
import struct
import sys
from mathutils import Matrix, Vector

repo=Path(__file__).resolve().parent.parent
room=json.loads((repo/'steam-frame/room-layout.json').read_text())
source,output=map(Path,sys.argv[sys.argv.index('--')+1:])
spec=importlib.util.spec_from_file_location('materials',Path(__file__).with_name('hardware-materials.py'))
helper=importlib.util.module_from_spec(spec)
spec.loader.exec_module(helper)
output.mkdir(parents=True,exist_ok=True)
vertices=bytearray()
report={}

def import_model(path):
    before=set(bpy.context.scene.objects)
    bpy.ops.import_scene.gltf(filepath=str(path))
    return [o for o in bpy.context.scene.objects if o not in before and o.type=='MESH']

def place(objects,origin,scale=1,yaw=0):
    rotation=Matrix.Rotation(yaw,4,'Z')
    matrix=Matrix.Translation(Vector((origin[0],-origin[2],origin[1])))@rotation@Matrix.Scale(scale,4)
    for obj in objects:
        world=obj.matrix_world.copy()
        obj.parent=None
        obj.matrix_world=matrix@world

def bake(objects,name,tile):
    for source_obj in objects:
        for material in source_obj.data.materials:
            for node in material.node_tree.nodes:
                if node.type=='NORMAL_MAP': node.uv_map='SourceUV'
    obj=helper.join_meshes(objects)
    folder=output/name
    folder.mkdir(exist_ok=True)
    helper.bake_materials(obj,folder,1024)
    mesh=obj.data
    mesh.calc_loop_triangles()
    lo=[min(v.co[i] for v in mesh.vertices) for i in range(3)]
    hi=[max(v.co[i] for v in mesh.vertices) for i in range(3)]
    report[name]={'triangles':len(mesh.loop_triangles),'bounds':[[lo[0],lo[2],-hi[1]],[hi[0],hi[2],-lo[1]]]}
    for tri in mesh.loop_triangles:
        for index in tri.loops:
            loop=mesh.loops[index]
            p=mesh.vertices[loop.vertex_index].co
            n=mesh.corner_normals[index].vector
            u,v=mesh.uv_layers['BakeUV'].data[index].uv
            uv=[(tile%2+u)/2,(tile//2+1-v)/2]
            vertices.extend(struct.pack('<12f',p.x,p.z,-p.y,1,n.x,n.z,-n.y,0,*uv,0,1))
    bpy.ops.wm.save_as_mainfile(filepath=str(folder/(name+'.blend')))
    print(name,report[name],flush=True)

bpy.ops.wm.read_factory_settings(use_empty=True)
counter=import_model(repo/'public/models/checkout-counter-shield-rounded.glb')
for obj in counter:
    for mat in obj.data.materials:
        if mat.name in ['CounterPlinth','CounterReveal']: continue
        tree=mat.node_tree
        bsdf=next(n for n in tree.nodes if n.type=='BSDF_PRINCIPLED')
        tint=bsdf.inputs['Base Color'].default_value[:]
        mapping=tree.nodes.new('ShaderNodeMapping')
        mapping.inputs['Scale'].default_value=(4,4,4)
        coords=tree.nodes.new('ShaderNodeUVMap')
        coords.uv_map='SourceUV'
        tree.links.new(coords.outputs['UV'],mapping.inputs['Vector'])
        for kind in ['color','normal','roughness']:
            image=bpy.data.images.load(str(repo/f'public/textures/surfaces/store-shelf/{kind}.png'),check_existing=True)
            if kind!='color': image.colorspace_settings.name='Non-Color'
            tex=tree.nodes.new('ShaderNodeTexImage')
            tex.image=image
            tree.links.new(mapping.outputs[0],tex.inputs['Vector'])
            if kind=='color':
                mix=tree.nodes.new('ShaderNodeMixRGB')
                mix.blend_type='MULTIPLY'
                mix.inputs[0].default_value=1
                mix.inputs[2].default_value=tint
                tree.links.new(tex.outputs['Color'],mix.inputs[1])
                tree.links.new(mix.outputs[0],bsdf.inputs['Base Color'])
            elif kind=='normal':
                normal=tree.nodes.new('ShaderNodeNormalMap')
                normal.inputs['Strength'].default_value=.35
                tree.links.new(tex.outputs['Color'],normal.inputs['Color'])
                tree.links.new(normal.outputs[0],bsdf.inputs['Normal'])
            else: tree.links.new(tex.outputs['Color'],bsdf.inputs['Roughness'])
place(counter,room['checkoutOrigin'],.3048)
bake(counter,'counter',0)

bpy.ops.wm.read_factory_settings(use_empty=True)
stations=[]
apex=-14.9+1.5*math.sqrt(2)
for side in [-1,1]:
    x=side*3.0
    normal=Vector((-side/math.sqrt(2),0,1/math.sqrt(2)))
    origin=room['checkoutOrigin']
    center=Vector((origin[0]+x*.3048,origin[1]+2.82*.3048,origin[2]+(apex+abs(x)+.8*math.sqrt(2))*.3048))
    yaw=math.pi-side*math.pi/4
    terminal=import_model(repo/'public/models/rental-terminal.glb')
    for obj in terminal:
        cable_vertices={index for polygon in obj.data.polygons
            if obj.data.materials[polygon.material_index].name.startswith('CableRubber') for index in polygon.vertices}
        for index in cable_vertices:
            p=obj.data.vertices[index].co
            if p.z<.08: p.y=max(p.y,-.36)
        obj.data.update()
        for mat in obj.data.materials:
            if mat.name.startswith(('CrtTube','CrtGlass')):
                bsdf=next(n for n in mat.node_tree.nodes if n.type=='BSDF_PRINCIPLED')
                bsdf.inputs['Base Color'].default_value=(.018,.065,.043,1)
                bsdf.inputs['Alpha'].default_value=1
                bsdf.inputs['Roughness'].default_value=.22
                bsdf.inputs['Transmission Weight'].default_value=0
    place(terminal,center-normal*.08,.3048,yaw)
    keyboard=import_model(repo/'public/models/rental-keyboard.glb')
    place(keyboard,center+normal*.16,.3048*.65,yaw)
    stations.extend(terminal+keyboard)
bake(stations,'registers',1)

bpy.ops.wm.read_factory_settings(use_empty=True)
cabinet=import_model(repo/'native/matineevr/assets/lounge/modern_wooden_cabinet/modern_wooden_cabinet_1k.gltf')
for o in cabinet: o.scale.y*=1.4
bpy.context.view_layer.update()
points=[o.matrix_world@v.co for o in cabinet for v in o.data.vertices]
floor=min(p.z for p in points)
center=Vector([(min(p[i] for p in points)+max(p[i] for p in points))/2 for i in range(2)]+[floor])
for obj in cabinet: obj.matrix_world=Matrix.Translation(-center)@obj.matrix_world
place(cabinet,room['musicCabinet'],1,math.pi)
top=max((o.matrix_world@v.co).z for o in cabinet for v in o.data.vertices)
bake(cabinet,'music-cabinet',3)

bpy.ops.wm.read_factory_settings(use_empty=True)
stereo=import_model(source/'panasonic/scene.gltf')
for obj in stereo:
    for material in obj.data.materials:
        tree=material.node_tree
        bsdf=next(n for n in tree.nodes if n.type=='BSDF_PRINCIPLED')
        transmission=bsdf.inputs['Transmission Weight']
        emission=bsdf.inputs['Emission Color']
        base=bsdf.inputs['Base Color']
        if transmission.is_linked and emission.is_linked and base.is_linked:
            mix=tree.nodes.new('ShaderNodeMixRGB')
            tree.links.new(transmission.links[0].from_socket,mix.inputs[0])
            tree.links.new(base.links[0].from_socket,mix.inputs[1])
            tree.links.new(emission.links[0].from_socket,mix.inputs[2])
            tree.links.new(mix.outputs[0],base)
stereo_x=room['stereoCenterX']
stereo_z=room['musicCabinet'][2]
scale=room['stereoScale']
place(stereo,(stereo_x,top,stereo_z),scale,math.pi)
for p in room['cornerSpeakers']:
    copies=import_model(source/'panasonic/scene.gltf')
    speaker=[o for o in copies if 'speaker-l' in o.name][0]
    world=[speaker.matrix_world@v.co for v in speaker.data.vertices]
    center=Vector([(min(v[i] for v in world)+max(v[i] for v in world))/2 for i in [0,1]]+[min(v.z for v in world)])
    speaker.matrix_world=Matrix.Translation(-center)@speaker.matrix_world
    yaw=math.atan2(6-p[0],10-p[2])
    place([speaker],(p[0],p[1]-.111*1.5,p[2]),1.5,yaw)
    stereo.append(speaker)
bake(stereo,'cd-stereo',2)
layout={'cover':[stereo_x+.84,top+.11,stereo_z-.18],
    'buttons':[[stereo_x+x*scale,top+.065*scale,stereo_z-.111*scale] for x in [-.065,-.030,.005]],'top':top}
(output/'inspection.json').write_text(json.dumps({'models':report,'layout':layout},indent=2)+'\n')
(repo/'native/matineevr/assets/retail-models.bin').write_bytes(b'FBPROP01'+struct.pack('<I',len(vertices)//48)+vertices)
bounds=report['music-cabinet']['bounds']
obstacle=[bounds[0][0],bounds[1][0],bounds[0][2],bounds[1][2]]
layout_source='pub const COVER: [f32;3]='+str(layout['cover']).replace(' ', '')+';\npub const BUTTONS: [[f32;3];3]='+str(layout['buttons']).replace(' ', '')+';\npub const CABINET: [f32;4]='+str(obstacle).replace(' ', '')+';\n'
layout_source+='pub const STEREO_SPEAKERS: [[f32;3];2]='+str([[stereo_x+x*scale,top+.11*scale,stereo_z] for x in [-.18,.18]]).replace(' ', '')+';\n'
(repo/'native/matineevr/src/store_retail_layout.rs').write_text(layout_source)
