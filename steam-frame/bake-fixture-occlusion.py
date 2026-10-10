from pathlib import Path
import bpy
import sys

folders=[Path(p).resolve() for p in sys.argv[sys.argv.index('--')+1:]]
for folder in folders:
    blend=next(folder.glob('*.blend'))
    bpy.ops.wm.open_mainfile(filepath=str(blend))
    obj=bpy.context.view_layer.objects.active
    assert obj.type=='MESH'
    scene=bpy.context.scene
    scene.render.engine='CYCLES'
    scene.cycles.samples=16
    scene.render.bake.margin=4
    image=bpy.data.images.new('FixtureOcclusion',width=1024,height=1024)
    image.colorspace_settings.name='Non-Color'
    for mat in obj.data.materials:
        tree=mat.node_tree
        target=tree.nodes.new('ShaderNodeTexImage')
        target.image=image
        tree.nodes.active=target
        ao=tree.nodes.new('ShaderNodeAmbientOcclusion')
        ao.only_local=True
        ao.inputs['Distance'].default_value=.22
        ao.inputs['Color'].default_value=(1,1,1,1)
        emit=tree.nodes.new('ShaderNodeEmission')
        tree.links.new(ao.outputs['Color'],emit.inputs['Color'])
        output=next(n for n in tree.nodes if n.type=='OUTPUT_MATERIAL')
        tree.links.new(emit.outputs[0],output.inputs['Surface'])
    bpy.ops.object.bake(type='EMIT')
    image.filepath_raw=str(folder/'ao.png')
    image.file_format='PNG'
    image.save()
    print('Baked material occlusion',folder.name,flush=True)
