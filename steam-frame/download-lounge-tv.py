from pathlib import Path
from urllib.request import Request,urlopen
from zipfile import ZipFile
import json
import shutil
import sys

folder=Path(sys.argv[1]).resolve()/'sony-pvm'
folder.mkdir(parents=True,exist_ok=True)
uid='5f80311f1a4646ef9673f20ee6dc6245'
metadata=json.load(urlopen(Request(f'https://api.sketchfab.com/v3/models/{uid}',
    headers={'User-Agent':'FrameBuster asset preparation'}),timeout=20))
if metadata['license']['slug']!='by': raise ValueError('Unexpected television asset license')
(folder/'source.json').write_text(json.dumps(metadata,indent=2),encoding='utf-8')
archive=folder/'source.zip'
if not archive.exists():
    url=f'https://mirror.traines.eu/sketchfab-backup/{uid[:2]}/{uid}.zip'
    with urlopen(url,timeout=60) as source,archive.open('xb') as target:
        shutil.copyfileobj(source,target)
with ZipFile(archive) as data:
    for item in data.infolist():
        if not (folder/item.filename).resolve().is_relative_to(folder):
            raise ValueError('Unsafe archive path')
    data.extractall(folder)
print('Licensed television source retained in',folder)
