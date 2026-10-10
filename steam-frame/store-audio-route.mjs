import { execFile } from 'node:child_process';
import { readFile } from 'node:fs/promises';
import { promisify } from 'node:util';
import { systemEnvironment } from './store-process.mjs';

const execute = promisify(execFile);
export function ownedMusicInput(inputs, pid) {
  return inputs.find(input => Number.isInteger(input.index)
    && input.properties?.['application.process.id'] === String(pid)
    && input.properties?.['application.name'] === 'ffplay');
}
export function channelVolumes(gains, channels) {
  if (!Array.isArray(gains) || gains.length !== 2 || gains.some(g => !Number.isFinite(g) || g < 0 || g > 1))
    throw new Error('Invalid proximity gains');
  if (channels === 1) return [`${((gains[0] + gains[1]) * 50).toFixed(2)}%`];
  if (channels !== 2) throw new Error('Unsupported music channel count');
  return gains.map(g => `${(g * 100).toFixed(2)}%`);
}
export function createMusicRouting(directory, getPlayer) {
  let busy = false, closed = false, warned = false;
  const timer = setInterval(async () => {
    const player = getPlayer();
    if (closed || busy || !player?.pid) return;
    busy = true;
    try {
      const gains = JSON.parse(await readFile(`${directory}/music-volume`, 'utf8'));
      const options = { env: systemEnvironment(), timeout: 2000, maxBuffer: 1024 * 1024 };
      const { stdout } = await execute('pactl', ['--format=json', 'list', 'sink-inputs'], options);
      const input = ownedMusicInput(JSON.parse(stdout), player.pid);
      if (!input || closed || getPlayer() !== player) return;
      const volumes = channelVolumes(gains, Object.keys(input.volume || {}).length);
      await execute('pactl', ['set-sink-input-volume', String(input.index), ...volumes], options);
    } catch (error) {
      if (error.code !== 'ENOENT' && !warned) { console.error('Store proximity audio temporarily unavailable'); warned = true; }
    } finally { busy = false; }
  }, 250);
  return { close() { closed = true; clearInterval(timer); } };
}
