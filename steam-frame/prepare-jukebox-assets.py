from pathlib import Path
import bpy
import math
import struct
import sys


source, output = map(Path, sys.argv[sys.argv.index('--')+1:])
bpy.ops.wm.open_mainfile(filepath=str(source))
scene = bpy.context.scene
scene.render.engine = 'CYCLES'
scene.cycles.samples = 1
scene.render.bake.margin = 8
objects = [o for o in scene.objects if o.type == 'MESH' and not o.name.startswith('Light Plane')]
bpy.ops.object.select_all(action='DESELECT')
for obj in objects:
    obj.hide_set(False)
    obj.hide_viewport = False
    obj.select_set(True)
    bpy.context.view_layer.objects.active = obj
    for modifier in list(obj.modifiers):
        bpy.ops.object.modifier_apply(modifier=modifier.name)
bpy.context.view_layer.objects.active = objects[0]
bpy.ops.object.join()
obj = bpy.context.object
bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
bpy.ops.object.mode_set(mode='EDIT')
bpy.ops.mesh.reveal(select=True)
bpy.ops.mesh.select_all(action='SELECT')
bpy.ops.uv.smart_project(angle_limit=math.radians(66), island_margin=.015)
bpy.ops.object.mode_set(mode='OBJECT')
obj.data.uv_layers.active.active_render = True
print("Material coverage",[(m.name,sum(p.area for p in obj.data.polygons if p.material_index==i)) for i,m in enumerate(obj.data.materials)])
colors = {}
for material in obj.data.materials:
    name = material.name.lower()
    metal = any(word in name for word in ['chrome', 'steel', 'gold'])
    rough = .24 if metal else .58
    color = (.65,.68,.72,1) if metal else (.025,.03,.035,1)
    if 'gold' in name: color = (.8,.44,.065,1)
    elif 'red' in name: color = (.42,.018,.015,1)
    elif 'white' in name: color = (.74,.78,.84,1)
    elif 'wood' in name: color = (.28,.11,.035,1)
    material.use_nodes = True
    tree = material.node_tree
    emission = tree.nodes.new('ShaderNodeEmission')
    emission.inputs['Color'].default_value = color
    if 'wood' in name:
        texture = tree.nodes.new('ShaderNodeTexImage')
        texture.image = bpy.data.images.load(str(output / 'dark_wood_diff_1k.jpg'))
        texture.projection = 'BOX'
        texture.projection_blend = .2
        coordinates = tree.nodes.new('ShaderNodeTexCoord')
        mapping = tree.nodes.new('ShaderNodeMapping')
        mapping.inputs['Scale'].default_value = (2,2,2)
        tree.links.new(coordinates.outputs['Generated'], mapping.inputs['Vector'])
        tree.links.new(mapping.outputs['Vector'], texture.inputs['Vector'])
        tree.links.new(texture.outputs['Color'], emission.inputs['Color'])
    target = tree.nodes.new('ShaderNodeTexImage')
    tree.nodes.active = target
    result = tree.nodes.new('ShaderNodeOutputMaterial')
    result.target = 'CYCLES'
    result.is_active_output = True
    tree.links.new(emission.outputs[0], result.inputs['Surface'])
    colors[material.name] = (emission, target, float(metal), rough)
output.mkdir(parents=True, exist_ok=True)
for kind in ['diff', 'arm', 'normal']:
    image = bpy.data.images.new('Jukebox_'+kind, width=1024, height=1024)
    if kind != 'diff': image.colorspace_settings.name = 'Non-Color'
    for material in obj.data.materials:
        emission, target, metal, rough = colors[material.name]
        target.image = image
        if kind == 'arm':
            for link in list(emission.inputs['Color'].links): material.node_tree.links.remove(link)
            emission.inputs['Color'].default_value = (1,rough,metal,1)
        elif kind == 'normal':
            emission.inputs['Color'].default_value = (.5,.5,1,1)
    bpy.ops.object.bake(type='EMIT')
    image.filepath_raw = str(output / f'jukebox_{kind}.png')
    image.file_format = 'PNG'
    image.save()
mesh = obj.data
mesh.calc_loop_triangles()
data = bytearray()
for triangle in mesh.loop_triangles:
    if 'glass' in mesh.materials[triangle.material_index].name.lower(): continue
    for loop_index in triangle.loops:
        loop = mesh.loops[loop_index]
        position = mesh.vertices[loop.vertex_index].co
        normal = mesh.corner_normals[loop_index].vector
        uv = mesh.uv_layers.active.data[loop_index].uv
        p = [position.x*.74, -1.5+(position.z+.169)*.74, (position.y-.98)*.74]
        n = [normal.x,normal.z,normal.y]
        data.extend(struct.pack('<12f',*p,1,*n,0,uv.x,1-uv.y,0,1))
(output / 'jukebox-model.bin').write_bytes(b'FBPROP01'+struct.pack('<I',len(data)//48)+data)
print(f'Baked licensed jukebox: {len(data)//144} triangles')
