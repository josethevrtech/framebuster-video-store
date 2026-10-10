import {spawn} from 'node:child_process';
import {systemEnvironment} from './store-process.mjs';
export function albumFallback() {
  const pixels=Buffer.alloc(192*288*4);
  for(let y=0;y<288;y++)for(let x=0;x<192;x++) {
    const d=Math.hypot(x-96,y-144),color=d<12?[20,20,25]:d<80?[155+Math.round(x/4),165,185]:[25,25,32];
    pixels.set([...color,255],(y*192+x)*4);
  }
  return pixels;
}
export function decodeAlbumArt(bytes) {
  return new Promise((resolve,reject)=>{
    const child=spawn('ffmpeg',['-hide_banner','-loglevel','error','-threads','1','-filter_threads','1','-i','pipe:0','-vf','scale=192:192:force_original_aspect_ratio=decrease,pad=192:192:(ow-iw)/2:(oh-ih)/2:color=black,pad=192:288:0:48:color=black,vflip','-frames:v','1','-f','rawvideo','-pix_fmt','rgba','pipe:1'],{stdio:['pipe','pipe','ignore'],env:systemEnvironment()});
    const chunks=[];let size=0;
    child.stdout.on('data',bytes=>{size+=bytes.length;if(size>192*288*4)child.kill();else chunks.push(bytes);});
    child.stdin.on('error',()=>{});child.once('error',reject);
    const timer=setTimeout(()=>child.kill(),10000);
    child.once('close',code=>{clearTimeout(timer);code===0&&size===192*288*4?resolve(Buffer.concat(chunks)):reject(new Error('Album art decode failed'));});child.stdin.end(bytes);
  });
}
