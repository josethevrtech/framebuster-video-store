import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, readFile, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { StoreApi } from '../steam-frame/store-api.mjs';
import { loadCatalog } from '../steam-frame/store-catalog.mjs';
import { createStorePlayback } from '../steam-frame/store-playback.mjs';
import { systemEnvironment } from '../steam-frame/store-process.mjs';

test('native Jellyfin client keeps device identity and tokens in request headers', async () => {
  const requests: any[] = [];
  const api = new StoreApi('http://server.test:8096/jellyfin', async (url: URL, options: any) => {
    requests.push({ url: String(url), options });
    return new Response('{}');
  });
  api.account = { token: 'private-token', userId: 'a'.repeat(32) };
  await api.json('/Users/Me', { query: { Fields: 'Overview' } });
  assert.equal(requests[0].url, 'http://server.test:8096/jellyfin/Users/Me?Fields=Overview');
  assert.equal(requests[0].options.headers['X-Emby-Token'], 'private-token');
  assert.match(requests[0].options.headers.Authorization, /FrameBuster/);
  assert.equal(requests[0].options.redirect, 'manual');
  assert.throws(() => new StoreApi('file:///etc/passwd'));
  assert.throws(() => new StoreApi('http://user:password@server.test'));
  assert.throws(() => new StoreApi('http://server.test?key=secret'));
  const isolated = systemEnvironment();
  assert.equal('LD_LIBRARY_PATH' in isolated, false);
  assert.equal('LD_PRELOAD' in isolated, false);
});

test('native catalog bounds identities and creates complete poster records without credentials', async () => {
  const directory = await mkdtemp(join(tmpdir(), 'framebuster-catalog-'));
  const id = 'a'.repeat(32);
  const api = { account: { userId: id }, json: async () => ({ Items: [
    { Id: id, Name: 'A real movie', Overview: 'A description' }, { Id: '../bad', Name: 'Invalid' },
  ], TotalRecordCount: 8 }), request: async () => { throw new Error('No artwork'); } };
  const result = await loadCatalog(api, directory, 0, '', 1);
  assert.equal(result.items.length, 1);
  const path = await readFile(join(directory, 'catalog-path'), 'utf8');
  const bytes = await readFile(path);
  assert.equal(bytes.subarray(0, 8).toString(), 'FBCAT001');
  assert.equal(bytes.readUInt32LE(8), 1);
  assert.equal(bytes.readUInt32LE(16), 8);
  assert.equal(bytes.subarray(20, 52).toString(), id);
  let offset = 52;
  for (const text of ['A real movie', 'A description']) {
    const length = bytes.readUInt32LE(offset); offset += 4;
    assert.equal(bytes.subarray(offset, offset + length).toString(), text); offset += length;
  }
  assert.equal(bytes.readUInt32LE(offset), 192);
  assert.equal(bytes.readUInt32LE(offset + 4), 288);
  assert.equal(bytes.length, offset + 8 + 192 * 288 * 4);
});

test('series use paged episodes, user progress and individual artwork with series fallback', async () => {
  const directory = await mkdtemp(join(tmpdir(), 'framebuster-episodes-'));
  const seriesId = 'c'.repeat(32), episodeId = 'd'.repeat(32);
  const requests: any[] = [];
  const api = { account: { userId: 'a'.repeat(32) },
    json: async (path: string, options: any) => {
      requests.push({ path, ...options });
      return { Items: [{ Id: episodeId, SeriesId: seriesId, Type: 'Episode', Name: 'Pilot',
        ParentIndexNumber: 1, IndexNumber: 2 }], TotalRecordCount: 23 };
    }, request: async (path: string) => { requests.push({ path }); throw new Error('No artwork'); } };
  await loadCatalog(api, directory, 1, '', 1, { Id: seriesId });
  assert.equal(requests[0].path, `/Shows/${seriesId}/Episodes`);
  assert.equal(requests[0].query.StartIndex, 54);
  assert.equal(requests[0].query.Limit, 54);
  assert.equal(requests[0].query.EnableUserData, true);
  assert.equal('SortBy' in requests[0].query, false);
  assert.equal(requests[1].path, `/Items/${episodeId}/Images/Primary`);
  assert.equal(requests[2].path, `/Items/${seriesId}/Images/Primary`);
  const bytes = await readFile(await readFile(join(directory, 'catalog-path'), 'utf8'));
  assert.match(bytes.toString('utf8', 56, 56 + bytes.readUInt32LE(52)), /S01E02 Pilot/);
});

test('native movie requests use an opaque relay and report progress without launching a browser', async () => {
  const directory = await mkdtemp(join(tmpdir(), 'framebuster-playback-'));
  const itemId = 'a'.repeat(32);
  const notifications: any[] = [];
  const music: boolean[] = [];
  let returns = 0;
  const api = { server: new URL('http://server.test:8096'), account: { userId: 'b'.repeat(32), token: 'private-token' },
    json: async () => ({ MediaSources: [{ Id: itemId }], PlaySessionId: 'c'.repeat(32) }),
    request: async (path: string, options: any) => { notifications.push({ path, data: options.data }); return new Response(); } };
  const playback = await createStorePlayback(api, directory, async () => {}, paused => music.push(paused), () => { returns++; });
  try {
    await playback.play({ Id: itemId, Name: 'Movie', UserData: { PlaybackPositionTicks: 420000000 } });
    const [, url, start] = (await readFile(join(directory, 'movie-request'), 'utf8')).split('\n');
    assert.match(url, /^http:\/\/127\.0\.0\.1:\d+\/__frame\/media\//);
    assert.equal(start, '42');
    assert.equal(Buffer.from(url.split('/')[6], 'base64url').toString().includes('private-token'), false);
    await writeFile(join(directory, 'progress.json'), JSON.stringify({ position: 43, running: true, paused: false, ended: false, error: false }));
    await new Promise(resolve => setTimeout(resolve, 1300));
    assert.equal(notifications[0].path, '/Sessions/Playing');
    assert.equal(notifications[0].data.PositionTicks, 430000000);
    assert.deepEqual(music, [true]);
    await writeFile(join(directory, 'progress.json'), JSON.stringify({ position: 43, running: false, paused: false, ended: false, error: false }));
    await new Promise(resolve => setTimeout(resolve, 1300));
    assert.deepEqual(music, [true, false]);
    assert.equal(returns, 1);
  } finally { await playback.close(); }
});
