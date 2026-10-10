use crate::{controller_pipeline::ControllerPipeline,graphics::Graphics,store_texture::StoreTexture};
use anyhow::{Result,ensure};
use ash::vk;
use matineevr::vk_memory::Buffer;
use std::{path::Path,rc::Rc};

pub struct MusicCover {
    texture: StoreTexture,
    pipeline: ControllerPipeline,
    vertices: Buffer,
    visible: bool,
    device: Rc<Graphics>,
}
impl MusicCover {
    pub fn new(device: Rc<Graphics>) -> Result<Self> {
        let texture=StoreTexture::with_size(device.clone(),(384,384))?;
        let vertices=Buffer::new(device.clone(),6*48)?;
        let p=crate::store_jukebox::COVER;
        let points: [crate::store_geometry::Vertex;4]=std::array::from_fn(|i| {
            [[p[0],p[1]+[-0.16,-0.16,0.16,0.16][i],p[2]-[-0.16,0.16,0.16,-0.16][i],1.0],
                [if i==1 || i==2 { 1.0 } else { 0.0 },if i<2 { 1.0 } else { 0.0 },0.0,0.0],[1.0;4]]
        });
        let mesh=[0,1,2,0,2,3].map(|i| points[i]);
        unsafe { std::ptr::copy_nonoverlapping(mesh.as_ptr() as *const u8,vertices.pointer,6*48); }
        Ok(Self { pipeline:ControllerPipeline::textured(device.clone(),texture.layout)?,texture,
            vertices,visible:false,device })
    }
    pub fn update(&mut self,path: &Path) -> Result<()> {
        let bytes=std::fs::read(path)?;
        ensure!(bytes.len()==384*384*4,"Invalid jukebox cover dimensions");
        unsafe { self.device.api.device_wait_idle()?; }
        self.texture.upload(&bytes)?; self.visible=true;
        Ok(())
    }
    pub fn prepare(&mut self,command: vk::CommandBuffer) {
        if self.visible { self.texture.prepare(command); }
    }
    pub fn draw(&self,command: vk::CommandBuffer,parameters: [[f32;4];5]) {
        if !self.visible { return; }
        unsafe {
            let d=&self.device.api;
            d.cmd_bind_pipeline(command,vk::PipelineBindPoint::GRAPHICS,self.pipeline.handle);
            d.cmd_bind_descriptor_sets(command,vk::PipelineBindPoint::GRAPHICS,self.pipeline.layout,0,&[self.texture.descriptor],&[]);
            let bytes=std::slice::from_raw_parts(parameters.as_ptr() as *const u8,80);
            d.cmd_push_constants(command,self.pipeline.layout,vk::ShaderStageFlags::VERTEX,0,bytes);
            d.cmd_bind_vertex_buffers(command,0,&[self.vertices.handle],&[0]);
            d.cmd_draw(command,6,1,0,0);
        }
    }
}
