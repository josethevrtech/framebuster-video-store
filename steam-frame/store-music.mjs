import { spawn } from 'node:child_process';
import { Readable } from 'node:stream';
import { systemEnvironment } from './store-process.mjs';
import { publishMusicArt } from './store-music-art.mjs';
import { createMusicRouting } from './store-audio-route.mjs';

export function eligibleMusic(item) {
  return item.Type === 'Audio' && Number.isInteger(item.ProductionYear)
    && item.ProductionYear > 0 && item.ProductionYear < 1999 && /^[a-f0-9]{32}$/i.test(item.Id);
}

export async function loadMusic(api) {
  const tracks = [];
  for (let start = 0;; start += 500) {
    const page = await api.json('/Items', { query: { UserId: api.account.userId,
      Recursive: true, IncludeItemTypes: 'Audio', StartIndex: start, Limit: 500,
      EnableImages: false, EnableUserData: false, Fields: 'ParentId', SortBy: 'SortName', SortOrder: 'Ascending' } });
    tracks.push(...(page.Items || []).filter(eligibleMusic));
    if (!page.Items?.length || start + page.Items.length >= page.TotalRecordCount) break;
  }
  for (let i = tracks.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [tracks[i], tracks[j]] = [tracks[j], tracks[i]];
  }
  return tracks;
}

export async function createStoreMusic(api, directory) {
  const tracks = await loadMusic(api);
  console.log(`Store music: ${tracks.length} audio tracks dated before 1999`);
  let closed = false, paused = false, manualPause = false, loading = false, player = null, stream = null, index = 0;
  const routing = createMusicRouting(directory, () => player);
  const next = async () => {
    if (closed || player || loading || paused || manualPause || !tracks.length) return;
    loading = true;
    const track = tracks[index++ % tracks.length];
    try {
      const response = await api.request(`/Audio/${track.Id}/stream`, { query: { Static: true }, timeout: 86400000 });
      if (closed || paused || manualPause) { await response.body.cancel(); return; }
      const child = spawn('ffplay', ['-nodisp', '-autoexit', '-loglevel', 'quiet', '-volume', '18',
        '-af', 'aformat=channel_layouts=stereo', '-i', 'pipe:0'],
        { stdio: ['pipe', 'ignore', 'ignore'], env: systemEnvironment() });
      player = child;
      void publishMusicArt(api, directory, track, () => !closed && player === child)
        .catch(() => console.error('Jukebox artwork unavailable'));
      stream = Readable.fromWeb(response.body);
      const input = stream;
      input.on('error', () => child.kill('SIGTERM'));
      child.stdin.on('error', () => input.destroy());
      child.once('error', () => console.error('Store music: cannot start audio player'));
      child.once('close', () => {
        input.destroy();
        if (player === child) { player = null; stream = null; }
        if (!closed) setTimeout(next, 1000).unref();
      });
      input.pipe(child.stdin);
    } catch {
      console.error('Store music: track unavailable; retrying');
      if (!closed) setTimeout(next, 5000).unref();
    } finally { loading = false; }
  };
  void next();
  const applyPause = () => {
    if (player) player.kill(paused || manualPause ? 'SIGSTOP' : 'SIGCONT');
    else if (!paused && !manualPause) void next();
  };
  const skip = () => {
    if (player) { player.kill('SIGCONT'); player.kill('SIGTERM'); }
    else void next();
  };
  return {
    setPaused(value) {
      paused = value;
      applyPause();
    },
    toggle() { manualPause = !manualPause; applyPause(); },
    next() { skip(); },
    previous() { index = Math.max(0, index - 2); skip(); },
    close() {
      closed = true;
      routing.close();
      stream?.destroy();
      if (player) { player.kill('SIGCONT'); player.kill('SIGTERM'); }
    },
  };
}
