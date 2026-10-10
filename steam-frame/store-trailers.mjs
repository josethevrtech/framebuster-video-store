import {spawn} from 'node:child_process';
import {writeFile} from 'node:fs/promises';
import {loadTrailers,trailerUrl} from './store-trailer-source.mjs';
import {localTrailer} from './store-trailer-local.mjs';
import {systemEnvironment} from './store-process.mjs';
export const FRAME_BYTES=512*384*4;
export function framePacket(pixels,sequence) {
  if(pixels.length!==FRAME_BYTES || !Number.isSafeInteger(sequence) || sequence<=0) throw new Error('Invalid TV frame');
  const stamp=Buffer.alloc(8);stamp.writeBigUInt64LE(BigInt(sequence));
  return Buffer.concat([stamp,pixels,stamp]);
}
export function createStoreTrailers(api,directory,base) {
  let closed=false,paused=false,player=null,index=0,sequence=0;
  let lastFrame=Date.now();
  const watchdog=setInterval(()=>{
    if(!paused && player && Date.now()-lastFrame>30000) player.kill('SIGTERM');
  },5000);
  watchdog.unref();
  const controller=new AbortController();
  const run=async()=>{
    const entries=await loadTrailers(api);
    console.log(`Store trailers: ${entries.length} matching library titles`);
    while(!closed && entries.length) {
      const entry=entries[index++%entries.length];
      let input=null;
      try {
        let source;
        if(entry.url) source=await trailerUrl(entry,base,controller.signal);
        else {
          input=await localTrailer(api,entry.id);source=input.url;
        }
        if(closed) { input?.close(); break; }
        const child=spawn('ffmpeg',['-hide_banner','-loglevel','error','-threads','1','-filter_threads','1','-re','-rw_timeout','15000000',
          '-i',source,'-t','240','-an','-vf','fps=12,scale=512:384:force_original_aspect_ratio=decrease,pad=512:384:(ow-iw)/2:(oh-ih)/2:color=black,vflip',
          '-f','rawvideo','-pix_fmt','rgba','pipe:1'],{env:systemEnvironment(),stdio:['pipe','pipe','ignore']});
        player=child;
        lastFrame=Date.now();
        const completion=new Promise(resolve=>{child.once('error',()=>resolve(-1));child.once('close',resolve);});
        child.stdin.end();
        if(paused) child.kill('SIGSTOP');
        console.log(`Store trailer: ${entry.name}`);
        let pixels=Buffer.alloc(FRAME_BYTES),offset=0;
        for await(const chunk of child.stdout) {
          let position=0;
          while(position<chunk.length && !closed) {
            const length=Math.min(FRAME_BYTES-offset,chunk.length-position);
            chunk.copy(pixels,offset,position,position+length);offset+=length;position+=length;
            if(offset===FRAME_BYTES) {
              lastFrame=Date.now();
              await writeFile(`${directory}/tv-frame.rgba`,framePacket(pixels,++sequence),{mode:0o600});
              await writeFile(`${directory}/tv-sequence`,String(sequence),{mode:0o600});offset=0;
            }
          }
          if(closed) break;
        }
        const code=await completion;
        if(code!==0 && !closed) console.error(`Trailer unavailable: ${entry.name}`);
      } catch {
        if(!closed) console.error(`Trailer unavailable: ${entry.name}`);
      } finally {
        input?.close();
        if(player) {player.kill('SIGCONT');player.kill('SIGTERM');player=null;}
      }
      if(!closed) await new Promise(resolve=>setTimeout(resolve,2000));
    }
  };
  void run().catch(()=>console.error('Store trailers unavailable; keeping poster displays'));
  return {
    setPaused(value) {paused=value;lastFrame=Date.now();if(player) player.kill(paused?'SIGSTOP':'SIGCONT');},
    close() {closed=true;clearInterval(watchdog);controller.abort();if(player) {player.kill('SIGCONT');player.kill('SIGTERM');}},
  };
}
