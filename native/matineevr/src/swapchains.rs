use anyhow::{Result, ensure};
use openxr as xr;

pub struct Eye {
    pub chain: xr::Swapchain<xr::Vulkan>,
    pub images: Vec<u64>,
    pub size: xr::Extent2Di,
}

fn projection_views<'a>(
    eyes: &'a [Eye],
    views: &[xr::View],
) -> Vec<xr::CompositionLayerProjectionView<'a, xr::Vulkan>> {
    eyes.iter()
        .zip(views)
        .map(|(eye, view)| {
            xr::CompositionLayerProjectionView::new()
                .pose(view.pose)
                .fov(view.fov)
                .sub_image(
                    xr::SwapchainSubImage::new()
                        .swapchain(&eye.chain)
                        .image_rect(xr::Rect2Di {
                            offset: xr::Offset2Di { x: 0, y: 0 },
                            extent: eye.size,
                        }),
                )
        })
        .collect()
}

pub fn submit(
    stream: &mut xr::FrameStream<xr::Vulkan>,
    time: xr::Time,
    space: &xr::Space,
    eyes: &[Eye],
    views: &[xr::View],
    quads: &[xr::CompositionLayerQuad<'_, xr::Vulkan>],
) -> Result<()> {
    let views = projection_views(eyes, views);
    let projection = xr::CompositionLayerProjection::new()
        .space(space)
        .views(&views);
    let mut layers: Vec<&xr::CompositionLayerBase<'_, xr::Vulkan>> = vec![&projection];
    layers.extend(quads.iter().map(|quad| &**quad));
    stream.end(time, xr::EnvironmentBlendMode::OPAQUE, &layers)?;
    Ok(())
}

pub fn create(
    instance: &xr::Instance,
    system: xr::SystemId,
    session: &xr::Session<xr::Vulkan>,
) -> Result<(u32, Vec<Eye>)> {
    let formats = session.enumerate_swapchain_formats()?;
    let format = matineevr::vk_pipeline::FORMAT.as_raw() as u32;
    ensure!(
        formats.contains(&format),
        "No sRGB RGBA XR swapchain format"
    );
    let views = instance
        .enumerate_view_configuration_views(system, xr::ViewConfigurationType::PRIMARY_STEREO)?;
    ensure!(views.len() == 2, "Expected two stereo views");
    let mut eyes = Vec::new();
    for view in views {
        let chain = session.create_swapchain(&xr::SwapchainCreateInfo {
            create_flags: Default::default(),
            usage_flags: xr::SwapchainUsageFlags::COLOR_ATTACHMENT
                | xr::SwapchainUsageFlags::TRANSFER_SRC,
            format,
            sample_count: 1,
            width: view.recommended_image_rect_width,
            height: view.recommended_image_rect_height,
            face_count: 1,
            array_size: 1,
            mip_count: 1,
        })?;
        let images = chain.enumerate_images()?;
        let size = xr::Extent2Di {
            width: view.recommended_image_rect_width as i32,
            height: view.recommended_image_rect_height as i32,
        };
        eprintln!("XR eye: {}x{}", size.width, size.height);
        eyes.push(Eye {
            chain,
            images,
            size,
        });
    }
    Ok((format, eyes))
}
