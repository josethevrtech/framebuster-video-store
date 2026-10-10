import test from 'node:test';
import assert from 'node:assert/strict';
import {youtubeTrailer,remoteTrailer} from './store-trailer-source.mjs';
import {FRAME_BYTES,framePacket} from './store-trailers.mjs';
import {localTrailer} from './store-trailer-local.mjs';
test('trailers use exact YouTube identities and prefer named trailers',()=>{
  assert.equal(youtubeTrailer('https://youtu.be/N1VlDVRiFrk'),'https://www.youtube.com/watch?v=N1VlDVRiFrk');
  for(const url of ['http://youtube.com/watch?v=N1VlDVRiFrk','https://youtube.com.evil.test/watch?v=N1VlDVRiFrk','https://example.com/file.mp4','https://youtube.com/watch?v=wrong']) assert.equal(youtubeTrailer(url),null);
  assert.equal(remoteTrailer({RemoteTrailers:[{Url:'https://youtu.be/loTIzXAS7v4'},{Name:'Official Trailer',Url:'https://youtu.be/N1VlDVRiFrk'}]}),'https://www.youtube.com/watch?v=N1VlDVRiFrk');
});
test('TV frames publish matching generation stamps and bounded dimensions',()=>{
  const packet=framePacket(Buffer.alloc(FRAME_BYTES,42),9);
  assert.equal(packet.length,FRAME_BYTES+16);
  assert.equal(packet.readBigUInt64LE(),9n);assert.equal(packet.readBigUInt64LE(packet.length-8),9n);
  assert.throws(()=>framePacket(Buffer.alloc(10),9));assert.throws(()=>framePacket(Buffer.alloc(FRAME_BYTES),0));
});
test('local trailers keep authentication in the relay and preserve byte-range seeking',async()=>{
  const id='a'.repeat(32),token='private-test-token';
  let observed;
  const relay=await localTrailer({server:new URL('http://media.test:8096'),account:{token},
    fetch:async(url,options)=>{observed={url,options};return new Response('part',
      {status:206,headers:{'Content-Range':'bytes 10-13/100','Content-Length':'4'}});}},id);
  try {
    assert(!relay.url.includes(token));
    const response=await fetch(relay.url,{headers:{Range:'bytes=10-13'}});
    assert.equal(response.status,206);assert.equal(await response.text(),'part');
    assert.equal(observed.options.headers.Range,'bytes=10-13');
    assert.equal(observed.options.headers['X-Emby-Token'],token);
    assert(!observed.url.includes(token));
    assert.equal((await fetch(relay.url,{headers:{Origin:'https://elsewhere.test'}})).status,403);
  } finally {relay.close();}
});
