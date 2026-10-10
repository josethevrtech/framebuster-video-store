import test from 'node:test';
import assert from 'node:assert/strict';
import {selectGames} from './store-games.mjs';
import {createGameShelves} from './store-games.mjs';
import {mkdtemp,mkdir,writeFile,readFile} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import {join} from 'node:path';

test('game pages preserve identities and clamp separately for each console',()=>{
  const items=Array.from({length:120},(_,id)=>({id,system:id<60?'nes':'psx'}));
  assert.deepEqual(selectGames(items,'nes',1).items.map(i=>i.id),[54,55,56,57,58,59]);
  assert.equal(selectGames(items,'psx',0).items[0].id,60);
  assert.equal(selectGames(items,'nes',999).page,1);
  assert.equal(selectGames(items,'nes',-1).page,0);
  assert.equal(selectGames(items,'n64',0).items.length,0);
});

test('game artwork and model types stay tied to IDs without modifying movie catalogs',async()=>{
  const base=await mkdtemp(join(tmpdir(),'framebuster-game-shelf-test-'));
  const library=join(base,'game-library'),session=join(base,'session');
  await mkdir(join(library,'artwork'),{recursive:true});await mkdir(session);
  const games=[{id:'a'.repeat(32),system:'nes',title:'First',art:'artwork/first.rgba'},
    {id:'b'.repeat(32),system:'psx',title:'Second',art:'artwork/second.rgba'}];
  await writeFile(join(library,'catalog.json'),JSON.stringify(games));
  for(const [i,game] of games.entries())await writeFile(join(library,game.art),Buffer.alloc(192*288*4,i+20));
  await writeFile(join(session,'catalog-path'),'unchanged-movie-catalog');
  const shelves=await createGameShelves(base,session);
  let path=await readFile(join(session,'game-catalog-path'),'utf8');
  assert.deepEqual([...await readFile(path+'.types')],[0,3]);
  const mixed=await readFile(path);
  assert.equal(mixed.readUInt32LE(8),2);
  assert.equal(mixed.subarray(20,52).toString(),games[0].id);
  await shelves.command('game-console',3);
  path=await readFile(join(session,'game-catalog-path'),'utf8');
  const selected=await readFile(path);
  assert.equal(selected.readUInt32LE(8),1);
  assert.equal(selected.subarray(20,52).toString(),games[1].id);
  assert.deepEqual([...await readFile(path+'.types')],[3]);
  assert.equal(await readFile(join(session,'catalog-path'),'utf8'),'unchanged-movie-catalog');
});
