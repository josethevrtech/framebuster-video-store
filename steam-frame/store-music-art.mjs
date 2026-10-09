import { spawn } from 'node:child_process';
import { writeFile } from 'node:fs/promises';
import { systemEnvironment } from './store-process.mjs';

const SIZE = 384;
let revision = 0;
function disc() {
  const bytes = Buffer.alloc(SIZE * SIZE * 4);
  for (let y = 0; y < SIZE; y++) for (let x = 0; x < SIZE; x++) {
    const distance = Math.hypot(x - SIZE / 2, y - SIZE / 2);
    const color = distance < 20 ? [20, 26, 35] : distance < 140
      ? [130 + Math.round(x / 5), 150 + Math.round(y / 6), 180] : [25, 34, 49];
    bytes.set([...color, 255], (y * SIZE + x) * 4);
  }
  return bytes;
}
function decode(bytes) {
  return new Promise((resolve, reject) => {
    const child = spawn('ffmpeg', ['-hide_banner', '-loglevel', 'error', '-threads', '1', '-filter_threads', '1',
      '-i', 'pipe:0', '-vf', `scale=${SIZE}:${SIZE}:force_original_aspect_ratio=decrease,pad=${SIZE}:${SIZE}:(ow-iw)/2:(oh-ih)/2:color=black,vflip`,
      '-frames:v', '1', '-f', 'rawvideo', '-pix_fmt', 'rgba', 'pipe:1'],
      { stdio: ['pipe', 'pipe', 'ignore'], env: systemEnvironment() });
    const chunks = []; let size = 0;
    child.stdout.on('data', chunk => { size += chunk.length; if (size > SIZE * SIZE * 4) child.kill(); else chunks.push(chunk); });
    child.stdin.on('error', () => {});
    child.once('error', reject);
    const timer = setTimeout(() => child.kill(), 10000);
    child.once('close', code => { clearTimeout(timer); code === 0 && size === SIZE * SIZE * 4
      ? resolve(Buffer.concat(chunks)) : reject(new Error('Album art conversion failed')); });
    child.stdin.end(bytes);
  });
}
async function publish(directory, bytes, current) {
  if (!current()) return;
  const path = `${directory}/music-art-${Date.now()}-${++revision}.rgba`;
  await writeFile(path, bytes, { mode: 0o600, flag: 'wx' });
  if (current()) await writeFile(`${directory}/music-art-path`, path, { mode: 0o600 });
}
export async function publishMusicArt(api, directory, track, current) {
  await publish(directory, disc(), current);
  for (const id of new Set([track.AlbumId, track.ParentId, track.Id].filter(id => /^[a-f0-9]{32}$/i.test(id || '')))) {
    try {
      const response = await api.request(`/Items/${id}/Images/Primary`,
        { query: { MaxWidth: SIZE, MaxHeight: SIZE, Format: 'jpg' } });
      const bytes = Buffer.from(await response.arrayBuffer());
      if (bytes.length > 8 * 1024 * 1024) continue;
      await publish(directory, await decode(bytes), current);
      return;
    } catch {}
  }
}
