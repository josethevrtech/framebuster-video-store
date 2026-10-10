from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
from urllib.request import urlopen, Request
from zipfile import ZipFile
import json
import shutil

root=Path(__file__).resolve().parent.parent/'native/matineevr/assets/rental-hardware'
models={
 'nes':'6de5054340184509af6f301c7f3fac57',
 'snes':'d42c35b117974ade8842f0964f5fd59e',
 'genesis':'d1487aa8b216490ba7c414b95531479e',
 'psx':'314a3d6565994565a7272223bd965348',
 'n64':'816d53eca00e4f3192a8d23f62388472',
 'nes-cartridge':'e3c8de99be1740ae8fa551aa9b2596d2',
 'snes-cartridge':'b2076d8a65d648ff99bf51ca9d5fca2a',
 'n64-cartridge':'850607d366e9482dbb30f0bd0450f4fe',
 'genesis-cartridge':'c17dd351aa2743048430bf9a2b8af1dc',
 'gb-cartridge':'8b9728eab16c4056ac2636ae7f0f038f',
 'psx-case':'f6c8386e76e54918b551ebdd276479dd',
}
def fetch(entry):
 name,uid=entry
 folder=root/name
 folder.mkdir(parents=True,exist_ok=True)
 try:
  metadata=json.load(urlopen(Request(f'https://api.sketchfab.com/v3/models/{uid}',headers={'User-Agent':'FrameBuster asset preparation'}),timeout=20))
  if metadata['license']['slug'] not in ['by','cc0']: raise ValueError('Asset requires another license')
  (folder/'source.json').write_text(json.dumps(metadata,indent=2),encoding='utf-8')
  url=f'https://mirror.traines.eu/sketchfab-backup/{uid[:2]}/{uid}.zip'
  archive=folder/'source.zip'
  if not archive.exists():
   with urlopen(url,timeout=30) as source, archive.open('xb') as target:
    shutil.copyfileobj(source,target)
  with ZipFile(archive) as data:
   for item in data.infolist():
    target=(folder/item.filename).resolve()
    if not target.is_relative_to(folder.resolve()): raise ValueError('Archive path escapes asset folder')
   data.extractall(folder)
  license=f"{metadata['name']} by {metadata['user']['displayName']}\nSource: {metadata['viewerUrl']}\nLicense: {metadata['license']['url']}\nArchive mirror: {url}\nDownloaded 2026-10-09. Original files preserved; any runtime adaptation is described separately.\n"
  (folder/'FRAMEBUSTER-SOURCE.md').write_text(license,encoding='utf-8')
  print(name,'downloaded',flush=True)
 except Exception as error:
  print(name,type(error).__name__,str(error),flush=True)
with ThreadPoolExecutor(max_workers=3) as pool:
 list(pool.map(fetch,models.items()))
