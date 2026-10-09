import { spawn } from 'node:child_process';
import { writeFile } from 'node:fs/promises';
import { systemEnvironment } from './store-process.mjs';

const WIDTH = 192, HEIGHT = 288;
export const SHELF_CAPACITY = 54;
function decode(bytes) {
  return new Promise((resolve, reject) => {
    const child = spawn('ffmpeg', ['-hide_banner', '-loglevel', 'error', '-threads', '1', '-filter_threads', '1', '-i', 'pipe:0', '-vf', `scale=${WIDTH}:${HEIGHT},vflip`, '-frames:v', '1', '-f', 'rawvideo', '-pix_fmt', 'rgba', 'pipe:1'], { stdio: ['pipe', 'pipe', 'ignore'], env: systemEnvironment() });
    const chunks = []; let size = 0;
    child.stdout.on('data', chunk => { size += chunk.length; if (size > WIDTH * HEIGHT * 4) child.kill(); else chunks.push(chunk); });
    child.stdin.on('error', () => {});
    child.once('error', reject);
    const timer = setTimeout(() => child.kill(), 10000);
    child.once('close', code => { clearTimeout(timer); code === 0 && size === WIDTH * HEIGHT * 4 ? resolve(Buffer.concat(chunks)) : reject(new Error('Artwork conversion failed')); });
    child.stdin.end(bytes);
  });
}
function u32(value) { const b = Buffer.alloc(4); b.writeUInt32LE(value); return b; }
function text(value, limit) { const b = Buffer.from(String(value || '').slice(0, limit)); return Buffer.concat([u32(b.length), b]); }
export async function loadCatalog(api, directory, page, search, revision, series = null) {
  const result = await api.json(series ? `/Shows/${series.Id}/Episodes` : `/Users/${api.account.userId}/Items`, { query: {
    UserId: api.account.userId, Recursive: true, IncludeItemTypes: 'Movie,Series', StartIndex: page * SHELF_CAPACITY, Limit: SHELF_CAPACITY,
    Fields: 'Overview,MediaSources', EnableUserData: true,
    ...(series ? {} : { SortBy: 'SortName', SortOrder: 'Ascending', SearchTerm: search || '' }),
  } });
  const items = (result.Items || []).filter(item => /^[a-f0-9]{32}$/i.test(item.Id)).slice(0, SHELF_CAPACITY);
  const artwork = new Map();
  const pictures = await Promise.all(items.map(item => {
    const artId = item.Type === 'Episode' && /^[a-f0-9]{32}$/i.test(item.SeriesId) ? item.SeriesId : item.Id;
    if (!artwork.has(artId)) artwork.set(artId, (async () => {
    try {
      const response = await api.request(`/Items/${artId}/Images/Primary`, { query: { MaxWidth: WIDTH, MaxHeight: HEIGHT, Format: 'jpg' } });
      const bytes = Buffer.from(await response.arrayBuffer());
      if (bytes.length > 8 * 1024 * 1024) throw new Error('Artwork too large');
      return await decode(bytes);
    } catch {
      const pixels = Buffer.alloc(WIDTH * HEIGHT * 4, 45);
      for (let i = 3; i < pixels.length; i += 4) pixels[i] = 255;
      return pixels;
    }
    })());
    return artwork.get(artId);
  }));
  const records = items.map((item, i) => Buffer.concat([Buffer.from(item.Id.toLowerCase()),
    text(item.Type === 'Series' ? `${item.Name} [Series]` : item.Type === 'Episode'
      ? `S${String(item.ParentIndexNumber || 0).padStart(2, '0')}E${String(item.IndexNumber || 0).padStart(2, '0')} ${item.Name}` : item.Name, 240),
    text(item.Overview, 2400), u32(WIDTH), u32(HEIGHT), pictures[i]]));
  const data = Buffer.concat([Buffer.from('FBCAT001'), u32(items.length), u32(page), u32(result.TotalRecordCount || items.length), ...records]);
  const path = `${directory}/catalog-${revision}.bin`;
  await writeFile(path, data, { mode: 0o600 });
  await writeFile(`${directory}/catalog-path`, path, { mode: 0o600 });
  return { items, total: result.TotalRecordCount || items.length };
}
