use crate::{controller_pipeline::ControllerPipeline,graphics::Graphics,store_catalog::Movie,store_texture::StoreTexture};
use anyhow::{Result,ensure};
use ash::vk;
use matineevr::vk_memory::Buffer;
use std::rc::Rc;

pub struct StoreCovers {
    pipeline: ControllerPipeline,
    texture: StoreTexture,
    vertices: Buffer,
    count: u32,
    device: Rc<Graphics>,
}
impl StoreCovers {
    pub fn new(device: Rc<Graphics>) -> Result<Self> {
        let texture = StoreTexture::new(device.clone())?;
        Self::with_texture(device,texture)
    }
    fn with_texture(device: Rc<Graphics>,texture: StoreTexture) -> Result<Self> {
        Ok(Self { pipeline:ControllerPipeline::textured(device.clone(),texture.layout)?,texture,
            vertices:Buffer::new(device.clone(),crate::store_cover_mesh::mesh(54).len()*48)?,count:0,device })
    }
    pub fn panel(device: Rc<Graphics>,pixels: &[u8],size: (u32,u32),vertices: Vec<crate::store_geometry::Vertex>) -> Result<Self> {
        let mut texture=StoreTexture::with_size(device.clone(),size)?;
        texture.upload(pixels)?;
        let mut result=Self::with_texture(device,texture)?;
        result.set_mesh(vertices)?;
        Ok(result)
    }
    pub fn update(&mut self,movies: &[Movie]) -> Result<()> {
        self.update_mesh(movies,crate::store_cover_mesh::mesh(movies.len()))
    }
    pub fn update_mesh(&mut self,movies: &[Movie],vertices: Vec<crate::store_geometry::Vertex>) -> Result<()> {
        unsafe { self.device.api.device_wait_idle()?; }
        let canvas = crate::store_poster::atlas(movies);
        self.texture.upload(&canvas.pixels)?;
        self.set_mesh(vertices)
    }
    pub(crate) fn set_mesh(&mut self,vertices: Vec<crate::store_geometry::Vertex>) -> Result<()> {
        let size = std::mem::size_of_val(vertices.as_slice());
        ensure!(size <= self.vertices.size,"Cover geometry exceeds capacity");
        unsafe { std::ptr::copy_nonoverlapping(vertices.as_ptr() as *const u8,self.vertices.pointer,size); }
        self.count = vertices.len() as u32;
        Ok(())
    }
    pub fn prepare(&mut self,command: vk::CommandBuffer) { if self.count > 0 { self.texture.prepare(command); } }
    pub fn draw(&self,command: vk::CommandBuffer,parameters: [[f32;4];5]) {
        if self.count == 0 { return; }
        unsafe {
            let d = &self.device.api;
            d.cmd_bind_pipeline(command,vk::PipelineBindPoint::GRAPHICS,self.pipeline.handle);
            d.cmd_bind_descriptor_sets(command,vk::PipelineBindPoint::GRAPHICS,self.pipeline.layout,0,&[self.texture.descriptor],&[]);
            let bytes = std::slice::from_raw_parts(parameters.as_ptr() as *const u8,80);
            d.cmd_push_constants(command,self.pipeline.layout,vk::ShaderStageFlags::VERTEX,0,bytes);
            d.cmd_bind_vertex_buffers(command,0,&[self.vertices.handle],&[0]);
            d.cmd_draw(command,self.count,1,0,0);
        }
    }
}
