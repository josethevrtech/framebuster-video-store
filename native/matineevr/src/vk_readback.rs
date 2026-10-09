use super::{vk_commands::Commands, vk_device::Device, vk_memory::Buffer, vk_sync::barrier};
use anyhow::Result;
use ash::vk;
use std::rc::Rc;

pub fn read(device: &Rc<Device>, image: vk::Image, size: (i32, i32)) -> Result<Vec<u8>> {
    let buffer = Buffer::new(device.clone(), (size.0 * size.1 * 4) as usize)?;
    let mut commands = Commands::new(device.clone())?;
    commands.begin()?;
    barrier(
        &device.api,
        commands.command,
        image,
        vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
    );
    unsafe {
        device.api.cmd_copy_image_to_buffer(
            commands.command,
            image,
            vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            buffer.handle,
            &[vk::BufferImageCopy::default()
                .image_subresource(
                    vk::ImageSubresourceLayers::default()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .layer_count(1),
                )
                .image_extent(vk::Extent3D {
                    width: size.0 as u32,
                    height: size.1 as u32,
                    depth: 1,
                })],
        );
        device.api.cmd_pipeline_barrier(
            commands.command,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::HOST,
            vk::DependencyFlags::empty(),
            &[vk::MemoryBarrier::default()
                .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                .dst_access_mask(vk::AccessFlags::HOST_READ)],
            &[],
            &[],
        );
    }
    barrier(
        &device.api,
        commands.command,
        image,
        vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
        vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
    );
    commands.timestamp(1);
    commands.timestamp(2);
    commands.submit(0)?;
    commands.collect()?;
    Ok(unsafe { std::slice::from_raw_parts(buffer.pointer, buffer.size) }.to_vec())
}
