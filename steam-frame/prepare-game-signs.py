from pathlib import Path
from PIL import Image,ImageDraw,ImageFont
import sys
import json


output,font_path=map(Path,sys.argv[1:3])
if '--music' in sys.argv:
    titles=['MUSIC']+json.loads(Path(__file__).with_name('music-genres.json').read_text())+['NOW PLAYING','PREVIOUS','PAUSE / PLAY','NEXT']
else:
    systems=json.loads(Path(__file__).with_name('game-systems.json').read_text())
    titles=['VIDEO GAMES']+[system['label'] for system in systems]
image=Image.new('RGBA',(1024,len(titles)*128),(13,13,18,255))
draw=ImageDraw.Draw(image)
font=ImageFont.truetype(str(font_path),54)
for row,title in enumerate(titles):
    top=row*128
    draw.rounded_rectangle((3,top+3,1020,top+124),12,fill='#c6a25c')
    draw.rounded_rectangle((9,top+9,1014,top+118),7,fill='#55283f')
    draw.text((512,top+64),title,font=font,fill='#eee5cc',anchor='mm')
image.save(output.with_suffix('.png'))
output.write_bytes(image.transpose(Image.Transpose.FLIP_TOP_BOTTOM).tobytes())
