use anyhow::{Context, Result, ensure};
use ash::{
    Entry,
    vk::{self, Handle},
};
use openxr as xr;
use std::{ffi::CStr, rc::Rc};

pub struct Instance {
    pub api: ash::Instance,
    _entry: Entry,
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            self.api.destroy_instance(None);
        }
    }
}

pub struct Device {
    pub api: ash::Device,
    pub physical: vk::PhysicalDevice,
    pub family: u32,
    pub queue: vk::Queue,
    pub timestamp_period: f64,
    pub timestamp_mask: u64,
    pub memory: vk::PhysicalDeviceMemoryProperties,
    pub limits: vk::PhysicalDeviceLimits,
    pub instance: Instance,
}

impl Device {
    pub fn new(xr: &xr::Instance, system: xr::SystemId) -> Result<Rc<Self>> {
        let requirements = xr.graphics_requirements::<xr::Vulkan>(system)?;
        ensure!(
            requirements.min_api_version_supported <= xr::Version::new(1, 3, 0)
                && requirements.max_api_version_supported.major() >= 1,
            "Runtime does not support Vulkan 1.3"
        );
        unsafe {
            let entry = Entry::load().context("Load Vulkan")?;
            let get_proc = std::mem::transmute::<
                vk::PFN_vkGetInstanceProcAddr,
                xr::sys::platform::VkGetInstanceProcAddr,
            >(entry.static_fn().get_instance_proc_addr);
            let application = vk::ApplicationInfo::default()
                .application_name(c"MatineeVR")
                .api_version(vk::API_VERSION_1_3);
            let info = vk::InstanceCreateInfo::default().application_info(&application);
            let raw = xr
                .create_vulkan_instance(system, get_proc, &info as *const _ as _)?
                .map_err(vk::Result::from_raw)?;
            let instance = Instance {
                api: ash::Instance::load(entry.static_fn(), vk::Instance::from_raw(raw as u64)),
                _entry: entry,
            };
            let physical =
                vk::PhysicalDevice::from_raw(xr.vulkan_graphics_device(system, raw)? as u64);
            let properties = instance.api.get_physical_device_properties(physical);
            ensure!(
                properties.api_version >= vk::API_VERSION_1_3,
                "GPU requires Vulkan 1.3"
            );
            let families = instance
                .api
                .get_physical_device_queue_family_properties(physical);
            let family = families
                .iter()
                .position(|p| {
                    p.queue_flags
                        .contains(vk::QueueFlags::GRAPHICS | vk::QueueFlags::COMPUTE)
                        && p.timestamp_valid_bits > 0
                })
                .context("No graphics queue with timestamps")? as u32;
            let bits = families[family as usize].timestamp_valid_bits;
            let mut dynamic = vk::PhysicalDeviceDynamicRenderingFeatures::default();
            instance.api.get_physical_device_features2(
                physical,
                &mut vk::PhysicalDeviceFeatures2::default().push_next(&mut dynamic),
            );
            ensure!(
                dynamic.dynamic_rendering != 0,
                "Dynamic rendering unavailable"
            );
            let queues = [vk::DeviceQueueCreateInfo::default()
                .queue_family_index(family)
                .queue_priorities(&[1.0])];
            let extensions = [
                ash::khr::external_memory_fd::NAME.as_ptr(),
                ash::ext::external_memory_dma_buf::NAME.as_ptr(),
                ash::ext::image_drm_format_modifier::NAME.as_ptr(),
                ash::ext::queue_family_foreign::NAME.as_ptr(),
            ];
            let mut ycbcr = vk::PhysicalDeviceSamplerYcbcrConversionFeatures::default();
            instance.api.get_physical_device_features2(
                physical,
                &mut vk::PhysicalDeviceFeatures2::default().push_next(&mut ycbcr),
            );
            ensure!(
                ycbcr.sampler_ycbcr_conversion != 0,
                "YCbCr conversion unavailable"
            );
            let info = vk::DeviceCreateInfo::default()
                .queue_create_infos(&queues)
                .push_next(&mut dynamic)
                .enabled_extension_names(&extensions)
                .push_next(&mut ycbcr);
            let raw_device = xr
                .create_vulkan_device(
                    system,
                    get_proc,
                    physical.as_raw() as _,
                    &info as *const _ as _,
                )?
                .map_err(vk::Result::from_raw)?;
            let api = ash::Device::load(
                instance.api.fp_v1_0(),
                vk::Device::from_raw(raw_device as u64),
            );
            let queue = api.get_device_queue(family, 0);
            let memory = instance.api.get_physical_device_memory_properties(physical);
            eprintln!(
                "GPU: {} Vulkan {}.{}.{} timestamp_ns={}",
                CStr::from_ptr(properties.device_name.as_ptr()).to_string_lossy(),
                vk::api_version_major(properties.api_version),
                vk::api_version_minor(properties.api_version),
                vk::api_version_patch(properties.api_version),
                properties.limits.timestamp_period
            );
            Ok(Rc::new(Self {
                api,
                physical,
                family,
                queue,
                memory,
                limits: properties.limits,
                instance,
                timestamp_period: properties.limits.timestamp_period as f64,
                timestamp_mask: u64::MAX >> (64 - bits),
            }))
        }
    }

    pub fn session(
        &self,
        instance: &xr::Instance,
        system: xr::SystemId,
    ) -> Result<super::graphics::Session> {
        Ok(unsafe {
            instance.create_session(
                system,
                &xr::vulkan::SessionCreateInfo {
                    instance: self.instance.api.handle().as_raw() as _,
                    physical_device: self.physical.as_raw() as _,
                    device: self.api.handle().as_raw() as _,
                    queue_family_index: self.family,
                    queue_index: 0,
                },
            )
        }?)
    }

    pub fn memory_type(&self, bits: u32, flags: vk::MemoryPropertyFlags) -> Result<u32> {
        (0..self.memory.memory_type_count)
            .find(|i| {
                bits & (1 << i) != 0
                    && self.memory.memory_types[*i as usize]
                        .property_flags
                        .contains(flags)
            })
            .context("No compatible Vulkan memory type")
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            if let Err(error) = self.api.device_wait_idle() {
                eprintln!("Vulkan shutdown: {error}");
            }
            self.api.destroy_device(None);
        }
    }
}
