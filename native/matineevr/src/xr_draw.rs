use crate::{
    graphics::Graphics,
    options::Options,
    renderer::{Renderer, Target},
    snapshot,
    swapchains::{self, Eye},
};
use anyhow::Result;
use matineevr::{presentation::Presentation, statistics::timed};
use openxr as xr;
use std::rc::Rc;

pub struct Video {
    pub renderer: Renderer,
    pub eyes: Vec<Eye>,
    pub views: Vec<xr::View>,
    controllers: crate::controller_draw::ControllerDraw,
    frames: u64,
    captured: bool,
}

impl Video {
    pub fn update_store(&mut self, active: bool, controls: crate::input::Controls,
        aims: [Option<xr::Posef>; 2]) -> bool {
        self.controllers.store.as_mut().is_some_and(|s| s.update(active, controls, aims))
    }
    pub fn new(
        device: Rc<Graphics>,
        instance: &xr::Instance,
        system: xr::SystemId,
        session: &xr::Session<xr::Vulkan>,
    ) -> Result<Self> {
        Ok(Self {
            renderer: Renderer::new(device.clone())?,
            controllers: crate::controller_draw::ControllerDraw::new(device)?,
            eyes: swapchains::create(instance, system, session)?.1,
            views: Vec::new(),
            frames: 0,
            captured: false,
        })
    }

    pub fn draw(
        &mut self,
        views: Vec<xr::View>,
        presentation: Presentation,
        yaw: f32,
        ready: bool,
        options: &Options,
        hands: [Option<xr::Posef>; 2],
    ) -> Result<()> {
        if ready && !self.renderer.ready() {
            return Ok(());
        }
        for (index, eye) in self.eyes.iter_mut().enumerate() {
            let image = timed(self.renderer.stats.as_mut(), "swapchain_wait_ms", || {
                let image = eye.chain.acquire_image()?;
                eye.chain.wait_image(xr::Duration::INFINITE)?;
                Ok::<_, xr::sys::Result>(image)
            })?;
            let target = Target {
                image: eye.images[image as usize],
                size: (eye.size.width, eye.size.height),
            };
            self.renderer
                .draw(target, &views[index], index as i32, presentation, yaw)?;
            self.controllers.draw(target, &views[index], hands)?;
            if index == 0 && ready && self.frames >= 30 && !self.captured {
                if let Some(path) = &options.snapshot {
                    self.renderer.finish()?;
                    snapshot::save(&self.renderer.device, path, target)?;
                }
                self.captured = true;
            }
            eye.chain.release_image()?;
        }
        self.frames += 1;
        self.views = views;
        Ok(())
    }
}
