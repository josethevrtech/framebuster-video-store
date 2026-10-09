use anyhow::{Result, ensure};
use ash::vk;
use matineevr::media::Frame;
use matineevr::{
    vk_commands::Commands, vk_device::Device, vk_import::Imported, vk_memory::Buffer,
    vk_sample::Sampling, vk_sync::foreign,
};
use std::rc::Rc;

pub fn read(
    device: Rc<Device>,
    frame: &Frame,
    color: bool,
    pending: impl FnOnce() -> Result<()>,
) -> Result<Vec<f32>> {
    let source = frame.dmabuf();
    let width = source.width - source.crop[2];
    let height = source.height - source.crop[3];
    let imported = Imported::new(device.clone(), &source)?;
    let sampling = Rc::new(Sampling::new(
        device.clone(),
        &imported,
        color.then_some(&frame.pixels),
        source.chroma_location,
    )?);
    let view = sampling.view(imported)?;
    let rows =
        height.min((device.limits.max_storage_buffer_range as u64 / width as u64 / 16) as i32);
    ensure!(
        rows > 0,
        "Verification row exceeds the storage-buffer limit"
    );
    let buffer = Buffer::new(device.clone(), width as usize * rows as usize * 16)?;
    let (pipeline, set) = super::dmabuf_pipeline::Pipeline::new(device.clone(), &view, &buffer)?;
    let mut commands = Commands::new(device.clone())?;
    let gate = Gate::new(device.clone())?;
    let mut pending = Some(pending);
    let mut output = Vec::with_capacity(width as usize * height as usize * 4);
    for row in (0..height).step_by(rows as usize) {
        let count = rows.min(height - row);
        unsafe {
            let d = &device.api;
            commands.begin()?;
            d.cmd_wait_events(
                commands.command,
                &[gate.event],
                vk::PipelineStageFlags::HOST,
                vk::PipelineStageFlags::COMPUTE_SHADER,
                &[],
                &[],
                &[],
            );
            foreign(&device, commands.command, view.imported.handle, true);
            commands.timestamp(1);
            d.cmd_bind_pipeline(
                commands.command,
                vk::PipelineBindPoint::COMPUTE,
                pipeline.handle,
            );
            d.cmd_bind_descriptor_sets(
                commands.command,
                vk::PipelineBindPoint::COMPUTE,
                pipeline.layout,
                0,
                &[set],
                &[],
            );
            let size = [width, height, row, count].map(i32::to_ne_bytes).concat();
            d.cmd_push_constants(
                commands.command,
                pipeline.layout,
                vk::ShaderStageFlags::COMPUTE,
                0,
                &size,
            );
            d.cmd_dispatch(
                commands.command,
                (width as u32).div_ceil(8),
                (count as u32).div_ceil(8),
                1,
            );
            foreign(&device, commands.command, view.imported.handle, false);
            d.cmd_pipeline_barrier(
                commands.command,
                vk::PipelineStageFlags::COMPUTE_SHADER,
                vk::PipelineStageFlags::HOST,
                vk::DependencyFlags::empty(),
                &[vk::MemoryBarrier::default()
                    .src_access_mask(vk::AccessFlags::SHADER_WRITE)
                    .dst_access_mask(vk::AccessFlags::HOST_READ)],
                &[],
                &[],
            );
            commands.timestamp(2);
            commands.submit(0)?;
            let result = pending.take().map_or(Ok(()), |run| run());
            gate.open()?;
            commands.collect()?;
            result?;
            let pixels = std::slice::from_raw_parts(
                buffer.pointer as *const f32,
                width as usize * count as usize * 4,
            );
            output.extend_from_slice(pixels);
        }
    }
    Ok(output)
}

pub struct Gate {
    pub event: vk::Event,
    device: Rc<Device>,
}

impl Gate {
    pub fn new(device: Rc<Device>) -> Result<Self> {
        let event = unsafe {
            device
                .api
                .create_event(&vk::EventCreateInfo::default(), None)?
        };
        Ok(Self { event, device })
    }

    pub fn open(&self) -> Result<()> {
        Ok(unsafe { self.device.api.set_event(self.event) }?)
    }
}

impl Drop for Gate {
    fn drop(&mut self) {
        unsafe {
            if let Err(error) = self.device.api.set_event(self.event) {
                eprintln!("Release GPU verification gate: {error}");
            }
            if let Err(error) = self.device.api.device_wait_idle() {
                eprintln!("Wait for GPU verification: {error}");
            }
            self.device.api.destroy_event(self.event, None);
        }
    }
}
