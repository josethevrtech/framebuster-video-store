from pathlib import Path
from PIL import Image
import sys


root,output=map(Path,sys.argv[1:3])
names=['nes','snes','genesis','psx','n64'] if len(sys.argv)==3 else [
    'nes-cartridge','snes-cartridge','genesis-cartridge','psx-case','n64-cartridge','gb-cartridge']
atlas=Image.new('RGBA',(3072,1536))
for tile,name in enumerate(names):
    for column,kind in enumerate(['diff','arm','normal']):
        image=Image.open(root/name/'baked'/f'{kind}.png').convert('RGBA')
        atlas.paste(image,(column*1024+(tile%2)*512,(tile//2)*512))
output.write_bytes(atlas.transpose(Image.Transpose.FLIP_TOP_BOTTOM).tobytes())
print('Packed licensed hardware materials')
