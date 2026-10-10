use anyhow::Result;
use ash::vk;
use crate::graphics::Graphics;
use std::rc::Rc;

pub struct ControllerDepth {
    pub image: vk::Image,
    pub view: vk::ImageView,
    memory: vk::DeviceMemory,
    device: Rc<Graphics>,
}

impl ControllerDepth {
    pub fn new(device: Rc<Graphics>, size: (i32, i32)) -> Result<Self> {
        let mut depth = Self { image: vk::Image::null(), view: vk::ImageView::null(),
            memory: vk::DeviceMemory::null(), device };
        unsafe {
            let d = &depth.device.api;
            depth.image = d.create_image(&vk::ImageCreateInfo::default()
                .image_type(vk::ImageType::TYPE_2D).format(vk::Format::D32_SFLOAT)
                .extent(vk::Extent3D { width: size.0 as u32, height: size.1 as u32, depth: 1 })
                .mip_levels(1).array_layers(1).samples(vk::SampleCountFlags::TYPE_1)
                .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT), None)?;
            let required = d.get_image_memory_requirements(depth.image);
            depth.memory = d.allocate_memory(&vk::MemoryAllocateInfo::default()
                .allocation_size(required.size).memory_type_index(depth.device.memory_type(
                    required.memory_type_bits, vk::MemoryPropertyFlags::DEVICE_LOCAL)?), None)?;
            d.bind_image_memory(depth.image, depth.memory, 0)?;
            depth.view = d.create_image_view(&vk::ImageViewCreateInfo::default()
                .image(depth.image).view_type(vk::ImageViewType::TYPE_2D).format(vk::Format::D32_SFLOAT)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::DEPTH, base_mip_level: 0, level_count: 1,
                    base_array_layer: 0, layer_count: 1 }), None)?;
        }
        Ok(depth)
    }

    pub fn prepare(&self, command: vk::CommandBuffer) {
        let barrier = [vk::ImageMemoryBarrier::default().image(self.image)
            .old_layout(vk::ImageLayout::UNDEFINED).new_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED).dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_access_mask(vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::DEPTH, base_mip_level: 0, level_count: 1,
                base_array_layer: 0, layer_count: 1 })];
        unsafe { self.device.api.cmd_pipeline_barrier(command,
            vk::PipelineStageFlags::TOP_OF_PIPE, vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            vk::DependencyFlags::empty(), &[], &[], &barrier); }
    }
}

impl Drop for ControllerDepth {
    fn drop(&mut self) {
        unsafe {
            let d = &self.device.api;
            d.destroy_image_view(self.view, None);
            d.destroy_image(self.image, None);
            d.free_memory(self.memory, None);
        }
    }
}
