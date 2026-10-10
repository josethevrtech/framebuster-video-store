from pathlib import Path
from PIL import Image,ImageDraw,ImageFont
import json
import sys


root,font_path=map(Path,sys.argv[1:])
items=json.loads((root/'catalog.json').read_text())
font=ImageFont.truetype(str(font_path),19)
small=ImageFont.truetype(str(font_path),11)
count=0
for item in items:
    target=root/item['art']
    if target.exists():
        if target.stat().st_size==192*288*4: continue
        raise ValueError(f'Preserve invalid existing artwork: {target}')
    image=Image.new('RGBA',(192,288),(25,30,37,255))
    draw=ImageDraw.Draw(image)
    draw.rectangle((5,5,186,282),outline='#c5a364',width=2)
    draw.rectangle((7,7,184,48),fill='#583149')
    draw.text((96,19),'FRAMEBUSTER',font=small,fill='#eee3cb',anchor='mm')
    draw.text((96,34),'VIDEO GAME RENTAL',font=small,fill='#d7b16c',anchor='mm')
    title=item['title']
    lines=[]
    line=''
    for character in title:
        if font.getlength(line+character)>165:
            lines.append(line.strip())
            line=''
        line+=character
    if line: lines.append(line.strip())
    y=83
    for line in lines[:7]:
        draw.text((96,y),line,font=font,fill='#eee3cb',anchor='mm')
        y+=24
    draw.text((96,249),item['system'].upper(),font=small,fill='#d7b16c',anchor='mm')
    draw.text((96,265),str(item['year']),font=small,fill='#d7b16c',anchor='mm')
    target.parent.mkdir(exist_ok=True)
    target.write_bytes(image.transpose(Image.Transpose.FLIP_TOP_BOTTOM).tobytes())
    image.save(target.with_name(target.stem+'-rental-sleeve.png'))
    target.with_name(target.stem+'.rental-source.txt').write_text('Original FrameBuster rental cover design. Title and release year from private imported library. No publisher artwork used.\n')
    count+=1
print(f'Prepared {count} original rental sleeves for games without an unambiguous box-art match')
