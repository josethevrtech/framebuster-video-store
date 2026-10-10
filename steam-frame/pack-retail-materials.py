from pathlib import Path
from PIL import Image
import sys

root,output=map(Path,sys.argv[1:])
atlas=Image.new('RGBA',(6144,2048))
for tile,name in enumerate(['counter','registers','cd-stereo','music-cabinet']):
    for column,kind in enumerate(['diff','arm','normal']):
        image=Image.open(root/name/(kind+'.png')).convert('RGBA')
        if kind=='arm' and (root/name/'ao.png').exists():
            _,g,b,a=image.split()
            image=Image.merge('RGBA',(Image.open(root/name/'ao.png').convert('L'),g,b,a))
        atlas.paste(image,(column*2048+(tile%2)*1024,(tile//2)*1024))
output.write_bytes(atlas.transpose(Image.Transpose.FLIP_TOP_BOTTOM).tobytes())
