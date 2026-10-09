use super::{
    renderer::Renderer,
    renderer::Target,
    vk_memory::Image,
    vk_pipeline::{FORMAT, Parameters},
    vk_sync::barrier,
};
use crate::{presentation::Presentation, video_params::View};
use anyhow::Result;
use ash::vk::{self, Handle};
use openxr as xr;

impl Renderer {
    pub(crate) fn draw_target(
        &mut self,
        command: vk::CommandBuffer,
        target: Target,
        view: &xr::View,
        eye: i32,
        presentation: Presentation,
        yaw: f32,
    ) -> Result<()> {
        let device = &self.device;
        let targets_cache = &mut self.targets;
        let d = &device.api;
        unsafe {
            if let std::collections::hash_map::Entry::Vacant(entry) =
                targets_cache.entry(target.image)
            {
                entry.insert(Image::borrowed(
                    device.clone(),
                    vk::Image::from_raw(target.image),
                    FORMAT,
                )?);
            }
            let image = &targets_cache[&target.image];
            barrier(
                d,
                command,
                image.handle,
                vk::ImageLayout::UNDEFINED,
                vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            );
            let extent = vk::Extent2D {
                width: target.size.0 as u32,
                height: target.size.1 as u32,
            };
            let area = vk::Rect2D::default().extent(extent);
            let attachment = [vk::RenderingAttachmentInfo::default()
                .image_view(image.view)
                .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .clear_value(vk::ClearValue {
                    color: vk::ClearColorValue {
                        float32: [0.0, 0.0, 0.0, 1.0],
                    },
                })
                .store_op(vk::AttachmentStoreOp::STORE)];
            d.cmd_begin_rendering(
                command,
                &vk::RenderingInfo::default()
                    .render_area(area)
                    .layer_count(1)
                    .color_attachments(&attachment),
            );
            d.cmd_set_viewport(
                command,
                0,
                &[vk::Viewport {
                    x: 0.0,
                    y: extent.height as f32,
                    width: extent.width as f32,
                    height: -(extent.height as f32),
                    min_depth: 0.0,
                    max_depth: 1.0,
                }],
            );
            d.cmd_set_scissor(command, 0, &[area]);
            if let Some(frame) = &self.current {
                let direct = self.direct.as_ref().unwrap();
                let parameters = Parameters {
                    view: View::new(
                        (frame.pixels.width, frame.pixels.height),
                        frame.pixels.sample_aspect_ratio,
                        view,
                        eye,
                        presentation,
                        yaw,
                    ),
                    crop: direct.crop(),
                };
                d.cmd_push_constants(
                    command,
                    direct.pipeline.layout,
                    vk::ShaderStageFlags::FRAGMENT,
                    0,
                    parameters.bytes(),
                );
                d.cmd_draw(command, 3, 1, 0, 0);
            }
            d.cmd_end_rendering(command);
        }
        Ok(())
    }
}
