use super::vk_device::Device;
use anyhow::{Result, ensure};
use ash::vk;
use std::{ptr, rc::Rc};

pub const COLOR: vk::ImageSubresourceRange = vk::ImageSubresourceRange {
    aspect_mask: vk::ImageAspectFlags::COLOR,
    base_mip_level: 0,
    level_count: 1,
    base_array_layer: 0,
    layer_count: 1,
};

pub struct Buffer {
    pub handle: vk::Buffer,
    pub pointer: *mut u8,
    pub size: usize,
    memory: vk::DeviceMemory,
    device: Rc<Device>,
}

impl Buffer {
    pub fn new(device: Rc<Device>, size: usize) -> Result<Self> {
        ensure!(size > 0, "Empty staging buffer");
        let mut result = Self {
            handle: vk::Buffer::null(),
            memory: vk::DeviceMemory::null(),
            pointer: ptr::null_mut(),
            size,
            device,
        };
        unsafe {
            let d = &result.device.api;
            result.handle = d.create_buffer(
                &vk::BufferCreateInfo::default().size(size as u64).usage(
                    vk::BufferUsageFlags::TRANSFER_SRC
                        | vk::BufferUsageFlags::TRANSFER_DST
                        | vk::BufferUsageFlags::STORAGE_BUFFER,
                ),
                None,
            )?;
            let requirements = d.get_buffer_memory_requirements(result.handle);
            result.memory = d.allocate_memory(
                &vk::MemoryAllocateInfo::default()
                    .allocation_size(requirements.size)
                    .memory_type_index(result.device.memory_type(
                        requirements.memory_type_bits,
                        vk::MemoryPropertyFlags::HOST_VISIBLE
                            | vk::MemoryPropertyFlags::HOST_COHERENT,
                    )?),
                None,
            )?;
            d.bind_buffer_memory(result.handle, result.memory, 0)?;
            result.pointer = d.map_memory(
                result.memory,
                0,
                vk::WHOLE_SIZE,
                vk::MemoryMapFlags::empty(),
            )? as _;
        }
        Ok(result)
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            let d = &self.device.api;
            if !self.pointer.is_null() {
                d.unmap_memory(self.memory);
            }
            d.destroy_buffer(self.handle, None);
            d.free_memory(self.memory, None);
        }
    }
}

pub struct Image {
    pub handle: vk::Image,
    pub view: vk::ImageView,
    memory: vk::DeviceMemory,
    owned: bool,
    device: Rc<Device>,
}

impl Image {
    pub fn new(
        device: Rc<Device>,
        size: (u32, u32),
        format: vk::Format,
        usage: vk::ImageUsageFlags,
    ) -> Result<Self> {
        let mut image = Self {
            handle: vk::Image::null(),
            view: vk::ImageView::null(),
            memory: vk::DeviceMemory::null(),
            owned: true,
            device,
        };
        unsafe {
            let d = &image.device.api;
            image.handle = d.create_image(
                &vk::ImageCreateInfo::default()
                    .image_type(vk::ImageType::TYPE_2D)
                    .format(format)
                    .extent(vk::Extent3D {
                        width: size.0,
                        height: size.1,
                        depth: 1,
                    })
                    .mip_levels(1)
                    .array_layers(1)
                    .samples(vk::SampleCountFlags::TYPE_1)
                    .usage(usage),
                None,
            )?;
            let requirements = d.get_image_memory_requirements(image.handle);
            image.memory = d.allocate_memory(
                &vk::MemoryAllocateInfo::default()
                    .allocation_size(requirements.size)
                    .memory_type_index(image.device.memory_type(
                        requirements.memory_type_bits,
                        vk::MemoryPropertyFlags::DEVICE_LOCAL,
                    )?),
                None,
            )?;
            d.bind_image_memory(image.handle, image.memory, 0)?;
            image.view = image.make_view(format)?;
        }
        Ok(image)
    }

    pub fn borrowed(device: Rc<Device>, handle: vk::Image, format: vk::Format) -> Result<Self> {
        let mut image = Self {
            handle,
            view: vk::ImageView::null(),
            memory: vk::DeviceMemory::null(),
            owned: false,
            device,
        };
        image.view = image.make_view(format)?;
        Ok(image)
    }

    fn make_view(&self, format: vk::Format) -> Result<vk::ImageView> {
        Ok(unsafe {
            self.device.api.create_image_view(
                &vk::ImageViewCreateInfo::default()
                    .image(self.handle)
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(format)
                    .subresource_range(COLOR),
                None,
            )
        }?)
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        unsafe {
            self.device.api.destroy_image_view(self.view, None);
            if self.owned {
                self.device.api.destroy_image(self.handle, None);
                self.device.api.free_memory(self.memory, None);
            }
        }
    }
}
