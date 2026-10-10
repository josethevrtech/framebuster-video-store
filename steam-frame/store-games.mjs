import { readFile, writeFile } from 'node:fs/promises';
import { join, resolve, sep } from 'node:path';
import systems from './game-systems.json' with {type:'json'};

export const SYSTEMS=systems.map(system=>system.id);
const CAPACITY=54, MAX_BAYS=32, PIXELS=192*288*4;
function number(value) { const bytes=Buffer.alloc(4);bytes.writeUInt32LE(value);return bytes; }
function text(value) { const bytes=Buffer.from(String(value).slice(0,240));return Buffer.concat([number(bytes.length),bytes]); }
export function arrangeGames(items) {
  const bays=[];
  const identities=new Set();
  for(const [systemIndex,system] of SYSTEMS.entries()) {
    const selected=items.filter(item=>item.system===system).sort((a,b)=>a.title.localeCompare(b.title));
    for(let start=0;start<selected.length;start+=CAPACITY) {
      const games=selected.slice(start,start+CAPACITY);
      for(const game of games) {
        if(identities.has(game.id))throw new Error('Duplicate game identity');
        identities.add(game.id);
      }
      bays.push({bay:bays.length,system:systemIndex,items:games});
    }
  }
  if(bays.length>MAX_BAYS)throw new Error('Game library exceeds physical shelf capacity');
  return bays;
}
export async function createGameShelves(base,directory) {
  const root=resolve(base,'game-library');
  const catalog=JSON.parse(await readFile(join(root,'catalog.json'),'utf8'));
  if(!Array.isArray(catalog)||catalog.length>10000)throw new Error('Invalid private game catalog');
  const items=catalog.filter(item=>/^[a-f0-9]{32}$/.test(item.id)&&typeof item.title==='string'&&SYSTEMS.includes(item.system));
  const bays=arrangeGames(items),manifest=[];
  for(const {bay,system,items:games} of bays) {
    const records=[];
    for(const item of games) {
      let pixels=Buffer.alloc(PIXELS,25);
      for(let i=3;i<pixels.length;i+=4)pixels[i]=255;
      const path=resolve(root,item.art);
      if(!path.startsWith(root+sep))throw new Error('Game artwork escapes library');
      try {const art=await readFile(path);if(art.length===PIXELS)pixels=art;}catch {}
      records.push(Buffer.concat([Buffer.from(item.id),text(item.title),number(0),number(192),number(288),pixels]));
    }
    const name=`games-bay-${bay}.bin`;
    await writeFile(join(directory,name),Buffer.concat([Buffer.from('FBCAT001'),number(records.length),number(0),number(records.length),...records]),{mode:0o600});
    manifest.push(`${bay}\t${system}\t${records.length}\t${name}`);
  }
  const path=join(directory,'game-shelves.manifest');
  await writeFile(path,manifest.join('\n'),{mode:0o600});
  await writeFile(join(directory,'game-catalog-path'),path,{mode:0o600});
  console.log(`Game shelves: ${items.length} games, ${bays.length} console bays, all visible without paging`);
}
