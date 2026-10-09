use crate::{controller_depth::ControllerDepth, controller_mesh::ControllerMesh,
    controller_pipeline::ControllerPipeline, graphics::Graphics};
use anyhow::Result;
use ash::vk::{self, Handle};
use matineevr::{renderer::{IN_FLIGHT, Target}, vk_commands::Commands,
    vk_memory::{COLOR, Image}, vk_pipeline::FORMAT};
use openxr as xr;
use std::{collections::HashMap, rc::Rc};

pub struct ControllerDraw {
    commands: Vec<Commands>,
    pipeline: ControllerPipeline,
    targets: HashMap<u64, (Image, ControllerDepth)>,
    mesh: ControllerMesh,
    device: Rc<Graphics>,
    sequence: usize,
    pub store: Option<crate::store_scene::StoreScene>,
}

impl ControllerDraw {
    pub fn new(device: Rc<Graphics>) -> Result<Self> {
        Ok(Self {
            commands: (0..IN_FLIGHT).map(|_| Commands::new(device.clone())).collect::<Result<_>>()?,
            pipeline: ControllerPipeline::new(device.clone())?, targets: HashMap::new(),
            mesh: ControllerMesh::new(device.clone())?,
            store: if std::env::var("HALCYON_FRAME_STORE").as_deref() == Ok("1") {
                Some(crate::store_scene::StoreScene::new(device.clone())?) } else { None },
            device, sequence: 0,
        })
    }

    pub fn draw(&mut self, target: Target, view: &xr::View,
        hands: [Option<xr::Posef>; 2]) -> Result<()> {
        let room = self.store.as_ref().filter(|s| s.active);
        if room.is_none() && hands.iter().all(Option::is_none) { return Ok(()); }
        let slot = self.sequence % IN_FLIGHT;
        let commands = &mut self.commands[slot];
        commands.collect()?;
        let dynamic = room.map(|s| s.upload(slot)).transpose()?.unwrap_or(0);
        if let std::collections::hash_map::Entry::Vacant(entry) = self.targets.entry(target.image) {
            entry.insert((Image::borrowed(self.device.clone(), vk::Image::from_raw(target.image), FORMAT)?,
                ControllerDepth::new(self.device.clone(), target.size)?));
        }
        let (image, depth) = &self.targets[&target.image];
        commands.begin()?;
        let command = commands.command;
        if let Some(room) = self.store.as_mut().filter(|s| s.active) { room.covers.prepare(command); }
        let room = self.store.as_ref().filter(|s| s.active);
        let d = &self.device.api;
        unsafe {
            let barriers = [vk::ImageMemoryBarrier::default().image(image.handle)
                .old_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .new_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .src_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
                .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
                .subresource_range(COLOR)];
            d.cmd_pipeline_barrier(command, vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT, vk::DependencyFlags::empty(),
                &[], &[], &barriers);
            depth.prepare(command);
            commands.timestamp(1);
            let area = vk::Rect2D::default().extent(vk::Extent2D {
                width: target.size.0 as u32, height: target.size.1 as u32 });
            let color = [vk::RenderingAttachmentInfo::default().image_view(image.view)
                .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .load_op(vk::AttachmentLoadOp::LOAD).store_op(vk::AttachmentStoreOp::STORE)];
            let depth_attachment = vk::RenderingAttachmentInfo::default().image_view(depth.view)
                .image_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                .load_op(vk::AttachmentLoadOp::CLEAR).store_op(vk::AttachmentStoreOp::DONT_CARE)
                .clear_value(vk::ClearValue { depth_stencil: vk::ClearDepthStencilValue { depth: 1.0, stencil: 0 } });
            d.cmd_begin_rendering(command, &vk::RenderingInfo::default().render_area(area)
                .layer_count(1).color_attachments(&color).depth_attachment(&depth_attachment));
            d.cmd_set_viewport(command, 0, &[vk::Viewport { x: 0.0, y: target.size.1 as f32,
                width: target.size.0 as f32, height: -(target.size.1 as f32), min_depth: 0.0, max_depth: 1.0 }]);
            d.cmd_set_scissor(command, 0, &[area]);
            d.cmd_bind_pipeline(command, vk::PipelineBindPoint::GRAPHICS, self.pipeline.handle);
            if let Some(room) = room {
                let parameters = parameters(view, &room.navigation.pose);
                let bytes = std::slice::from_raw_parts(parameters.as_ptr() as *const u8, 80);
                d.cmd_push_constants(command, self.pipeline.layout, vk::ShaderStageFlags::VERTEX, 0, bytes);
                d.cmd_bind_vertex_buffers(command, 0, &[room.room.handle], &[0]);
                d.cmd_draw(command, room.count, 1, 0, 0);
                if dynamic > 0 {
                    d.cmd_bind_vertex_buffers(command, 0, &[room.frames[slot].handle], &[0]);
                    d.cmd_draw(command, dynamic, 1, 0, 0);
                }
            }
            d.cmd_bind_vertex_buffers(command, 0, &[self.mesh.buffer.handle], &[0]);
            for (hand, pose) in hands.iter().enumerate() {
                let Some(pose) = pose else { continue; };
                let parameters = parameters(view, pose);
                let bytes = std::slice::from_raw_parts(parameters.as_ptr() as *const u8, 80);
                d.cmd_push_constants(command, self.pipeline.layout, vk::ShaderStageFlags::VERTEX, 0, bytes);
                d.cmd_draw(command, self.mesh.counts[hand], 1,
                    if hand == 0 { 0 } else { self.mesh.counts[0] }, 0);
            }
            if let Some(room) = room { room.covers.draw(command,parameters(view,&room.navigation.pose)); }
            d.cmd_end_rendering(command);
        }
        commands.timestamp(2);
        commands.submit(self.sequence)?;
        self.sequence += 1;
        Ok(())
    }
}

fn parameters(view: &xr::View, hand: &xr::Posef) -> [[f32; 4]; 5] {
    let q = |q: xr::Quaternionf| [q.x, q.y, q.z, q.w];
    let p = |p: xr::Vector3f| [p.x, p.y, p.z, 0.0];
    let f = view.fov;
    [q(view.pose.orientation), p(view.pose.position), q(hand.orientation), p(hand.position),
        [f.angle_left.tan(), f.angle_right.tan(), f.angle_down.tan(), f.angle_up.tan()]]
}
