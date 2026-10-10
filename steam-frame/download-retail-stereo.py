from pathlib import Path
from urllib.request import Request, urlopen
from zipfile import ZipFile
import json
import shutil
import sys

folder=Path(sys.argv[1]).resolve()/'panasonic'
folder.mkdir(parents=True,exist_ok=True)
uid='e6b0d376c49d4d4c9ce7c1502661c8e4'
metadata=json.load(urlopen(Request(f'https://api.sketchfab.com/v3/models/{uid}',headers={'User-Agent':'FrameBuster asset preparation'}),timeout=20))
if metadata['license']['slug']!='by': raise ValueError('Unexpected stereo asset license')
(folder/'source.json').write_text(json.dumps(metadata,indent=2),encoding='utf-8')
url=f'https://mirror.traines.eu/sketchfab-backup/{uid[:2]}/{uid}.zip'
archive=folder/'source.zip'
if not archive.exists():
    with urlopen(url,timeout=40) as source,archive.open('xb') as target: shutil.copyfileobj(source,target)
with ZipFile(archive) as data:
    for item in data.infolist():
        if not (folder/item.filename).resolve().is_relative_to(folder): raise ValueError('Unsafe archive path')
    data.extractall(folder)
print('Licensed stereo source retained in',folder)
