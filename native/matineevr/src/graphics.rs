use anyhow::{Context, Result, ensure};
use openxr as xr;

pub use crate::vk_device::Device as Graphics;
pub type Session = (
    xr::Session<xr::Vulkan>,
    xr::FrameWaiter,
    xr::FrameStream<xr::Vulkan>,
);

impl Graphics {
    pub fn instance() -> Result<xr::Instance> {
        let entry = unsafe { xr::Entry::load() }.context("Load native SteamVR OpenXR")?;
        ensure!(
            entry.enumerate_extensions()?.khr_vulkan_enable2,
            "Runtime lacks Vulkan support"
        );
        let mut extensions = xr::ExtensionSet::default();
        extensions.khr_vulkan_enable2 = true;
        extensions.other = vec![b"XR_VALVE_frame_controller_interaction\0".to_vec()];
        entry
            .create_instance(
                &xr::ApplicationInfo {
                    application_name: "FrameBuster Video Store",
                    ..Default::default()
                },
                &extensions,
                &[],
            )
            .context("Create OpenXR instance")
    }
}
