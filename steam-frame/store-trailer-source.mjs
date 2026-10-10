import {execFile} from 'node:child_process';
import {promisify} from 'node:util';
import {join} from 'node:path';
import {systemEnvironment} from './store-process.mjs';
const execute=promisify(execFile);
export function youtubeTrailer(value) {
  try {
    const url=new URL(value);
    if(url.protocol!=='https:' || url.username || url.password) return null;
    const id=url.hostname==='youtu.be' ? url.pathname.slice(1) :
      ['youtube.com','www.youtube.com'].includes(url.hostname) && url.pathname==='/watch' ? url.searchParams.get('v') : null;
    return /^[a-zA-Z0-9_-]{11}$/.test(id||'') ? `https://www.youtube.com/watch?v=${id}` : null;
  } catch { return null; }
}
export function remoteTrailer(item) {
  const trailers=item.RemoteTrailers||[];
  return [...trailers.filter(t=>/trailer/i.test(t.Name||'')),...trailers]
    .map(t=>youtubeTrailer(t.Url)).find(Boolean)||null;
}
export async function loadTrailers(api) {
  const result=[];
  for(let start=0;;start+=500) {
    const page=await api.json('/Items',{query:{UserId:api.account.userId,Recursive:true,
      IncludeItemTypes:'Movie,Series',Fields:'RemoteTrailers,LocalTrailerCount',StartIndex:start,Limit:500}});
    for(const item of page.Items||[]) {
      if(!/^[a-f0-9]{32}$/i.test(item.Id)) continue;
      let local=[];
      if(item.LocalTrailerCount>0) {
        try { local=await api.json(`/Items/${item.Id}/LocalTrailers`,{query:{UserId:api.account.userId}}); } catch {}
      }
      const trailer=local.find(t=>/^[a-f0-9]{32}$/i.test(t.Id));
      const url=remoteTrailer(item);
      if(trailer || url) result.push({name:item.Name,id:trailer?.Id,url:trailer ? null : url});
    }
    if(!page.Items?.length || start+page.Items.length>=page.TotalRecordCount) break;
  }
  for(let i=result.length-1;i>0;i--) {
    const j=Math.floor(Math.random()*(i+1));[result[i],result[j]]=[result[j],result[i]];
  }
  return result;
}
export async function trailerUrl(entry,base,signal) {
  const {stdout}=await execute('python3',[join(base,'native-tools/yt-dlp'),'--ignore-config',
    '--no-playlist','--js-runtimes',`node:${join(base,'runtime/bin/node')}`,'--remote-components','ejs:github',
    '--skip-download','-f','bestvideo[height<=360][vcodec^=avc1]/bestvideo[height<=360]/best[height<=360]',
    '--get-url',entry.url],{env:systemEnvironment(),timeout:70000,maxBuffer:16384,signal});
  const url=new URL(stdout.trim());
  if(url.protocol!=='https:' || !url.hostname.endsWith('.googlevideo.com') || url.username || url.password)
    throw new Error('Unexpected trailer stream destination');
  return url.href;
}
