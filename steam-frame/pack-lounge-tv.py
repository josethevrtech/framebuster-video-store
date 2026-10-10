from pathlib import Path
from PIL import Image
import sys

folder,output=map(Path,sys.argv[1:])
atlas=Image.new('RGBA',(3072,1024))
for column,kind in enumerate(['diff','arm','normal']):
    image=Image.open(folder/(kind+'.png')).convert('RGBA')
    if kind=='arm' and (folder/'ao.png').exists():
        _,g,b,a=image.split()
        image=Image.merge('RGBA',(Image.open(folder/'ao.png').convert('L'),g,b,a))
    atlas.paste(image,(column*1024,0))
output.write_bytes(atlas.transpose(Image.Transpose.FLIP_TOP_BOTTOM).tobytes())
