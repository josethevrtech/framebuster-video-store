import {randomBytes} from 'node:crypto';
import {createServer} from 'node:http';
import {relayMedia} from './native-media.mjs';
import {relayPath} from './native-policy.mjs';

export async function localTrailer(api,itemId) {
  if(!/^[a-f0-9]{32}$/i.test(itemId)) throw new Error('Invalid local trailer identity');
  const source=new URL(api.server.href.replace(/\/+$/,'')+`/Videos/${itemId}/stream`);
  source.searchParams.set('Static','true');
  const session={id:randomBytes(16).toString('hex'),itemId,source:source.href,token:api.account.token};
  const path=relayPath(session.id,source.href);
  const server=createServer(async(req,res)=>{
    if(req.headers.host!==`127.0.0.1:${server.address().port}` || req.headers.origin || req.url!==path
      || !['GET','HEAD'].includes(req.method)) { res.writeHead(403);res.end();return; }
    await relayMedia(req,res,new URL(req.url,`http://${req.headers.host}`),session,api.server.href,api.fetch);
  });
  await new Promise((resolve,reject)=>{server.once('error',reject);server.listen(0,'127.0.0.1',resolve);});
  return {url:`http://127.0.0.1:${server.address().port}${path}`,
    close() {server.closeAllConnections();server.close();}};
}
