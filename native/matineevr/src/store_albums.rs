use crate::{graphics::Graphics,controller_pipeline::ControllerPipeline,store_covers::StoreCovers,store_album_catalog::AlbumBay};
use anyhow::{Result,ensure};
use ash::vk;
use matineevr::vk_memory::Buffer;
use std::{collections::VecDeque,path::Path,rc::Rc};
const BODY_BYTES: usize=crate::store_album_racks::MAX_BAYS*(14+54*2)*36*48;
pub struct StoreAlbums {
    bodies: Buffer,count: u32,pipeline: ControllerPipeline,signs: StoreCovers,
    covers: Vec<(usize,StoreCovers)>,pending: VecDeque<AlbumBay>,bays: Vec<AlbumBay>,device: Rc<Graphics>,
}
impl StoreAlbums {
    pub fn new(device: Rc<Graphics>) -> Result<Self> {
        Ok(Self {bodies:Buffer::new(device.clone(),BODY_BYTES)?,count:0,pipeline:ControllerPipeline::new(device.clone())?,
            signs:StoreCovers::panel(device.clone(),include_bytes!("../assets/music-signs.rgba"),(1024,crate::store_album_signs::HEIGHT),crate::store_album_signs::mesh(&[]))?,
            covers:Vec::new(),pending:VecDeque::new(),bays:Vec::new(),device})
    }
    pub fn begin(&mut self,path: &Path) -> Result<()> {
        let bays=crate::store_album_catalog::read(path)?;
        ensure!(bays.len()>=self.bays.len(),"Album catalog unexpectedly shrank");
        for (a,b) in self.bays.iter().zip(&bays) {ensure!(a.bay==b.bay && a.genre==b.genre && a.count==b.count && a.first==b.first && a.path==b.path,"Album section changed identity");}
        let mut vertices=Vec::new();for b in &bays {crate::store_album_racks::rack(&mut vertices,b.bay,b.count);}
        let size=std::mem::size_of_val(vertices.as_slice());ensure!(size<=self.bodies.size,"Music geometry exceeds capacity");
        unsafe {self.device.api.device_wait_idle()?;std::ptr::copy_nonoverlapping(vertices.as_ptr() as *const u8,self.bodies.pointer,size);}
        self.count=vertices.len() as u32;self.signs.set_mesh(crate::store_album_signs::mesh(&bays))?;
        self.pending.extend(bays[self.bays.len()..].iter().cloned());
        crate::store_album_racks::activate(bays.len());self.bays=bays;Ok(())
    }
    pub fn poll(&mut self) -> Result<()> {
        let Some(b)=self.pending.pop_front() else {return Ok(());};
        let items=crate::store_catalog::read(&b.path)?;ensure!(items.len()==b.count,"Album section count mismatch");
        let mut covers=StoreCovers::new(self.device.clone())?;covers.update_mesh(&items,crate::store_album_racks::labels(b.bay,items.len()))?;
        self.covers.push((b.bay,covers));eprintln!("Music shelf loaded: bay {}, {} albums",b.bay,items.len());Ok(())
    }
    pub fn hit(&self,p: [f32;3],d: [f32;3]) -> Option<(usize,f32)> {
        self.bays.iter().filter(|b| self.covers.iter().any(|(bay,_)| *bay==b.bay)).flat_map(|b| (0..b.count)
            .filter_map(move |i| crate::store_album_racks::hit(p,d,b.bay,i).map(|t| (b.first+i,t))))
            .min_by(|a,b| a.1.total_cmp(&b.1))
    }
    pub fn prepare(&mut self,command: vk::CommandBuffer) {self.signs.prepare(command);for (_,covers) in &mut self.covers {covers.prepare(command);}}
    pub fn draw(&self,command: vk::CommandBuffer,p: [[f32;4];5]) {
        unsafe {
            let d=&self.device.api;d.cmd_bind_pipeline(command,vk::PipelineBindPoint::GRAPHICS,self.pipeline.handle);
            let bytes=std::slice::from_raw_parts(p.as_ptr() as *const u8,80);
            d.cmd_push_constants(command,self.pipeline.layout,vk::ShaderStageFlags::VERTEX,0,bytes);
            d.cmd_bind_vertex_buffers(command,0,&[self.bodies.handle],&[0]);d.cmd_draw(command,self.count,1,0,0);
        }
        self.signs.draw(command,p);for (_,covers) in &self.covers {covers.draw(command,p);}
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn full_music_collection_fits_the_geometry_buffer() {
        let mut vertices=Vec::new();
        for bay in 0..crate::store_album_racks::MAX_BAYS {crate::store_album_racks::rack(&mut vertices,bay,54);}
        assert_eq!(vertices.len()*48,super::BODY_BYTES);
    }
}
