use crate::{graphics::Graphics,controller_pipeline::ControllerPipeline,store_covers::StoreCovers,
    store_game_catalog::GameBay};
use anyhow::{Result,ensure};
use ash::vk;
use matineevr::vk_memory::Buffer;
use std::{collections::VecDeque,path::Path,rc::Rc};

pub struct StoreGames {
    bodies: Buffer,
    count: u32,
    pipeline: ControllerPipeline,
    covers: Vec<StoreCovers>,
    signs: StoreCovers,
    pending: VecDeque<GameBay>,
    device: Rc<Graphics>,
}
impl StoreGames {
    pub fn new(device: Rc<Graphics>) -> Result<Self> {
        Ok(Self {bodies:Buffer::new(device.clone(),32*54*36*48)?,count:0,
            pipeline:ControllerPipeline::new(device.clone())?,covers:Vec::new(),
            signs:Self::signs(device.clone(),&[])?,pending:VecDeque::new(),device})
    }
    fn signs(device: Rc<Graphics>,bays: &[GameBay]) -> Result<StoreCovers> {
        StoreCovers::panel(device,include_bytes!("../assets/video-games-signs.rgba"),(1024,crate::store_game_signs::HEIGHT),
            crate::store_game_signs::mesh(bays))
    }
    pub fn begin(&mut self,path: &Path) -> Result<()> {
        let bays=crate::store_game_catalog::read(path)?;
        let vertices: Vec<_>=bays.iter().flat_map(|s| crate::store_game_mesh::boxes(s.bay,s.count)).collect();
        let size=std::mem::size_of_val(vertices.as_slice());
        ensure!(size<=self.bodies.size,"Game case geometry exceeds capacity");
        unsafe {
            self.device.api.device_wait_idle()?;
            std::ptr::copy_nonoverlapping(vertices.as_ptr() as *const u8,self.bodies.pointer,size);
        }
        self.count=vertices.len() as u32;
        self.signs=Self::signs(self.device.clone(),&bays)?;
        self.covers.clear();self.pending=bays.into();
        Ok(())
    }
    pub fn poll(&mut self) -> Result<()> {
        let Some(section)=self.pending.pop_front() else {return Ok(());};
        let games=crate::store_catalog::read(&section.path)?;
        ensure!(games.len()==section.count,"Game shelf count does not match its catalog");
        let mut covers=StoreCovers::new(self.device.clone())?;
        covers.update_mesh(&games,crate::store_game_mesh::labels(section.bay,games.len()))?;
        self.covers.push(covers);
        if self.pending.is_empty() {eprintln!("Game shelves: all {} console bays loaded",self.covers.len());}
        Ok(())
    }
    pub fn prepare(&mut self,command: vk::CommandBuffer) {
        self.signs.prepare(command);
        for covers in &mut self.covers {covers.prepare(command);}
    }
    pub fn draw(&self,command: vk::CommandBuffer,p: [[f32;4];5]) {
        unsafe {
            let d=&self.device.api;
            d.cmd_bind_pipeline(command,vk::PipelineBindPoint::GRAPHICS,self.pipeline.handle);
            let bytes=std::slice::from_raw_parts(p.as_ptr() as *const u8,80);
            d.cmd_push_constants(command,self.pipeline.layout,vk::ShaderStageFlags::VERTEX,0,bytes);
            d.cmd_bind_vertex_buffers(command,0,&[self.bodies.handle],&[0]);
            d.cmd_draw(command,self.count,1,0,0);
        }
        self.signs.draw(command,p);
        for covers in &self.covers {covers.draw(command,p);}
    }
}
