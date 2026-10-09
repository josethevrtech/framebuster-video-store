from pathlib import Path
from PIL import Image, ImageDraw, ImageFont
import sys

folder = Path(sys.argv[1])
font = ImageFont.truetype(sys.argv[2], 62)
image = Image.new('RGBA', (960, 240))
draw = ImageDraw.Draw(image)
draw.rounded_rectangle((4, 4, 955, 235), 32, fill='#d8b76c')
draw.rounded_rectangle((13, 13, 946, 226), 24, fill='#692542')
draw.text((480, 120), 'MOVIES & SERIES', font=font, fill='#fff8e5', anchor='mm')
image.save(folder/'movies-series-sign.png')
pixels = image.tobytes()
(folder/'movies-series-sign.rgba').write_bytes(b''.join(pixels[y*960*4:(y+1)*960*4] for y in reversed(range(240))))
