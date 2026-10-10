import { test } from 'node:test';
import assert from 'node:assert/strict';
import { eligibleMusic, loadMusic } from './store-music.mjs';

test('music allows every year but rejects non-audio items and invalid identities', () => {
  const item = { Id: 'a'.repeat(32), Type: 'Audio', ProductionYear: 1998 };
  assert.equal(eligibleMusic(item), true);
  for (const year of [1999, 2020, null, undefined, 0, '1990'])
    assert.equal(eligibleMusic({ ...item, ProductionYear: year }), true);
  assert.equal(eligibleMusic({ ...item, Type: 'Movie' }), false);
  assert.equal(eligibleMusic({...item,Id:'invalid'}),false);
});

test('music filters every page rather than truncating the library', async () => {
  const eligible = { Id: 'b'.repeat(32), Type: 'Audio', ProductionYear: 1990 };
  const starts = [];
  const api = { account: { userId: 'user' }, async json(path, { query }) {
    starts.push(query.StartIndex);
    return query.StartIndex === 0
      ? { Items: Array(500).fill({ ...eligible, ProductionYear: 2000 }), TotalRecordCount: 501 }
      : { Items: [eligible], TotalRecordCount: 501 };
  } };
  const tracks=await loadMusic(api);
  assert.equal(tracks.length,501);
  assert.equal(tracks.filter(t=>t.ProductionYear===2000).length,500);
  assert.equal(tracks.filter(t=>t.ProductionYear===1990).length,1);
  assert.deepEqual(starts, [0, 500]);
});
