use ash::vk::{self, AccessFlags as Access, ImageLayout as Layout, PipelineStageFlags as Stage};

fn scope(layout: Layout) -> (Stage, Access) {
    match layout {
        Layout::UNDEFINED => (Stage::TOP_OF_PIPE, Access::empty()),
        Layout::SHADER_READ_ONLY_OPTIMAL => (Stage::FRAGMENT_SHADER, Access::SHADER_READ),
        Layout::COLOR_ATTACHMENT_OPTIMAL => (
            Stage::COLOR_ATTACHMENT_OUTPUT,
            Access::COLOR_ATTACHMENT_WRITE,
        ),
        Layout::TRANSFER_SRC_OPTIMAL => (Stage::TRANSFER, Access::TRANSFER_READ),
        Layout::TRANSFER_DST_OPTIMAL => (Stage::TRANSFER, Access::TRANSFER_WRITE),
        _ => unreachable!("Unsupported image layout"),
    }
}

pub fn barrier(
    d: &ash::Device,
    command: vk::CommandBuffer,
    image: vk::Image,
    old: Layout,
    new: Layout,
) {
    let (source_stage, source_access) = scope(old);
    let (target_stage, target_access) = scope(new);
    unsafe {
        d.cmd_pipeline_barrier(
            command,
            source_stage,
            target_stage,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[vk::ImageMemoryBarrier::default()
                .image(image)
                .old_layout(old)
                .new_layout(new)
                .subresource_range(super::vk_memory::COLOR)
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .src_access_mask(source_access)
                .dst_access_mask(target_access)],
        );
    }
}

pub fn foreign(
    device: &super::vk_device::Device,
    command: vk::CommandBuffer,
    image: vk::Image,
    acquire: bool,
) {
    let stages = vk::PipelineStageFlags::COMPUTE_SHADER | vk::PipelineStageFlags::FRAGMENT_SHADER;
    let (src, dst, old, new, src_access, dst_access, src_stage, dst_stage) = if acquire {
        (
            vk::QUEUE_FAMILY_FOREIGN_EXT,
            device.family,
            vk::ImageLayout::GENERAL,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            vk::AccessFlags::empty(),
            vk::AccessFlags::SHADER_READ,
            vk::PipelineStageFlags::TOP_OF_PIPE,
            stages,
        )
    } else {
        (
            device.family,
            vk::QUEUE_FAMILY_FOREIGN_EXT,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            vk::ImageLayout::GENERAL,
            vk::AccessFlags::SHADER_READ,
            vk::AccessFlags::empty(),
            stages,
            vk::PipelineStageFlags::BOTTOM_OF_PIPE,
        )
    };
    unsafe {
        device.api.cmd_pipeline_barrier(
            command,
            src_stage,
            dst_stage,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[vk::ImageMemoryBarrier::default()
                .image(image)
                .subresource_range(super::vk_memory::COLOR)
                .old_layout(old)
                .new_layout(new)
                .src_queue_family_index(src)
                .dst_queue_family_index(dst)
                .src_access_mask(src_access)
                .dst_access_mask(dst_access)],
        );
    }
}
