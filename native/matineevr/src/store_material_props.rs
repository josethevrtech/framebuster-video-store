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
        Self::create(device,include_bytes!("../assets/crt-models.bin"),include_bytes!("../assets/crt-materials.rgba"),false)
    }
    pub fn jukebox(device: Rc<Graphics>) -> Result<Self> {
        Self::create(device,include_bytes!("../assets/olesk-jukebox/jukebox-model.bin"),
            include_bytes!("../assets/olesk-jukebox/jukebox-materials-v2.rgba"),true)
    }
    pub fn lounge(device: Rc<Graphics>) -> Result<Self> {
        Self::create(device,include_bytes!("../assets/lounge-models.bin"),include_bytes!("../assets/lounge-materials.rgba"),false)
    }
    pub fn consoles(device: Rc<Graphics>) -> Result<(Self,[u32;5])> {
        let source=include_bytes!("../assets/console-models-v2.bin");
        ensure!(&source[..8]==b"FBCONS01","Invalid console mesh header");
        let counts=std::array::from_fn(|i| u32::from_le_bytes(source[8+i*4..12+i*4].try_into().unwrap()));
        let mut mesh=b"FBPROP01".to_vec();
        mesh.extend(counts.iter().sum::<u32>().to_le_bytes()); mesh.extend(&source[28..]);
        Ok((Self::create_size(device,&mesh,include_bytes!("../assets/console-materials.rgba"),false,(3072,1536))?,counts))
    }
    pub fn rentals(device: Rc<Graphics>,mesh: &[u8]) -> Result<Self> {
        Self::create_size(device,mesh,include_bytes!("../assets/cartridge-materials.rgba"),false,(3072,1536))
    }
    fn create(device: Rc<Graphics>,data: &[u8],pixels: &[u8],jukebox: bool) -> Result<Self> {
        Self::create_size(device,data,pixels,jukebox,(3072,1024))
    }
    fn create_size(device: Rc<Graphics>,data: &[u8],pixels: &[u8],jukebox: bool,size: (u32,u32)) -> Result<Self> {
        ensure!(&data[..8]==b"FBPROP01","Invalid CRT asset header");
        let count=u32::from_le_bytes(data[8..12].try_into()?);
        ensure!(data.len()==12+count as usize*48,"Invalid CRT mesh length");
        let vertices=Buffer::new(device.clone(),data.len()-12)?;
        unsafe { std::ptr::copy_nonoverlapping(data[12..].as_ptr(),vertices.pointer,data.len()-12); }
        if jukebox {
            let values=unsafe { std::slice::from_raw_parts_mut(vertices.pointer as *mut f32,count as usize*12) };
            for vertex in values.chunks_exact_mut(12) {
                let p=crate::store_jukebox::place([vertex[0],vertex[1],vertex[2]]);
                let n=crate::store_jukebox::rotate([vertex[4],vertex[5],vertex[6]]);
                vertex[..3].copy_from_slice(&p); vertex[4..7].copy_from_slice(&n);
            }
        }
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
