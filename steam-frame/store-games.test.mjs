import test from 'node:test';
import assert from 'node:assert/strict';
import {arrangeGames,createGameShelves,SYSTEMS} from './store-games.mjs';
import {mkdtemp,mkdir,writeFile,readFile} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import {join} from 'node:path';

test('all games occupy one console section without duplicates or pagination',()=>{
  const items=SYSTEMS.flatMap((system,s)=>Array.from({length:65},(_,i)=>({
    id:(s*100+i).toString(16).padStart(32,'0'),title:`Game ${i}`,system})));
  const bays=arrangeGames(items);
  assert.equal(bays.length,22);
  assert.equal(bays.flatMap(b=>b.items).length,items.length);
  assert.equal(new Set(bays.flatMap(b=>b.items.map(g=>g.id))).size,items.length);
  for(const bay of bays) {
    assert.ok(bay.items.length<=54);
    assert.ok(bay.items.every(i=>i.system===SYSTEMS[bay.system]));
  }
  assert.throws(()=>arrangeGames([items[0],items[0]]),/Duplicate/);
  assert.throws(()=>arrangeGames(Array.from({length:54*33},(_,i)=>({id:String(i),title:String(i),system:'nes'}))),/capacity/);
});

test('each console bay keeps its own IDs and artwork without modifying movie catalogs',async()=>{
  const base=await mkdtemp(join(tmpdir(),'framebuster-game-section-test-'));
  const library=join(base,'game-library'),session=join(base,'session');
  await mkdir(join(library,'artwork'),{recursive:true});await mkdir(session);
  const games=[{id:'a'.repeat(32),system:'nes',title:'First',art:'artwork/first.rgba'},
    {id:'b'.repeat(32),system:'psx',title:'Second',art:'artwork/second.rgba'},
    {id:'c'.repeat(32),system:'pcengine',title:'Third',art:'artwork/third.rgba'}];
  await writeFile(join(library,'catalog.json'),JSON.stringify(games));
  for(const [i,game] of games.entries()) {
    const pixels=Buffer.alloc(192*288*4,i+20);
    for(let p=3;p<pixels.length;p+=4)pixels[p]=255;
    await writeFile(join(library,game.art),pixels);
  }
  await writeFile(join(session,'catalog-path'),'unchanged-movie-catalog');
  await createGameShelves(base,session);
  const manifest=await readFile(await readFile(join(session,'game-catalog-path'),'utf8'),'utf8');
  const rows=manifest.split('\n').map(line=>line.split('\t'));
  assert.deepEqual(rows.map(r=>Number(r[1])),[0,3,10]);
  for(const [i,row] of rows.entries()) {
    const bytes=await readFile(join(session,row[3]));
    assert.equal(bytes.readUInt32LE(8),1);
    assert.equal(bytes.subarray(20,52).toString(),games[i].id);
    assert.equal(bytes[bytes.length-4],i+20);
    assert.equal(bytes[bytes.length-1],255);
  }
  assert.equal(await readFile(join(session,'catalog-path'),'utf8'),'unchanged-movie-catalog');
});
