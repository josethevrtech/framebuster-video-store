import bpy
import math


def join_meshes(objects):
    bpy.ops.object.select_all(action='DESELECT')
    for obj in objects:
        world=obj.matrix_world.copy()
        obj.parent=None
        obj.matrix_world=world
        obj.select_set(True)
        if not obj.data.uv_layers:
            obj.data.uv_layers.new(name='SourceUV')
        obj.data.uv_layers.active.name='SourceUV'
        for material in obj.data.materials:
            tree=material.node_tree
            if not tree:
                continue
            source=tree.nodes.new('ShaderNodeUVMap')
            source.uv_map='SourceUV'
            for node in list(tree.nodes):
                if node.type=='UVMAP':
                    node.uv_map='SourceUV'
                if node.type=='TEX_IMAGE' and not node.inputs['Vector'].is_linked:
                    tree.links.new(source.outputs['UV'],node.inputs['Vector'])
    bpy.context.view_layer.objects.active=objects[0]
    bpy.ops.object.join()
    obj=bpy.context.object
    bpy.ops.object.transform_apply(location=True,rotation=True,scale=True)
    obj.data.uv_layers.new(name='BakeUV')
    obj.data.uv_layers.active=obj.data.uv_layers['BakeUV']
    obj.data.uv_layers['BakeUV'].active_render=True
    bpy.ops.object.mode_set(mode='EDIT')
    bpy.ops.mesh.select_all(action='SELECT')
    bpy.ops.uv.smart_project(angle_limit=math.radians(66),island_margin=.012)
    bpy.ops.object.mode_set(mode='OBJECT')
    return obj


def bake_materials(obj,folder,size=512):
    scene=bpy.context.scene
    scene.render.engine='CYCLES'
    scene.cycles.samples=1
    scene.render.bake.margin=4
    materials=[]
    for material in obj.data.materials:
        material.use_nodes=True
        tree=material.node_tree
        bsdf=next(n for n in tree.nodes if n.type=='BSDF_PRINCIPLED')
        target=tree.nodes.new('ShaderNodeTexImage')
        target.label='FrameBuster bake target'
        tree.nodes.active=target
        output=next(n for n in tree.nodes if n.type=='OUTPUT_MATERIAL')
        emission=tree.nodes.new('ShaderNodeEmission')
        materials.append((tree,bsdf,target,output,emission))
    for kind in ['diff','arm','normal']:
        image=bpy.data.images.new('Hardware_'+kind,width=size,height=size)
        if kind!='diff': image.colorspace_settings.name='Non-Color'
        for tree,bsdf,target,output,emission in materials:
            target.image=image
            tree.nodes.active=target
            if kind=='normal':
                tree.links.new(bsdf.outputs['BSDF'],output.inputs['Surface'])
                continue
            color=bsdf.inputs['Base Color']
            if kind=='arm':
                combine=tree.nodes.new('ShaderNodeCombineColor')
                combine.mode='RGB'
                combine.inputs['Red'].default_value=1
                for name,channel in [('Roughness','Green'),('Metallic','Blue')]:
                    value=bsdf.inputs[name]
                    if value.is_linked:
                        tree.links.new(value.links[0].from_socket,combine.inputs[channel])
                    else:
                        combine.inputs[channel].default_value=value.default_value
                tree.links.new(combine.outputs[0],emission.inputs['Color'])
            elif color.is_linked:
                tree.links.new(color.links[0].from_socket,emission.inputs['Color'])
            else:
                emission.inputs['Color'].default_value=color.default_value
            tree.links.new(emission.outputs[0],output.inputs['Surface'])
        bpy.ops.object.bake(type='NORMAL' if kind=='normal' else 'EMIT')
        image.filepath_raw=str((folder/f'{kind}.png').resolve())
        image.file_format='PNG'
        image.save()
