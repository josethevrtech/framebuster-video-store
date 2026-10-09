import { test } from 'node:test';
import assert from 'node:assert/strict';
import { eligibleMusic, loadMusic } from './store-music.mjs';

test('music excludes 1999, newer, missing or invalid years and video', () => {
  const item = { Id: 'a'.repeat(32), Type: 'Audio', ProductionYear: 1998 };
  assert.equal(eligibleMusic(item), true);
  for (const year of [1999, 2020, null, undefined, 0, '1990'])
    assert.equal(eligibleMusic({ ...item, ProductionYear: year }), false);
  assert.equal(eligibleMusic({ ...item, Type: 'Movie' }), false);
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
  assert.deepEqual(await loadMusic(api), [eligible]);
  assert.deepEqual(starts, [0, 500]);
});
