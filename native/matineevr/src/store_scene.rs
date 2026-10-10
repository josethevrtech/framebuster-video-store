use crate::{graphics::Graphics, input::Controls, store_geometry::{self, Vertex}};
use anyhow::{Result, ensure};
use matineevr::{renderer::IN_FLIGHT, vk_memory::Buffer};
use openxr as xr;
use std::rc::Rc;

pub struct StoreScene {
    pub active: bool,
    pub room: Buffer,
    pub count: u32,
    pub frames: Vec<Buffer>,
    dynamic: Vec<Vertex>,
    held: [bool; 2],
    selected: Option<usize>,
    pub scale: f32,
    calibrated: bool,
    pub navigation: crate::store_motion::StoreNavigation,
    pub covers: crate::store_covers::StoreCovers,
    catalog_count: usize,
    pub music_cover: crate::store_music_cover::MusicCover,
    pub material_props: crate::store_material_props::MaterialProps,
    pub albums: crate::store_albums::StoreAlbums,
    pub lounge_props: crate::store_material_props::MaterialProps,
    pub retail_props: crate::store_material_props::MaterialProps,
    pub console: crate::store_console::StoreConsole,
    pub games: crate::store_games::StoreGames,
    pub trailer: crate::store_trailer::StoreTrailer,
    pub music_action: Option<&'static str>,
    pub album_action: Option<usize>,
    pub audio: [f32;2],
}

impl StoreScene {
    pub fn new(device: Rc<Graphics>) -> Result<Self> {
        let mut vertices = store_geometry::room();
        crate::store_scale::vertices(&mut vertices, crate::store_scale::SCALE);
        let room = Buffer::new(device.clone(), vertices.len() * 48)?;
        upload(&room, &vertices)?;
        Ok(Self { active: true, room, count: vertices.len() as u32,
            frames: (0..IN_FLIGHT).map(|_| Buffer::new(device.clone(), 64 * 1024)).collect::<Result<_>>()?,
            dynamic: Vec::new(), held: [false; 2], selected: None, navigation: Default::default(),
            scale: crate::store_scale::SCALE, calibrated: false,
            covers: crate::store_covers::StoreCovers::new(device.clone())?, catalog_count: 54,
            music_cover: crate::store_music_cover::MusicCover::new(device.clone())?,
            material_props: crate::store_material_props::MaterialProps::new(device.clone())?,
            albums: crate::store_albums::StoreAlbums::new(device.clone())?,
            lounge_props: crate::store_material_props::MaterialProps::lounge(device.clone())?,
            retail_props: crate::store_material_props::MaterialProps::retail(device.clone())?,
            console: crate::store_console::StoreConsole::new(device.clone())?,
            games: crate::store_games::StoreGames::new(device.clone())?,
            trailer: crate::store_trailer::StoreTrailer::new(device)?,music_action:None,album_action:None,audio:[0.0;2] })
    }

    pub fn set_catalog(&mut self, movies: &[crate::store_catalog::Movie]) -> Result<()> {
        self.covers.update(movies)?;
        self.catalog_count = movies.len(); self.selected = None;
        Ok(())
    }

    pub fn calibrate(&mut self, head_y: f32) -> Result<()> {
        if self.calibrated { return Ok(()); }
        if !head_y.is_finite() { return Ok(()); }
        self.navigation.pose.position.y = crate::store_scale::eye_offset(head_y);
        self.calibrated = true;
        eprintln!("FrameBuster eye origin: {head_y:.3}m; floor: {:.3}m; physical scale: 1.0", head_y - 1.65);
        Ok(())
    }

    pub fn update(&mut self, active: bool, controls: Controls, aims: [Option<xr::Posef>; 2],
        hands: [Option<xr::Posef>; 2], head: Option<xr::Posef>, dt: f32) -> Option<usize> {
        self.active = active;
        self.dynamic.clear();
        let mut launch = false;
        if active && let Some(head) = head {
            self.navigation.update(controls, head, hands, dt);
            let p=head.position; let q=head.orientation;
            let right=[1.0-2.0*(q.y*q.y+q.z*q.z),2.0*(q.x*q.y+q.w*q.z),2.0*(q.x*q.z-q.w*q.y)];
            let (listener,right)=self.navigation.inverse_ray([p.x,p.y,p.z],right);
            self.audio=crate::store_audio::mix(listener,right,true);
        } else { self.navigation.reset(); self.audio=[0.0;2]; }
        for (hand, pose) in aims.into_iter().enumerate() {
            let pressed = controls.triggers[hand] > 0.65;
            if active && let Some(pose) = pose {
                let (origin, direction) = ray(pose);
                let (origin, direction) = self.navigation.inverse_ray(origin, direction);
                let (origin, direction) = crate::store_scale::inverse(origin, direction, self.scale);
                let hit = (0..crate::store_display::BAYS.len()*18)
                    .filter(|i| crate::store_display::catalog_index(*i,self.catalog_count).is_some()).filter_map(|i|
                    crate::store_display::hit(origin, direction, i).map(|t| (i, t)))
                    .min_by(|a, b| a.1.total_cmp(&b.1));
                let music=crate::store_album_racks::controls(origin,direction)
                    .filter(|m| hit.is_none_or(|h| m.1<h.1));
                let console=crate::store_console::hit(origin,direction)
                    .filter(|m| hit.is_none_or(|h| m.1<h.1) && music.is_none_or(|h| m.1<h.1));
                let album=self.albums.hit(origin,direction).filter(|m| hit.is_none_or(|h| m.1<h.1)
                    && music.is_none_or(|h| m.1<h.1) && console.is_none_or(|h| m.1<h.1));
                let color = if hit.is_some() || music.is_some() || console.is_some() || album.is_some() { [1.0, 0.65, 0.12] } else { [0.25, 0.65, 1.0] };
                store_geometry::beam(&mut self.dynamic, origin, direction,
                    album.map_or_else(|| console.map_or_else(|| music.map_or_else(|| hit.map_or(4.0,|h| h.1),|m| m.1),|m| m.1),|m| m.1), color);
                if let Some((index,_))=album {
                    if pressed && !self.held[hand] {self.album_action=Some(index);self.selected=None;}
                } else if let Some((index,_))=console {
                    if pressed && !self.held[hand] { self.console.selected=index; self.selected=None; }
                } else if let Some((action,_))=music {
                    if pressed && !self.held[hand] { self.music_action=Some(action); self.selected=None; }
                } else if let Some((i, _)) = hit {
                    crate::store_display::outline(&mut self.dynamic, i, color);
                    if pressed && !self.held[hand] { self.selected = Some(i); launch = true; }
                }
            }
            self.held[hand] = pressed;
        }
        if active && let Some(i) = self.selected { crate::store_display::outline(&mut self.dynamic, i, [0.2, 1.0, 0.4]); }
        if active { self.console.lights(&mut self.dynamic); }
        crate::store_scale::vertices(&mut self.dynamic, self.scale);
        if active && (launch || controls.a) {
            self.selected.and_then(|i| crate::store_display::catalog_index(i,self.catalog_count))
        } else { None }
    }

    pub fn upload(&self, slot: usize) -> Result<u32> {
        upload(&self.frames[slot], &self.dynamic)?;
        Ok(self.dynamic.len() as u32)
    }
}

fn upload(buffer: &Buffer, vertices: &[Vertex]) -> Result<()> {
    let bytes = std::mem::size_of_val(vertices);
    ensure!(bytes <= buffer.size, "Store geometry exceeds buffer capacity");
    unsafe { std::ptr::copy_nonoverlapping(vertices.as_ptr() as *const u8, buffer.pointer, bytes); }
    Ok(())
}

fn ray(p: xr::Posef) -> ([f32; 3], [f32; 3]) {
    let q = p.orientation;
    ([p.position.x, p.position.y, p.position.z],
        [-2.0 * (q.x * q.z + q.w * q.y), 2.0 * (q.w * q.x - q.y * q.z),
            -1.0 + 2.0 * (q.x * q.x + q.y * q.y)])
}
