import { spawn } from 'node:child_process';
import { Readable } from 'node:stream';
import { systemEnvironment } from './store-process.mjs';
import { publishMusicArt } from './store-music-art.mjs';
import { createMusicRouting } from './store-audio-route.mjs';

export function eligibleMusic(item) {
  return item.Type === 'Audio' && /^[a-f0-9]{32}$/i.test(item.Id);
}

export async function loadMusic(api) {
  const tracks = [];
  for (let start = 0;; start += 500) {
    const page = await api.json('/Items', { query: { UserId: api.account.userId,
      Recursive: true, IncludeItemTypes: 'Audio', StartIndex: start, Limit: 500,
      EnableImages: false, EnableUserData: false, Fields: 'ParentId,AlbumId', SortBy: 'SortName', SortOrder: 'Ascending' } });
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
  console.log(`Store music: ${tracks.length} audio tracks, any year`);
  let queue=tracks,selection=0,albumRequest=0;
  let closed = false, paused = false, manualPause = false, loading = false, player = null, stream = null, index = 0;
  const routing = createMusicRouting(directory, () => player);
  const next = async () => {
    if (closed || player || loading || paused || manualPause || !queue.length) return;
    loading = true;
    const track = queue[index++ % queue.length];const generation=selection;
    try {
      const response = await api.request(`/Audio/${track.Id}/stream`, { query: { Static: true }, timeout: 86400000 });
      if (closed || paused || manualPause || generation!==selection) { await response.body.cancel(); return; }
      const child = spawn('ffplay', ['-nodisp', '-autoexit', '-loglevel', 'quiet', '-volume', '18',
        '-af', 'aformat=channel_layouts=stereo', '-i', 'pipe:0'],
        { stdio: ['pipe', 'ignore', 'ignore'], env: systemEnvironment() });
      player = child;
      void publishMusicArt(api, directory, track, () => !closed && player === child)
        .catch(() => console.error('Now-playing artwork unavailable'));
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
    } finally { loading = false;if(generation!==selection&&!closed)setTimeout(next,0).unref(); }
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
    async album(id) {
      if(!/^[a-f0-9]{32}$/i.test(id||''))throw new Error('Invalid album identity');
      const request=++albumRequest;
      const items=[];
      for(let start=0;;start+=500) {
        const page=await api.json('/Items',{query:{UserId:api.account.userId,ParentId:id,Recursive:true,IncludeItemTypes:'Audio',StartIndex:start,Limit:500,Fields:'ParentId,AlbumId',SortBy:'ParentIndexNumber,IndexNumber,SortName',SortOrder:'Ascending'}});
        items.push(...(page.Items||[]).filter(eligibleMusic));if(!page.Items?.length||start+page.Items.length>=page.TotalRecordCount)break;
      }
      if(!items.length)throw new Error('This album has no playable audio tracks');
      if(closed||request!==albumRequest)return;
      console.log(`Store music: selected album, ${items.length} tracks in disc/track order`);
      selection++;queue=items;index=0;manualPause=false;skip();
    },
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
