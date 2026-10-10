from pathlib import Path
import bpy
import struct
import sys
from mathutils import Vector

assets,output=(Path(p).resolve() for p in sys.argv[sys.argv.index('--')+1:])
output.mkdir(parents=True,exist_ok=True)
bpy.ops.wm.read_factory_settings(use_empty=True)
for name,width,height in [('retail',6144,2048),('lounge',3072,1024),('lounge-tv',3072,1024)]:
    mesh_name={'retail':'retail-models','lounge':'lounge-models','lounge-tv':'lounge-tv-model'}[name]
    data=(assets/(mesh_name+'.bin')).read_bytes()
    rows=[struct.unpack_from('<12f',data,i) for i in range(12,len(data),48)]
    geometry=bpy.data.meshes.new(name)
    geometry.from_pydata([(r[0],-r[2],r[1]) for r in rows],[],[(i,i+1,i+2) for i in range(0,len(rows),3)])
    obj=bpy.data.objects.new(name,geometry)
    bpy.context.collection.objects.link(obj)
    uv=geometry.uv_layers.new()
    for i,r in enumerate(rows): uv.data[i].uv=(r[8],1-r[9])
    for p in geometry.polygons: p.use_smooth=True
    geometry.normals_split_custom_set([(r[4],-r[6],r[5]) for r in rows])
    material=bpy.data.materials.new(name)
    material.use_nodes=True
    tree=material.node_tree
    bsdf=tree.nodes['Principled BSDF']
    pixels=(assets/(name+'-materials.rgba')).read_bytes()
    image=bpy.data.images.new(name,width=width,height=height,alpha=True)
    image.colorspace_settings.name='Non-Color'
    image.pixels.foreach_set([p/255 for p in pixels])
    for column,kind in enumerate(['diff','arm','normal']):
        tex=tree.nodes.new('ShaderNodeTexImage')
        tex.image=image
        coords=tree.nodes.new('ShaderNodeTexCoord')
        mapping=tree.nodes.new('ShaderNodeVectorMath')
        mapping.operation='MULTIPLY_ADD'
        mapping.inputs[1].default_value=(1/3,1,1)
        mapping.inputs[2].default_value=(column/3,0,0)
        tree.links.new(coords.outputs['UV'],mapping.inputs[0])
        tree.links.new(mapping.outputs['Vector'],tex.inputs['Vector'])
        if kind=='diff':
            gamma=tree.nodes.new('ShaderNodeGamma')
            gamma.inputs['Gamma'].default_value=2.2
            tree.links.new(tex.outputs['Color'],gamma.inputs['Color'])
            tree.links.new(gamma.outputs['Color'],bsdf.inputs['Base Color'])
        elif kind=='normal':
            normal=tree.nodes.new('ShaderNodeNormalMap')
            tree.links.new(tex.outputs['Color'],normal.inputs['Color'])
            tree.links.new(normal.outputs['Normal'],bsdf.inputs['Normal'])
        else:
            split=tree.nodes.new('ShaderNodeSeparateColor')
            tree.links.new(tex.outputs['Color'],split.inputs['Color'])
            tree.links.new(split.outputs['Green'],bsdf.inputs['Roughness'])
            tree.links.new(split.outputs['Blue'],bsdf.inputs['Metallic'])
    geometry.materials.append(material)
scene=bpy.context.scene
scene.world=bpy.data.worlds.new('Preview')
scene.world.use_nodes=True
scene.world.node_tree.nodes['Background'].inputs[0].default_value=(.15,.16,.18,1)
scene.render.engine='CYCLES'
scene.cycles.samples=16
scene.render.resolution_x=1400
scene.render.resolution_y=900
scene.render.resolution_percentage=100
bpy.ops.mesh.primitive_plane_add(size=60,location=(0,-10,-1.5))
floor=bpy.context.object
material=bpy.data.materials.new('Neutral floor')
material.diffuse_color=(.04,.035,.07,1)
floor.data.materials.append(material)
for location in [(-9,-21,3),(0,-18,3)]:
    bpy.ops.object.light_add(type='AREA',location=location)
    light=bpy.context.object
    light.data.energy=800
    light.data.size=5
for name,eye,target in [('lounge',(-5,-17,.3),(-10.6,-22,-.45)),('checkout',(4,-23,.15),(0,-19.5,-.65))]:
    bpy.ops.object.camera_add(location=eye)
    camera=bpy.context.object
    camera.rotation_euler=(Vector(target)-camera.location).to_track_quat('-Z','Y').to_euler()
    camera.data.lens=30
    scene.camera=camera
    scene.render.filepath=str(output/(name+'.png'))
    bpy.ops.render.render(write_still=True)
