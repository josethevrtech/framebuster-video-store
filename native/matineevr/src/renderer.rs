use crate::{
    media::Frame, presentation::Presentation, vk_commands::Commands, vk_device::Device,
    vk_direct::Direct, vk_memory::Image, vk_sync::foreign,
};
use anyhow::Result;
use ash::vk;
use openxr as xr;
use std::{collections::HashMap, rc::Rc, time::Instant};

pub const IN_FLIGHT: usize = 3;
pub type Completed = (usize, [f64; 2]);

#[derive(Clone, Copy)]
pub struct Target {
    pub image: u64,
    pub size: (i32, i32),
}

pub struct Renderer {
    commands: Vec<Commands>,
    pub(crate) targets: HashMap<u64, Image>,
    pub(crate) direct: Option<Direct>,
    pub(crate) current: Option<Rc<Frame>>,
    sequence: usize,
    pub import_ms: f64,
    pub device: Rc<Device>,
    pub stats: Option<crate::statistics::Statistics>,
}

impl Renderer {
    pub fn new(device: Rc<Device>) -> Result<Self> {
        Ok(Self {
            commands: (0..IN_FLIGHT)
                .map(|_| Commands::new(device.clone()))
                .collect::<Result<_>>()?,
            targets: HashMap::new(),
            direct: None,
            current: None,
            sequence: 0,
            import_ms: 0.0,
            device,
            stats: None,
        })
    }

    pub fn ready(&self) -> bool {
        self.current.is_some()
    }

    pub fn upload(&mut self, frame: Rc<Frame>) -> Result<()> {
        let started = Instant::now();
        if self.current.is_none()
            && self
                .direct
                .as_ref()
                .is_none_or(|direct| !direct.compatible(&frame))
        {
            self.direct = Some(Direct::new(
                self.device.clone(),
                frame.dmabuf(),
                &frame,
                self.stats.as_mut(),
            )?);
        }
        self.direct.as_mut().unwrap().prepare(&frame)?;
        self.current = Some(frame);
        self.import_ms = started.elapsed().as_secs_f64() * 1000.0;
        if let Some(stats) = &mut self.stats {
            stats.sample("import_ms", self.import_ms);
        }
        Ok(())
    }

    pub fn draw(
        &mut self,
        target: Target,
        view: &xr::View,
        eye: i32,
        presentation: Presentation,
        yaw: f32,
    ) -> Result<Option<Completed>> {
        let slot = self.sequence % IN_FLIGHT;
        let commands = &mut self.commands[slot];
        let completed = commands.collect_stats(self.stats.as_mut())?;
        let started = Instant::now();
        let image = match (&self.direct, &self.current) {
            (Some(direct), Some(frame)) => Some(direct.bind(frame, slot)),
            _ => None,
        };
        self.import_ms += started.elapsed().as_secs_f64() * 1000.0;
        commands.begin()?;
        commands.timestamp(1);
        let command = commands.command;
        if let Some(image) = image {
            foreign(&self.device, command, image, true);
            commands.frame = self.current.clone();
        }
        if let Some(direct) = self.direct.as_ref().filter(|_| self.current.is_some()) {
            let pipeline = &direct.pipeline;
            unsafe {
                let d = &self.device.api;
                d.cmd_bind_pipeline(command, vk::PipelineBindPoint::GRAPHICS, pipeline.handle);
                d.cmd_bind_descriptor_sets(
                    command,
                    vk::PipelineBindPoint::GRAPHICS,
                    pipeline.layout,
                    0,
                    &[pipeline.descriptors[slot]],
                    &[],
                );
            }
        }
        self.draw_target(command, target, view, eye, presentation, yaw)?;
        let commands = &mut self.commands[slot];
        if let Some(image) = image {
            foreign(&self.device, command, image, false);
        }
        commands.timestamp(2);
        commands.submit(self.sequence)?;
        self.sequence += 1;
        if let Some(stats) = &mut self.stats {
            stats.sample(
                "held_submissions",
                self.commands.iter().filter(|c| c.frame.is_some()).count() as f64,
            );
        }
        Ok(completed)
    }

    pub fn wait(&mut self) -> Result<Option<Completed>> {
        if self.sequence == 0 {
            return Ok(None);
        }
        self.commands[(self.sequence - 1) % IN_FLIGHT].collect_stats(self.stats.as_mut())
    }

    pub fn finish(&mut self) -> Result<Vec<Completed>> {
        self.commands
            .iter_mut()
            .filter_map(|c| c.collect_stats(self.stats.as_mut()).transpose())
            .collect()
    }

    pub fn reset(&mut self) -> Result<Vec<Completed>> {
        let completed = self.finish()?;
        self.current = None;
        if let Some(direct) = &mut self.direct {
            direct.reset();
        }
        Ok(completed)
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        if let Err(error) = self.finish() {
            eprintln!("Vulkan renderer shutdown: {error}");
        }
    }
}
