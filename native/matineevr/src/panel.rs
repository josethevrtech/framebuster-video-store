use crate::hud_text::WIDTH;
use anyhow::{Context, Result, ensure};
use ash::vk::{self, Handle};
use matineevr::{vk_pipeline::FORMAT, vk_transfer::Upload};
use openxr as xr;
use std::rc::Rc;

pub struct Panel {
    pub size: [usize; 2],
    upload: Upload,
    chain: xr::Swapchain<xr::Vulkan>,
    images: Vec<u64>,
}

impl Panel {
    pub fn new(
        session: &xr::Session<xr::Vulkan>,
        device: Rc<crate::graphics::Graphics>,
        size: [usize; 2],
    ) -> Result<Self> {
        let formats = session.enumerate_swapchain_formats()?;
        ensure!(
            formats.contains(&(FORMAT.as_raw() as u32)),
            "HUD requires an sRGB RGBA swapchain"
        );
        let chain = session.create_swapchain(&xr::SwapchainCreateInfo {
            create_flags: Default::default(),
            usage_flags: xr::SwapchainUsageFlags::TRANSFER_DST
                | xr::SwapchainUsageFlags::SAMPLED
                | xr::SwapchainUsageFlags::COLOR_ATTACHMENT,
            format: (FORMAT.as_raw() as u32),
            sample_count: 1,
            width: size[0] as u32,
            height: size[1] as u32,
            face_count: 1,
            array_size: 1,
            mip_count: 1,
        }).context("Create native panel swapchain")?;
        let images = chain.enumerate_images()?;
        Ok(Self {
            chain,
            images,
            upload: Upload::new(device, (size[0] as i32, size[1] as i32))?,
            size,
        })
    }

    pub fn upload(&mut self, pixels: &[u8]) -> Result<()> {
        let image = self.chain.acquire_image()?;
        self.chain.wait_image(xr::Duration::INFINITE)?;
        let result = self
            .upload
            .write(vk::Image::from_raw(self.images[image as usize]), pixels);
        self.chain.release_image()?;
        result
    }

    pub fn render(&mut self, draw: impl FnOnce(u64) -> Result<()>) -> Result<()> {
        let image = self.chain.acquire_image()?;
        self.chain.wait_image(xr::Duration::INFINITE)?;
        let result = draw(self.images[image as usize]);
        self.chain.release_image()?;
        result
    }

    pub fn layer<'a>(
        &'a self,
        space: &'a xr::Space,
        pose: xr::Posef,
        width: f32,
    ) -> xr::CompositionLayerQuad<'a, xr::Vulkan> {
        self.region_layer(space, pose, width, 0, self.size)
    }

    pub fn region_layer<'a>(&'a self, space: &'a xr::Space, pose: xr::Posef,
        width: f32, bottom: usize, size: [usize; 2]) -> xr::CompositionLayerQuad<'a, xr::Vulkan> {
        xr::CompositionLayerQuad::new()
            .space(space)
            .eye_visibility(xr::EyeVisibility::BOTH)
            .layer_flags(
                xr::CompositionLayerFlags::BLEND_TEXTURE_SOURCE_ALPHA
                    | xr::CompositionLayerFlags::UNPREMULTIPLIED_ALPHA,
            )
            .pose(pose)
            .size(extent(size, width))
            .sub_image(
                xr::SwapchainSubImage::new()
                    .swapchain(&self.chain)
                    .image_rect(xr::Rect2Di {
                        offset: xr::Offset2Di { x: 0, y: bottom as i32 },
                        extent: xr::Extent2Di {
                            width: size[0] as i32,
                            height: size[1] as i32,
                        },
                    }),
            )
    }
}

fn extent(size: [usize; 2], width: f32) -> xr::Extent2Df {
    xr::Extent2Df {
        width: width * size[0] as f32 / WIDTH as f32,
        height: width * size[1] as f32 / WIDTH as f32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{browser_view::panel_size, shortcuts::PanelKind::Browser};

    #[test]
    fn standalone_browser_keeps_width_and_scales_height() {
        let small = extent(panel_size(Browser, true), 1.2);
        let large = extent(panel_size(Browser, false), 1.2);
        assert!((large.width / small.width - 1.0).abs() < 0.0001);
        assert!((large.height / small.height - 4.0).abs() < 0.0001);
    }
}
