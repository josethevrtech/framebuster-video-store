from pathlib import Path
import bpy
import json
import struct
import sys
from mathutils import Vector

root,mesh=(Path(p).resolve() for p in sys.argv[sys.argv.index('--')+1:])
inspection=json.loads((root/'inspection.json').read_text())
data=mesh.read_bytes()
rows=[struct.unpack_from('<12f',data,i) for i in range(12,len(data),48)]
bpy.ops.wm.read_factory_settings(use_empty=True)
offset=0
for name,tile in [('counter',0),('registers',1),('music-cabinet',3),('cd-stereo',2)]:
    count=inspection['models'][name]['triangles']*3
    vertices=rows[offset:offset+count]
    offset+=count
    geometry=bpy.data.meshes.new(name)
    geometry.from_pydata([(r[0],-r[2],r[1]) for r in vertices],[],[(i,i+1,i+2) for i in range(0,count,3)])
    obj=bpy.data.objects.new(name,geometry)
    bpy.context.collection.objects.link(obj)
    uv=geometry.uv_layers.new()
    for i,r in enumerate(vertices): uv.data[i].uv=(r[8]*2-tile%2,1-(r[9]*2-tile//2))
    for p in geometry.polygons: p.use_smooth=True
    geometry.normals_split_custom_set([(r[4],-r[6],r[5]) for r in vertices])
    material=bpy.data.materials.new(name)
    material.use_nodes=True
    tree=material.node_tree
    bsdf=tree.nodes['Principled BSDF']
    for kind in ['diff','arm','normal']:
        tex=tree.nodes.new('ShaderNodeTexImage')
        tex.image=bpy.data.images.load(str(root/name/(kind+'.png')))
        if kind!='diff': tex.image.colorspace_settings.name='Non-Color'
        if kind=='diff': tree.links.new(tex.outputs['Color'],bsdf.inputs['Base Color'])
        elif kind=='normal':
            normal=tree.nodes.new('ShaderNodeNormalMap')
            tree.links.new(tex.outputs['Color'],normal.inputs['Color'])
            tree.links.new(normal.outputs['Normal'],bsdf.inputs['Normal'])
        else:
            separate=tree.nodes.new('ShaderNodeSeparateColor')
            tree.links.new(tex.outputs['Color'],separate.inputs['Color'])
            tree.links.new(separate.outputs['Green'],bsdf.inputs['Roughness'])
            tree.links.new(separate.outputs['Blue'],bsdf.inputs['Metallic'])
    geometry.materials.append(material)
scene=bpy.context.scene
scene.world=bpy.data.worlds.new('Preview')
scene.world.use_nodes=True
scene.world.node_tree.nodes['Background'].inputs[0].default_value=(.09,.09,.11,1)
scene.render.engine='CYCLES'
scene.cycles.samples=32
scene.render.resolution_x=1200
scene.render.resolution_y=800
scene.render.resolution_percentage=100
for label,eye,target in [('checkout',(0,-17,1.0),(0,-21,-.5)),('cd-console',(-10.9,-22.9,-.05),(-11.7,-24,-.69))]:
    bpy.ops.object.camera_add(location=eye)
    camera=bpy.context.object
    camera.rotation_euler=(Vector(target)-camera.location).to_track_quat('-Z','Y').to_euler()
    camera.data.lens=45
    scene.camera=camera
    lights=[]
    for location,energy,size in [(Vector(target)+Vector((0,1,3)),500,4),(Vector(target)+Vector((-2,1,1)),100,2)]:
        bpy.ops.object.light_add(type='AREA',location=location)
        light=bpy.context.object
        light.data.energy=energy
        light.data.size=size
        light.rotation_euler=(Vector(target)-light.location).to_track_quat('-Z','Y').to_euler()
        lights.append(light)
    scene.render.filepath=str(root/(label+'-preview.png'))
    bpy.ops.render.render(write_still=True)
    for light in lights: light.hide_render=True
