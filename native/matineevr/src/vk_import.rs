use super::vk_device::Device;
use crate::media::DmaBuf;
use anyhow::{Context, Result, bail, ensure};
use ash::vk;
use std::{
    os::fd::{AsRawFd, BorrowedFd, IntoRawFd},
    rc::Rc,
};

pub struct Imported {
    pub handle: vk::Image,
    pub format: vk::Format,
    pub descriptor_count: u32,
    pub features: vk::FormatFeatureFlags,
    memory: vk::DeviceMemory,
    device: Rc<Device>,
}

impl Imported {
    pub fn new(device: Rc<Device>, source: &DmaBuf) -> Result<Self> {
        let format = match source.format.to_le_bytes() {
            [b'N', b'V', b'1', b'2'] => vk::Format::G8_B8R8_2PLANE_420_UNORM,
            [b'P', b'0', b'1', b'0'] => vk::Format::G10X6_B10X6R10X6_2PLANE_420_UNORM_3PACK16,
            _ => bail!("Unsupported DRM format: {:#x}", source.format),
        };
        let mut result = Self {
            handle: vk::Image::null(),
            format,
            descriptor_count: 0,
            features: vk::FormatFeatureFlags::empty(),
            memory: vk::DeviceMemory::null(),
            device,
        };
        unsafe {
            let device = &result.device;
            let d = &device.api;
            let handle_type = vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT;
            let mut modifiers = vk::DrmFormatModifierPropertiesListEXT::default();
            device.instance.api.get_physical_device_format_properties2(
                device.physical,
                format,
                &mut vk::FormatProperties2::default().push_next(&mut modifiers),
            );
            let mut entries = vec![
                vk::DrmFormatModifierPropertiesEXT::default();
                modifiers.drm_format_modifier_count as usize
            ];
            modifiers = modifiers.drm_format_modifier_properties(&mut entries);
            device.instance.api.get_physical_device_format_properties2(
                device.physical,
                format,
                &mut vk::FormatProperties2::default().push_next(&mut modifiers),
            );
            result.features = entries
                .iter()
                .find(|m| {
                    m.drm_format_modifier == source.modifier
                        && m.drm_format_modifier_plane_count == 2
                })
                .context("Unsupported DRM modifier or plane count")?
                .drm_format_modifier_tiling_features;
            let mut external =
                vk::PhysicalDeviceExternalImageFormatInfo::default().handle_type(handle_type);
            let mut modifier = vk::PhysicalDeviceImageDrmFormatModifierInfoEXT::default()
                .drm_format_modifier(source.modifier)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);
            let info = vk::PhysicalDeviceImageFormatInfo2::default()
                .format(format)
                .ty(vk::ImageType::TYPE_2D)
                .tiling(vk::ImageTiling::DRM_FORMAT_MODIFIER_EXT)
                .usage(vk::ImageUsageFlags::SAMPLED)
                .push_next(&mut external)
                .push_next(&mut modifier);
            let mut ycbcr = vk::SamplerYcbcrConversionImageFormatProperties::default();
            let mut external_properties = vk::ExternalImageFormatProperties::default();
            let mut properties = vk::ImageFormatProperties2::default()
                .push_next(&mut external_properties)
                .push_next(&mut ycbcr);
            device
                .instance
                .api
                .get_physical_device_image_format_properties2(
                    device.physical,
                    &info,
                    &mut properties,
                )?;
            ensure!(
                external_properties
                    .external_memory_properties
                    .external_memory_features
                    .contains(vk::ExternalMemoryFeatureFlags::IMPORTABLE),
                "DMA-BUF image is not importable"
            );
            result.descriptor_count = ycbcr.combined_image_sampler_descriptor_count;
            let layouts = [0, 1].map(|i| {
                vk::SubresourceLayout::default()
                    .offset(source.offsets[i])
                    .row_pitch(source.pitches[i])
            });
            let mut explicit = vk::ImageDrmFormatModifierExplicitCreateInfoEXT::default()
                .drm_format_modifier(source.modifier)
                .plane_layouts(&layouts);
            let mut external =
                vk::ExternalMemoryImageCreateInfo::default().handle_types(handle_type);
            result.handle = d
                .create_image(
                    &vk::ImageCreateInfo::default()
                        .image_type(vk::ImageType::TYPE_2D)
                        .format(format)
                        .extent(vk::Extent3D {
                            width: (source.width - source.crop[2]) as u32,
                            height: (source.height - source.crop[3]) as u32,
                            depth: 1,
                        })
                        .mip_levels(1)
                        .array_layers(1)
                        .samples(vk::SampleCountFlags::TYPE_1)
                        .tiling(vk::ImageTiling::DRM_FORMAT_MODIFIER_EXT)
                        .usage(vk::ImageUsageFlags::SAMPLED)
                        .push_next(&mut explicit)
                        .push_next(&mut external),
                    None,
                )
                .context("Create image with the decoder's explicit layout")?;
            let mut dedicated = vk::MemoryDedicatedRequirements::default();
            let mut requirements = vk::MemoryRequirements2::default().push_next(&mut dedicated);
            d.get_image_memory_requirements2(
                &vk::ImageMemoryRequirementsInfo2::default().image(result.handle),
                &mut requirements,
            );
            let requirements = requirements.memory_requirements;
            ensure!(
                requirements.size <= source.size,
                "Decoder allocation is smaller than Vulkan requires: {} < {}",
                source.size,
                requirements.size
            );
            let fd = BorrowedFd::borrow_raw(source.fd).try_clone_to_owned()?;
            let loader = ash::khr::external_memory_fd::Device::new(&device.instance.api, d);
            let mut fd_properties = vk::MemoryFdPropertiesKHR::default();
            loader.get_memory_fd_properties(handle_type, fd.as_raw_fd(), &mut fd_properties)?;
            let mut import = vk::ImportMemoryFdInfoKHR::default()
                .handle_type(handle_type)
                .fd(fd.as_raw_fd());
            let mut allocation = vk::MemoryDedicatedAllocateInfo::default().image(result.handle);
            result.memory = d.allocate_memory(
                &vk::MemoryAllocateInfo::default()
                    .allocation_size(source.size)
                    .memory_type_index(device.memory_type(
                        requirements.memory_type_bits & fd_properties.memory_type_bits,
                        vk::MemoryPropertyFlags::empty(),
                    )?)
                    .push_next(&mut import)
                    .push_next(&mut allocation),
                None,
            )?;
            let _ = fd.into_raw_fd();
            d.bind_image_memory(result.handle, result.memory, 0)?;
        }
        Ok(result)
    }
}

impl Drop for Imported {
    fn drop(&mut self) {
        unsafe {
            self.device.api.destroy_image(self.handle, None);
            self.device.api.free_memory(self.memory, None);
        }
    }
}
