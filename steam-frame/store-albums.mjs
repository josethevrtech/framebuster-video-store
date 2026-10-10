import {readFile,writeFile,mkdir} from 'node:fs/promises';
import {join} from 'node:path';
import genres from './music-genres.json' with {type:'json'};
import {decodeAlbumArt,albumFallback} from './store-album-art.mjs';
export {genres};
export function genreOf(item) {
  const value=(item.Genres?.[0]||'').toLowerCase();
  if(/alternative|alternatif|indie/.test(value))return 0;
  if(/reggaeton/.test(value))return 5;
  if(/salsa/.test(value))return 4;
  if(/latin pop/.test(value))return 3;
  if(/latin|latine|латино|banda|monde|merengue|bachata/.test(value))return 6;
  if(/hip.?hop|rap/.test(value))return 7;
  if(/soundtrack|anime|film|игры|саундтреки|фильмы/.test(value))return 9;
  if(/j.?pop/.test(value))return 8;
  if(/rock/.test(value))return 1;
  if(/electron|électron|electro|house|dance|techno|танцев/.test(value))return 10;
  if(/soul|funk|r&b/.test(value))return 11;
  if(/reggae/.test(value))return 12;
  if(/country/.test(value))return 13;
  if(/classical/.test(value))return 14;
  if(/pop|поп/.test(value))return 2;
  return 15;
}
export function arrangeAlbums(items) {
  const unique=new Map(items.filter(i=>i.Type==='MusicAlbum'&&/^[a-f0-9]{32}$/i.test(i.Id)).map(i=>[i.Id.toLowerCase(),i]));
  const bays=[];let first=0;
  for(let genre=0;genre<genres.length;genre++) {
    const albums=[...unique.values()].filter(i=>genreOf(i)===genre).sort((a,b)=>String(a.Name).localeCompare(String(b.Name)));
    for(let start=0;start<albums.length;start+=54) {
      const items=albums.slice(start,start+54);bays.push({bay:bays.length,genre,first,items});first+=items.length;
    }
  }
  if(bays.length>24)throw new Error('Album collection exceeds the physical music area');
  return bays;
}
export async function loadAlbums(api) {
  const items=[];
  for(let start=0;;start+=500) {
    const page=await api.json('/Items',{query:{UserId:api.account.userId,Recursive:true,IncludeItemTypes:'MusicAlbum',Fields:'Genres,ProductionYear,AlbumArtist,Artists',StartIndex:start,Limit:500,SortBy:'SortName',SortOrder:'Ascending'}});
    items.push(...(page.Items||[]));if(!page.Items?.length||start+page.Items.length>=page.TotalRecordCount)break;
  }
  return items;
}
function number(value){const bytes=Buffer.alloc(4);bytes.writeUInt32LE(value);return bytes;}
function text(value){const bytes=Buffer.from(String(value||'').slice(0,240));return Buffer.concat([number(bytes.length),bytes]);}
export async function createAlbumShelves(api,base,directory,active=()=>true) {
  const bays=arrangeAlbums(await loadAlbums(api));const albums=bays.flatMap(b=>b.items);
  return {albums,ready:publishShelves(api,base,directory,bays,active)};
}
async function publishShelves(api,base,directory,bays,active) {
  const cache=join(base,'native-store','album-art');await mkdir(cache,{recursive:true,mode:0o700});
  const manifest=[];
  for(const {bay,genre,first,items} of bays) {
    if(!active())return;
    const records=[];
    for(let start=0;start<items.length;start+=4) records.push(...await Promise.all(items.slice(start,start+4).map(async item=> {
      const id=item.Id.toLowerCase(),path=join(cache,`${id}.rgba`);let pixels;
      try{pixels=await readFile(path);if(pixels.length!==192*288*4)throw new Error('Invalid cached album art');}
      catch{
        try{
          const response=await api.request(`/Items/${id}/Images/Primary`,{query:{MaxWidth:192,MaxHeight:192,Format:'jpg'}});
          const bytes=Buffer.from(await response.arrayBuffer());if(bytes.length>8*1024*1024)throw new Error('Album image exceeds capacity');
          pixels=await decodeAlbumArt(bytes);await writeFile(path,pixels,{mode:0o600,flag:'wx'}).catch(error=>{if(error.code!=='EEXIST')throw error;});
        }catch{pixels=albumFallback();}
      }
      return Buffer.concat([Buffer.from(id),text(item.Name),number(0),number(192),number(288),pixels]);
    })));
    const name=`albums-bay-${bay}.bin`;
    await writeFile(join(directory,name),Buffer.concat([Buffer.from('FBCAT001'),number(items.length),number(0),number(items.length),...records]),{mode:0o600});
    manifest.push(`${bay}\t${genre}\t${items.length}\t${first}\t${name}`);
    const path=join(directory,`album-shelves-${bay}.manifest`);
    await writeFile(path,manifest.join('\n'),{mode:0o600});await writeFile(join(directory,'album-catalog-path'),path,{mode:0o600});
    console.log(`Music artwork ready: section ${bay+1}/${bays.length}, ${items.length} albums`);
  }
  console.log(`Music shelves: ${bays.flatMap(b=>b.items).length} albums in ${bays.length} genre sections`);
}
