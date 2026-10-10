use crate::{vk_commands::Commands, vk_device::Device, vk_memory::Buffer, vk_sync::barrier};
use anyhow::{Result, ensure};
use ash::vk;
use std::rc::Rc;

pub struct Upload {
    commands: Commands,
    buffer: Buffer,
    size: (i32, i32),
    device: Rc<Device>,
}

impl Upload {
    pub fn new(device: Rc<Device>, size: (i32, i32)) -> Result<Self> {
        Ok(Self {
            commands: Commands::new(device.clone())?,
            buffer: Buffer::new(device.clone(), (size.0 * size.1 * 4) as usize)?,
            size,
            device,
        })
    }

    pub fn write(&mut self, image: vk::Image, pixels: &[u8]) -> Result<()> {
        ensure!(
            pixels.len() == self.buffer.size,
            "Panel pixel count differs from its dimensions"
        );
        self.commands.collect()?;
        let staging =
            unsafe { std::slice::from_raw_parts_mut(self.buffer.pointer, self.buffer.size) };
        for (target, source) in staging
            .chunks_exact_mut(self.size.0 as usize * 4)
            .zip(pixels.chunks_exact(self.size.0 as usize * 4).rev())
        {
            target.copy_from_slice(source);
        }
        self.commands.begin()?;
        let command = self.commands.command;
        barrier(
            &self.device.api,
            command,
            image,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        );
        unsafe {
            self.device.api.cmd_copy_buffer_to_image(
                command,
                self.buffer.handle,
                image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[vk::BufferImageCopy::default()
                    .image_subresource(
                        vk::ImageSubresourceLayers::default()
                            .aspect_mask(vk::ImageAspectFlags::COLOR)
                            .layer_count(1),
                    )
                    .image_extent(vk::Extent3D {
                        width: self.size.0 as u32,
                        height: self.size.1 as u32,
                        depth: 1,
                    })],
            );
        }
        barrier(
            &self.device.api,
            command,
            image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        );
        self.commands.timestamp(1);
        self.commands.timestamp(2);
        self.commands.submit(0)?;
        Ok(())
    }
}
