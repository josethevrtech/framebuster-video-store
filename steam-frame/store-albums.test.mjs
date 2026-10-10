import test from 'node:test';
import assert from 'node:assert/strict';
import {arrangeAlbums,genreOf,loadAlbums} from './store-albums.mjs';
test('genre sections preserve every album once and stable selection offsets across all years',()=>{
  const items=Array.from({length:100},(_,i)=>({Id:i.toString(16).padStart(32,'0'),Name:`Album ${i}`,Type:'MusicAlbum',Genres:i<65?['Latin Pop']:['Electronic'],ProductionYear:i%2?2026:1975}));
  const bays=arrangeAlbums([...items,items[0]]);
  assert.equal(bays.length,3);assert.deepEqual(bays.map(b=>b.first),[0,54,65]);
  assert.equal(new Set(bays.flatMap(b=>b.items.map(i=>i.Id))).size,100);
  for(const b of bays)assert.ok(b.items.every(i=>genreOf(i)===b.genre));
  assert.equal(genreOf({Genres:['Hip Hop']}),7);assert.equal(genreOf({Genres:['Rap']}),7);
  assert.equal(genreOf({Genres:['Reggaeton, Latin Music']}),5);assert.equal(genreOf({}),15);
});
test('album loading paginates the full collection without a release-year restriction',async()=>{
  const starts=[];const api={account:{userId:'user'},async json(path,{query}) {
    starts.push(query.StartIndex);return {Items:Array(query.StartIndex?114:500).fill({Type:'MusicAlbum',ProductionYear:2026}),TotalRecordCount:614};
  }};
  assert.equal((await loadAlbums(api)).length,614);assert.deepEqual(starts,[0,500]);
});
