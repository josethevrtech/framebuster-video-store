import argparse, base64, io, json, re, subprocess, tarfile, uuid
from pathlib import Path, PurePosixPath
from zipfile import ZipFile
from xml.etree import ElementTree as ET

SYSTEMS = [
 ('Nintendo - NES.zip','nes','Nintendo - Nintendo Entertainment System'),
 ('Nintendo - SNES.zip','snes','Nintendo - Super Nintendo Entertainment System'),
 ('Nintendo - N64.zip','n64','Nintendo - Nintendo 64'),
 ('Nintendo - Game Boy.zip','gb','Nintendo - Game Boy'),
 ('Nintendo - Game Boy Color.zip','gbc','Nintendo - Game Boy Color'),
 ('Sega - Genesis.zip','genesis','Sega - Mega Drive - Genesis'),
 ('Sega - Genesis (Update 1).zip','genesis','Sega - Mega Drive - Genesis'),
 ('Sega - Master System.zip','mastersystem','Sega - Master System - Mark III'),
 ('Sega - Game Gear.zip','gamegear','Sega - Game Gear'),
 ('Atari - 2600.zip','atari2600','Atari - 2600'),
 ('NEC - TurboGrafx-16.zip','pcengine','NEC - PC Engine - TurboGrafx 16'),
]

def clean(name):
 return re.sub(r'\s+', ' ', re.sub(r'\s*\[[^]]*\]|\s*\((?:Rev|v)[^)]*\)', '', name)).strip().casefold()

def plan(source, metadata):
 result, skipped, seen = [], [], set()
 for archive, system, database in SYSTEMS:
  data=(metadata/(database+'.dat')).read_text(encoding='utf-8-sig')
  crc_year, name_year={},{}
  for block in re.split(r'\bgame\s*\(', data)[1:]:
   year=re.search(r'releaseyear\s+"(\d{4})"',block)
   title=re.search(r'comment\s+"([^"]+)"',block)
   crc=re.search(r'crc\s+([0-9a-fA-F]+)',block)
   if year and title:
    name_year[clean(title[1])]=int(year[1])
    if crc: crc_year[int(crc[1],16)]=int(year[1])
  with ZipFile(source/archive) as outer:
   for entry in outer.infolist():
    if not entry.filename.lower().endswith('.zip'): continue
    raw=outer.read(entry)
    with ZipFile(io.BytesIO(raw)) as inner:
     roms=[i for i in inner.infolist() if not i.is_dir()]
     if len(roms)!=1: skipped.append([archive,entry.filename,'not a single ROM']); continue
     rom=roms[0]
     year=crc_year.get(rom.CRC,name_year.get(clean(Path(entry.filename).stem)))
     if not year or year>1999: skipped.append([archive,entry.filename,'unknown year' if not year else year]); continue
     destination=system+'/'+Path(rom.filename).name
     if destination in seen: continue
     seen.add(destination)
     result.append(dict(archive=archive,entry=entry.filename,inner=rom.filename,destination=destination,
       title=Path(entry.filename).stem,year=year,size=rom.file_size,metadata='libretro releaseyear'))
 shared_games = None
 for archive in ['Sony - PS1 (A-L).zip','Sony - PS1 (L-Z).zip','Sony - PS1 (Update 1).zip']:
  with ZipFile(source/archive) as outer:
   names={Path(n).name:n for n in outer.namelist()}
   xml=next((n for n in outer.namelist() if n.endswith('gamelist.xml')),None)
   if xml: shared_games = ET.fromstring(outer.read(xml)).findall('game')
   if shared_games is None: skipped.append([archive,'all','missing release metadata']); continue
   for game in shared_games:
    date=game.findtext('releasedate','')
    if not re.match(r'^\d{4}',date) or not 1<=int(date[:4])<=1999: continue
    playlist=Path(game.findtext('path','')).name
    if playlist not in names or not playlist.endswith('.m3u'): continue
    refs=outer.read(names[playlist]).decode('utf-8-sig').splitlines()
    files=[playlist]+[Path(r.replace('\\','/')).name for r in refs if r.strip() and not r.startswith('#')]
    if any(n not in names for n in files): skipped.append([archive,playlist,'missing disc']); continue
    for filename in files:
     destination='psx/'+filename
     if destination in seen: continue
     seen.add(destination)
     result.append(dict(archive=archive,entry=names[filename],destination=destination,title=game.findtext('name'),
       year=int(date[:4]),size=outer.getinfo(names[filename]).file_size,metadata='archive ScreenScraper releasedate'))
 return dict(files=result,skipped=skipped)

RECEIVER = '''
import sys,tarfile,json,hashlib,uuid
from pathlib import Path,PurePosixPath
base=Path.home()/'Emulation/roms'
state=Path.home()/'.local/share/halcyon-frame/game-library'
state.mkdir(parents=True,exist_ok=True)
manifest=state/('import-'+str(uuid.uuid4())+'.jsonl')
count=0
with manifest.open('x') as log, tarfile.open(fileobj=sys.stdin.buffer,mode='r|') as archive:
 for member in archive:
  relative=PurePosixPath(member.name)
  if not member.isfile() or relative.is_absolute() or '..' in relative.parts or len(relative.parts)!=2:
   raise RuntimeError('Invalid library path')
  target=base.joinpath(*relative.parts)
  target.parent.mkdir(parents=True,exist_ok=True)
  if target.parent.resolve()!=base.resolve()/relative.parts[0]: raise RuntimeError('Unsafe library folder')
  source=archive.extractfile(member)
  record=json.loads(member.pax_headers['framebuster.metadata'])
  if target.is_symlink(): raise RuntimeError('Existing library file is a symbolic link: '+member.name)
  if target.exists():
   if not target.is_file() or target.stat().st_size!=member.size: raise RuntimeError('Existing file has a different size: '+member.name)
   record['result']='preserved existing'
  else:
   digest=hashlib.sha256()
   with target.open('xb') as output:
    for chunk in iter(lambda:source.read(1024*1024),b''):
     output.write(chunk); digest.update(chunk)
   record.update(result='copied',sha256=digest.hexdigest())
  log.write(json.dumps(record)+'\\n'); log.flush()
  count+=1
  if count%20==0: print('Imported entries:',count,file=sys.stderr,flush=True)
print('Library import complete:',count,'entries; manifest',manifest,file=sys.stderr,flush=True)
'''

def transfer(source, selection, key, host):
 encoded=base64.b64encode(RECEIVER.encode()).decode()
 process=subprocess.Popen(['ssh','-i',str(key),'-o','BatchMode=yes',host,
  'python3 -c "import base64;exec(base64.b64decode(\''+encoded+'\'))"'],stdin=subprocess.PIPE)
 try:
  with tarfile.open(fileobj=process.stdin,mode='w|',format=tarfile.PAX_FORMAT) as stream:
   for item in selection['files']:
    with ZipFile(source/item['archive']) as outer:
     if 'inner' in item:
      with ZipFile(io.BytesIO(outer.read(item['entry']))) as inner:
       contents=inner.open(item['inner']); add(stream,item,contents)
     else:
      with outer.open(item['entry']) as contents: add(stream,item,contents)
  process.stdin.close()
  if process.wait()!=0: raise RuntimeError('Headset import failed; partial files are preserved')
 except BaseException:
  process.stdin.close(); process.wait(); raise

def add(stream,item,contents):
 info=tarfile.TarInfo(item['destination']); info.size=item['size']; info.mode=0o644
 info.pax_headers={'framebuster.metadata':json.dumps(item)}
 stream.addfile(info,contents)

if __name__=='__main__':
 parser=argparse.ArgumentParser()
 parser.add_argument('--source',type=Path,required=True); parser.add_argument('--metadata',type=Path,required=True)
 parser.add_argument('--plan',type=Path,required=True); parser.add_argument('--copy',action='store_true')
 parser.add_argument('--key',type=Path); parser.add_argument('--host',default='steamos@192.168.0.76')
 args=parser.parse_args()
 if args.plan.exists(): selection=json.loads(args.plan.read_text())
 else:
  selection=plan(args.source,args.metadata)
  with args.plan.open('x',encoding='utf-8') as output: json.dump(selection,output,ensure_ascii=False,indent=2)
 print(json.dumps(dict(entries=len(selection['files']),bytes=sum(i['size'] for i in selection['files']),
   bySystem={s:sum(i['destination'].startswith(s+'/') for i in selection['files']) for s in sorted({i['destination'].split('/')[0] for i in selection['files']})})),flush=True)
 if args.copy:
  if not args.key: parser.error('--key is required to copy')
  transfer(args.source,selection,args.key,args.host)
