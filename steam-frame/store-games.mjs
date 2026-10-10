import { readFile, writeFile } from 'node:fs/promises';
import { join, resolve, sep } from 'node:path';

export const SYSTEMS=['nes','snes','genesis','psx','n64'];
const CAPACITY=54, PIXELS=192*288*4;
function number(value) { const bytes=Buffer.alloc(4);bytes.writeUInt32LE(value);return bytes; }
function text(value) { const bytes=Buffer.from(String(value).slice(0,240));return Buffer.concat([number(bytes.length),bytes]); }
export function selectGames(items,system,page) {
  let filtered=items.filter(item=>SYSTEMS.includes(item.system) && (!system || item.system===system));
  if(!system) {
    const groups=SYSTEMS.map(system=>filtered.filter(item=>item.system===system));
    filtered=[];
    for(let row=0;row<Math.max(0,...groups.map(group=>group.length));row++)
      for(const group of groups)if(group[row])filtered.push(group[row]);
  }
  const pages=Math.max(1,Math.ceil(filtered.length/CAPACITY));
  page=Math.max(0,Math.min(pages-1,page));
  return {items:filtered.slice(page*CAPACITY,(page+1)*CAPACITY),page,pages};
}
export async function createGameShelves(base,directory) {
  const root=resolve(base,'game-library');
  const catalog=JSON.parse(await readFile(join(root,'catalog.json'),'utf8'));
  if (!Array.isArray(catalog) || catalog.length>10000) throw new Error('Invalid private game catalog');
  const items=catalog.filter(item=>/^[a-f0-9]{32}$/.test(item.id) && typeof item.title==='string' && SYSTEMS.includes(item.system));
  let system=null,page=0,revision=0;
  const publish=async()=>{
    const selected=selectGames(items,system,page);page=selected.page;
    const records=[];
    for (const item of selected.items) {
      let pixels=Buffer.alloc(PIXELS,25);
      for(let i=3;i<pixels.length;i+=4)pixels[i]=255;
      const path=resolve(root,item.art);
      if(!path.startsWith(root+sep))throw new Error('Game artwork escapes library');
      try { const art=await readFile(path);if(art.length===PIXELS)pixels=art; } catch {}
      records.push(Buffer.concat([Buffer.from(item.id),text(item.title),number(0),number(192),number(288),pixels]));
    }
    const path=join(directory,`games-${++revision}.bin`);
    await writeFile(path,Buffer.concat([Buffer.from('FBCAT001'),number(records.length),number(page),number(items.length),...records]),{mode:0o600});
    await writeFile(path+'.types',Buffer.from(selected.items.map(item=>SYSTEMS.indexOf(item.system))),{mode:0o600});
    await writeFile(join(directory,'game-catalog-path'),path,{mode:0o600});
    console.log(`Game shelves: ${system || 'all consoles'}, ${records.length} games, page ${page+1}/${selected.pages}`);
  };
  await publish();
  return { async command(action,value) {
    if(action==='game-console') {if(!Number.isInteger(value)||value<0||value>=SYSTEMS.length)return;system=SYSTEMS[value];page=0;}
    else if(action==='game-next')page++;
    else if(action==='game-previous')page--;
    else return;
    await publish();
  }};
}
