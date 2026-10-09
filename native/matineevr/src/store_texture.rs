use crate::graphics::Graphics;
use anyhow::Result;
use ash::vk;
use matineevr::{vk_memory::{Image,COLOR}, vk_pipeline::FORMAT, vk_transfer::Upload};
use std::rc::Rc;

pub struct StoreTexture {
    pub layout: vk::DescriptorSetLayout,
    pub descriptor: vk::DescriptorSet,
    pool: vk::DescriptorPool,
    sampler: vk::Sampler,
    image: Image,
    upload: Upload,
    dirty: bool,
    device: Rc<Graphics>,
}
impl StoreTexture {
    pub fn new(device: Rc<Graphics>) -> Result<Self> {
        Self::with_size(device,(1440,3240))
    }
    pub fn with_size(device: Rc<Graphics>,size: (u32,u32)) -> Result<Self> {
        Self::with_format(device,size,FORMAT)
    }
    pub fn with_format(device: Rc<Graphics>,size: (u32,u32),format: vk::Format) -> Result<Self> {
        let mut t = Self { layout:vk::DescriptorSetLayout::null(), descriptor:vk::DescriptorSet::null(),
            pool:vk::DescriptorPool::null(), sampler:vk::Sampler::null(),
            image:Image::new(device.clone(),size,format,
                vk::ImageUsageFlags::SAMPLED|vk::ImageUsageFlags::TRANSFER_DST|vk::ImageUsageFlags::COLOR_ATTACHMENT)?,
            upload:Upload::new(device.clone(),(size.0 as i32,size.1 as i32))?,dirty:false,device };
        unsafe {
            let d = &t.device.api;
            t.sampler = d.create_sampler(&vk::SamplerCreateInfo::default()
                .mag_filter(vk::Filter::LINEAR).min_filter(vk::Filter::LINEAR)
                .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE),None)?;
            let bindings = [vk::DescriptorSetLayoutBinding::default().binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT)];
            t.layout = d.create_descriptor_set_layout(&vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings),None)?;
            let sizes = [vk::DescriptorPoolSize { ty:vk::DescriptorType::COMBINED_IMAGE_SAMPLER,descriptor_count:1 }];
            t.pool = d.create_descriptor_pool(&vk::DescriptorPoolCreateInfo::default().max_sets(1).pool_sizes(&sizes),None)?;
            t.descriptor = d.allocate_descriptor_sets(&vk::DescriptorSetAllocateInfo::default()
                .descriptor_pool(t.pool).set_layouts(&[t.layout]))?[0];
            let image = [vk::DescriptorImageInfo::default().sampler(t.sampler).image_view(t.image.view)
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)];
            d.update_descriptor_sets(&[vk::WriteDescriptorSet::default().dst_set(t.descriptor).dst_binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).image_info(&image)],&[]);
        }
        Ok(t)
    }
    pub fn upload(&mut self,pixels: &[u8]) -> Result<()> {
        self.upload.write(self.image.handle,pixels)?; self.dirty = true; Ok(())
    }
    pub fn prepare(&mut self,command: vk::CommandBuffer) {
        if !self.dirty { return; }
        let barrier = [vk::ImageMemoryBarrier::default().image(self.image.handle)
            .old_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL).new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED).dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .src_access_mask(vk::AccessFlags::MEMORY_WRITE).dst_access_mask(vk::AccessFlags::SHADER_READ)
            .subresource_range(COLOR)];
        unsafe { self.device.api.cmd_pipeline_barrier(command,vk::PipelineStageFlags::ALL_COMMANDS,
            vk::PipelineStageFlags::FRAGMENT_SHADER,vk::DependencyFlags::empty(),&[],&[],&barrier); }
        self.dirty = false;
    }
}
impl Drop for StoreTexture {
    fn drop(&mut self) {
        unsafe {
            let d = &self.device.api;
            if let Err(e) = d.device_wait_idle() { eprintln!("Store texture shutdown: {e}"); }
            d.destroy_descriptor_pool(self.pool,None); d.destroy_descriptor_set_layout(self.layout,None);
            d.destroy_sampler(self.sampler,None);
        }
    }
}
