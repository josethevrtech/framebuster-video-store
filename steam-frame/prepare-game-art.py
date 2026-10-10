import argparse
from concurrent.futures import ThreadPoolExecutor
from hashlib import sha256
from pathlib import Path
from urllib.parse import quote
from urllib.request import Request,urlopen
from PIL import Image
import io
import json
import re


REPOS={
 'nes':'Nintendo_-_Nintendo_Entertainment_System',
 'snes':'Nintendo_-_Super_Nintendo_Entertainment_System',
 'n64':'Nintendo_-_Nintendo_64','gb':'Nintendo_-_Game_Boy','gbc':'Nintendo_-_Game_Boy_Color',
 'genesis':'Sega_-_Mega_Drive_-_Genesis','mastersystem':'Sega_-_Master_System_-_Mark_III',
 'gamegear':'Sega_-_Game_Gear','atari2600':'Atari_-_2600',
 'pcengine':'NEC_-_PC_Engine_-_TurboGrafx_16','psx':'Sony_-_PlayStation',
}
def normalized(value):
    value=re.sub(r'\([^)]*\)|\[[^]]*\]','',value)
    return re.sub(r'[^a-z0-9]','',value.casefold())
def request(url):
    return urlopen(Request(url,headers={'User-Agent':'FrameBuster private library artwork'}),timeout=20)
def catalog(plan):
    result=[]
    for item in plan['files']:
        path=Path(item['destination'])
        system=path.parts[0]
        if system=='psx' and path.suffix!='.m3u': continue
        identity=sha256(item['destination'].encode()).hexdigest()[:32]
        result.append({'id':identity,'system':system,'title':item['title'],'year':item['year'],
            'path':item['destination'],'art':f'artwork/{identity}.rgba','romName':path.stem})
    return sorted(result,key=lambda i:(i['system'],i['title'].casefold()))
def prepare(root,items):
    root.mkdir(parents=True,exist_ok=True)
    (root/'artwork').mkdir(exist_ok=True)
    (root/'catalog.json').write_text(json.dumps(items,indent=2),encoding='utf-8')
    for system,repo in REPOS.items():
        selected=[i for i in items if i['system']==system]
        if not selected: continue
        try:
            with request(f'https://api.github.com/repos/libretro-thumbnails/{repo}/git/trees/master?recursive=1') as data:
                tree=json.load(data)['tree']
        except Exception as error:
            print(system,'art index unavailable:',type(error).__name__,flush=True)
            continue
        names=[i['path'] for i in tree if i['path'].startswith('Named_Boxarts/') and i['path'].endswith('.png')]
        by_name={Path(n).stem.casefold():n for n in names}
        by_title={}
        for name in names: by_title.setdefault(normalized(Path(name).stem),[]).append(name)
        def download(item):
            target=root/item['art']
            if target.exists() and target.stat().st_size==192*288*4: return True
            name=by_name.get(item['romName'].casefold())
            candidates=by_title.get(normalized(item['romName']),[])
            if not name:
                if len(candidates)==1: name=candidates[0]
                elif candidates:
                    us=[n for n in candidates if '(USA)' in n]
                    if len(us)==1: name=us[0]
            if not name: return False
            try:
                with request(f'https://raw.githubusercontent.com/libretro-thumbnails/{repo}/master/{quote(name)}') as data:
                    content=data.read(8*1024*1024+1)
                if len(content)>8*1024*1024: return False
                image=Image.open(io.BytesIO(content)).convert('RGBA')
                image.thumbnail((192,288),Image.Resampling.LANCZOS)
                canvas=Image.new('RGBA',(192,288),(12,12,15,255))
                canvas.paste(image,((192-image.width)//2,(288-image.height)//2),image)
                target.write_bytes(canvas.transpose(Image.Transpose.FLIP_TOP_BOTTOM).tobytes())
                (root/'artwork'/f"{item['id']}.png").write_bytes(content)
                (root/'artwork'/f"{item['id']}.source.txt").write_text(
                    f"{item['title']}\nhttps://github.com/libretro-thumbnails/{repo}/blob/master/{name}\nPrivate library artwork; publisher artwork is not licensed by the app's source license.\n")
                return True
            except Exception:
                return False
        with ThreadPoolExecutor(max_workers=3) as pool:
            found=sum(pool.map(download,selected))
        print(system,f'{found}/{len(selected)} matching covers',flush=True)
if __name__=='__main__':
    parser=argparse.ArgumentParser()
    parser.add_argument('plan',type=Path)
    parser.add_argument('output',type=Path)
    args=parser.parse_args()
    items=catalog(json.loads(args.plan.read_text(encoding='utf-8-sig')))
    print(f'Preparing {len(items)} private game records',flush=True)
    prepare(args.output,items)
