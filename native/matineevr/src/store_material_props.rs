use crate::{controller_pipeline::ControllerPipeline,graphics::Graphics,store_texture::StoreTexture};
use anyhow::{Result,ensure};
use ash::vk;
use matineevr::vk_memory::Buffer;
use std::rc::Rc;

pub struct MaterialProps {
    texture: StoreTexture,
    pipeline: ControllerPipeline,
    vertices: Buffer,
    count: u32,
}
impl MaterialProps {
    pub fn new(device: Rc<Graphics>) -> Result<Self> {
        Self::create(device,include_bytes!("../assets/crt-models.bin"),include_bytes!("../assets/crt-materials.rgba"))
    }
    pub fn lounge(device: Rc<Graphics>) -> Result<Self> {
        Self::create(device,include_bytes!("../assets/lounge-models.bin"),include_bytes!("../assets/lounge-materials.rgba"))
    }
    pub fn retail(device: Rc<Graphics>) -> Result<Self> {
        Self::create_size(device,include_bytes!("../assets/retail-models.bin"),include_bytes!("../assets/retail-materials.rgba"),(6144,2048))
    }
    pub fn consoles(device: Rc<Graphics>) -> Result<(Self,[u32;5])> {
        let source=include_bytes!("../assets/console-models-v2.bin");
        ensure!(&source[..8]==b"FBCONS01","Invalid console mesh header");
        let counts=std::array::from_fn(|i| u32::from_le_bytes(source[8+i*4..12+i*4].try_into().unwrap()));
        let mut mesh=b"FBPROP01".to_vec();
        mesh.extend(counts.iter().sum::<u32>().to_le_bytes()); mesh.extend(&source[28..]);
        Ok((Self::create_size(device,&mesh,include_bytes!("../assets/console-materials.rgba"),(3072,1536))?,counts))
    }
    fn create(device: Rc<Graphics>,data: &[u8],pixels: &[u8]) -> Result<Self> {
        Self::create_size(device,data,pixels,(3072,1024))
    }
    fn create_size(device: Rc<Graphics>,data: &[u8],pixels: &[u8],size: (u32,u32)) -> Result<Self> {
        ensure!(&data[..8]==b"FBPROP01","Invalid CRT asset header");
        let count=u32::from_le_bytes(data[8..12].try_into()?);
        ensure!(data.len()==12+count as usize*48,"Invalid CRT mesh length");
        let vertices=Buffer::new(device.clone(),data.len()-12)?;
        unsafe { std::ptr::copy_nonoverlapping(data[12..].as_ptr(),vertices.pointer,data.len()-12); }
        let mut texture=StoreTexture::with_format(device.clone(),size,vk::Format::R8G8B8A8_UNORM)?;
        texture.upload(pixels)?;
        Ok(Self { pipeline:ControllerPipeline::material(device,texture.layout)?,texture,vertices,count })
    }
    pub fn prepare(&mut self,command: vk::CommandBuffer) { self.texture.prepare(command); }
    pub fn draw(&self,command: vk::CommandBuffer,parameters: [[f32;4];5],device: &Graphics) {
        self.draw_range(command,parameters,device,0,self.count);
    }
    pub fn draw_range(&self,command: vk::CommandBuffer,parameters: [[f32;4];5],device: &Graphics,first: u32,count: u32) {
        unsafe {
            let d=&device.api;
            d.cmd_bind_pipeline(command,vk::PipelineBindPoint::GRAPHICS,self.pipeline.handle);
            d.cmd_bind_descriptor_sets(command,vk::PipelineBindPoint::GRAPHICS,self.pipeline.layout,0,&[self.texture.descriptor],&[]);
            let bytes=std::slice::from_raw_parts(parameters.as_ptr() as *const u8,80);
            d.cmd_push_constants(command,self.pipeline.layout,vk::ShaderStageFlags::VERTEX,0,bytes);
            d.cmd_bind_vertex_buffers(command,0,&[self.vertices.handle],&[0]);
            d.cmd_draw(command,count,1,first,0);
        }
    }
}
