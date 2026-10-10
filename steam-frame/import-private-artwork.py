from pathlib import Path
from zipfile import ZipFile
import sys


def import_artwork(archive_path,root):
    root=root.resolve()
    written=0
    with ZipFile(archive_path) as archive:
        entries=[item for item in archive.infolist() if not item.is_dir()]
        if len(entries)>10000 or sum(item.file_size for item in entries)>512*1024*1024:
            raise ValueError('Artwork archive exceeds capacity')
        pending=[]
        for item in entries:
            target=(root/item.filename).resolve()
            if not target.is_relative_to(root) or not item.filename.startswith('artwork/'):
                raise ValueError('Artwork path escapes private library')
            if item.file_size>8*1024*1024 or target.suffix not in ['.rgba','.png','.txt']:
                raise ValueError('Invalid artwork entry')
            if target.exists():
                if target.stat().st_size!=item.file_size or target.read_bytes()!=archive.read(item):
                    raise ValueError(f'Preserve conflicting existing artwork: {target}')
                continue
            pending.append((item,target))
        for item,target in pending:
            target.parent.mkdir(parents=True,exist_ok=True)
            with target.open('xb') as output: output.write(archive.read(item))
            written+=1
    return written


if __name__=='__main__':
    written=import_artwork(*map(Path,sys.argv[1:]))
    print(f'Imported {written} private artwork files; existing artwork preserved')
