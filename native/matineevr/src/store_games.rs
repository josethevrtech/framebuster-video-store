use crate::{graphics::Graphics,store_catalog::Movie,store_material_props::MaterialProps,store_covers::StoreCovers};
use anyhow::{Result,ensure};
use ash::vk;
use std::rc::Rc;

pub struct StoreGames {
    bodies: Option<MaterialProps>,
    covers: StoreCovers,
    device: Rc<Graphics>,
}
impl StoreGames {
    pub fn new(device: Rc<Graphics>) -> Result<Self> {
        Ok(Self {bodies:None,covers:StoreCovers::new(device.clone())?,device})
    }
    pub fn update(&mut self,games: &[Movie],types: &[u8]) -> Result<()> {
        ensure!(games.len()==types.len() && types.len()<=54 && types.iter().all(|t| *t<6),"Invalid game shelf catalog");
        unsafe {self.device.api.device_wait_idle()?;}
        if types.is_empty() {self.bodies=None;return self.covers.update_mesh(games,Vec::new());}
        let (mesh,sizes)=crate::store_game_mesh::body(types);
        self.bodies=Some(MaterialProps::rentals(self.device.clone(),&mesh)?);
        self.covers.update_mesh(games,crate::store_game_mesh::labels(types,&sizes))
    }
    pub fn prepare(&mut self,command: vk::CommandBuffer) {
        if let Some(bodies)=self.bodies.as_mut() {bodies.prepare(command);}
        self.covers.prepare(command);
    }
    pub fn draw(&self,command: vk::CommandBuffer,p: [[f32;4];5]) {
        if let Some(bodies)=self.bodies.as_ref() {bodies.draw(command,p,&self.device);}
        self.covers.draw(command,p);
    }
}
