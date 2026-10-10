from pathlib import Path
from PIL import Image
import json
import sys


root, output = map(Path, sys.argv[1:])
names = ['sofa_02', 'modern_wooden_cabinet', 'modern_coffee_table_01', 'wooden_bookshelf_worn']
atlas = Image.new('RGBA', (3072, 1024))
for tile, name in enumerate(names):
    scene = json.loads((root/name/f'{name}_1k.gltf').read_text())
    material = scene['materials'][0]
    maps = [material['pbrMetallicRoughness']['baseColorTexture']['index'],
            material['pbrMetallicRoughness']['metallicRoughnessTexture']['index'],
            material['normalTexture']['index']]
    for column, texture in enumerate(maps):
        image = scene['images'][scene['textures'][texture]['source']]['uri']
        pixels = Image.open(root/name/image).convert('RGBA').resize((512, 512), Image.Resampling.LANCZOS)
        atlas.paste(pixels, (column*1024+(tile%2)*512, (tile//2)*512))
output.write_bytes(atlas.transpose(Image.Transpose.FLIP_TOP_BOTTOM).tobytes())
print('Baked photographic lounge material atlas')
