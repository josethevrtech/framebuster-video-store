use crate::{controller_pipeline::ControllerPipeline,graphics::Graphics,store_texture::StoreTexture};
use anyhow::Result;
use ash::vk;
use matineevr::{renderer::IN_FLIGHT,vk_memory::Buffer};
use std::{io::Read,path::Path,rc::Rc,time::{Instant,Duration}};

const BYTES: usize=512*384*4;
pub struct StoreTrailer {
    textures: Vec<StoreTexture>,
    pipeline: ControllerPipeline,
    vertices: Buffer,
    frame: Vec<u8>,
    revision: u64,
    uploaded: Vec<u64>,
    next: Instant,
    device: Rc<Graphics>,
}
impl StoreTrailer {
    pub fn new(device: Rc<Graphics>) -> Result<Self> {
        let textures=(0..IN_FLIGHT).map(|_| StoreTexture::with_size(device.clone(),(512,384))).collect::<Result<Vec<_>>>()?;
        let vertices=Buffer::new(device.clone(),4*6*48)?;
        let mut mesh=Vec::new();
        for p in crate::store_arcade::CEILING_TVS {
            let (p,size)=crate::store_crt_screen::ceiling(p);
            let points: [crate::store_geometry::Vertex;4]=std::array::from_fn(|i|
                [[p[0]+[-0.5,0.5,0.5,-0.5][i]*size[0],p[1]+[-0.5,-0.5,0.5,0.5][i]*size[1],p[2],1.0],
                [if i==1 || i==2 { 1.0 } else { 0.0 },if i<2 { 1.0 } else { 0.0 },0.0,0.0],[1.0;4]]);
            for i in [0,1,2,0,2,3] { mesh.push(points[i]); }
        }
        unsafe { std::ptr::copy_nonoverlapping(mesh.as_ptr() as *const u8,vertices.pointer,mesh.len()*48); }
        Ok(Self { pipeline:ControllerPipeline::textured(device.clone(),textures[0].layout)?,textures,vertices,
            frame:Vec::new(),revision:0,uploaded:vec![0;IN_FLIGHT],next:Instant::now(),device })
    }
    pub fn poll(&mut self,directory: &Path) {
        if Instant::now()<self.next { return; }
        self.next=Instant::now()+Duration::from_millis(50);
        let Ok(sequence)=std::fs::read_to_string(directory.join("tv-sequence")) else { return; };
        let Ok(sequence)=sequence.parse::<u64>() else { return; };
        if sequence==0 || sequence==self.revision { return; }
        let Ok(bytes)=std::fs::read(directory.join("tv-frame.rgba")) else { return; };
        let mut stamp=[0;8];
        let Ok(mut source)=std::fs::File::open(directory.join("tv-frame.rgba")) else { return; };
        if source.read_exact(&mut stamp).is_err() || u64::from_le_bytes(stamp)!=sequence { return; }
        if let Some(pixels)=frame(&bytes,sequence) { self.frame=pixels.to_vec(); self.revision=sequence; }
    }
    pub fn prepare(&mut self,command: vk::CommandBuffer,slot: usize) -> Result<()> {
        if self.revision>0 && self.uploaded[slot]!=self.revision {
            self.textures[slot].upload(&self.frame)?; self.uploaded[slot]=self.revision;
        }
        if self.uploaded[slot]>0 { self.textures[slot].prepare(command); }
        Ok(())
    }
    pub fn draw(&self,command: vk::CommandBuffer,parameters: [[f32;4];5],slot: usize) {
        if self.uploaded[slot]==0 { return; }
        unsafe {
            let d=&self.device.api;
            d.cmd_bind_pipeline(command,vk::PipelineBindPoint::GRAPHICS,self.pipeline.handle);
            d.cmd_bind_descriptor_sets(command,vk::PipelineBindPoint::GRAPHICS,self.pipeline.layout,0,&[self.textures[slot].descriptor],&[]);
            let bytes=std::slice::from_raw_parts(parameters.as_ptr() as *const u8,80);
            d.cmd_push_constants(command,self.pipeline.layout,vk::ShaderStageFlags::VERTEX,0,bytes);
            d.cmd_bind_vertex_buffers(command,0,&[self.vertices.handle],&[0]);
            d.cmd_draw(command,24,1,0,0);
        }
    }
}
fn frame(bytes: &[u8],sequence: u64) -> Option<&[u8]> {
    if bytes.len()!=BYTES+16 || u64::from_le_bytes(bytes[..8].try_into().ok()?)!=sequence
        || u64::from_le_bytes(bytes[BYTES+8..].try_into().ok()?)!=sequence { return None; }
    Some(&bytes[8..BYTES+8])
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn incomplete_or_overwritten_frames_are_not_uploaded() {
        let mut bytes=vec![0;BYTES+16];
        bytes[..8].copy_from_slice(&9_u64.to_le_bytes());
        assert!(frame(&bytes,9).is_none());
        bytes[BYTES+8..].copy_from_slice(&9_u64.to_le_bytes());
        assert_eq!(frame(&bytes,9).unwrap().len(),BYTES);
        assert!(frame(&bytes,8).is_none());
        assert!(frame(&bytes[..BYTES],9).is_none());
    }
}
