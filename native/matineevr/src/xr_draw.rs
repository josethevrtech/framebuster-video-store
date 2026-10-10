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
    scene_capture: String,
}

impl Video {
    pub fn store_trailer(&mut self,directory: &std::path::Path) {
        if let Some(store)=self.controllers.store.as_mut() { store.trailer.poll(directory); }
    }
    pub fn store_music(&mut self,path: &std::path::Path) -> Result<()> {
        if let Some(store)=&mut self.controllers.store { store.music_cover.update(path)?; }
        Ok(())
    }
    pub fn store_audio(&self) -> [f32;2] {
        self.controllers.store.as_ref().map_or([0.0;2],|s| s.audio)
    }
    pub fn music_action(&mut self) -> Option<&'static str> {
        self.controllers.store.as_mut().and_then(|s| s.music_action.take())
    }
    pub fn game_action(&mut self) -> Option<(&'static str,usize)> {
        self.controllers.store.as_mut().and_then(|s| s.game_action.take())
    }
    pub fn store_games(&mut self,games: &[crate::store_catalog::Movie],types: &[u8]) -> Result<()> {
        if let Some(store)=self.controllers.store.as_mut() {store.games.update(games,types)?;}
        Ok(())
    }
    pub fn store_covers(&mut self, movies: &[crate::store_catalog::Movie]) -> Result<()> {
        if let Some(store) = &mut self.controllers.store { store.set_catalog(movies)?; }
        Ok(())
    }
    pub fn calibrate_store(&mut self, views: &[xr::View]) -> Result<()> {
        if let Some(store) = &mut self.controllers.store { store.calibrate(views[0].pose.position.y)?; }
        Ok(())
    }

    pub fn store_floor(&mut self, offset: f32) {
        if let Some(store) = &mut self.controllers.store { store.navigation.pose.position.y = offset; }
    }
    pub fn update_store(&mut self, active: bool, controls: crate::input::Controls,
        aims: [Option<xr::Posef>; 2], hands: [Option<xr::Posef>; 2], dt: f32) -> Option<usize> {
        let head = self.views.first().map(|v| v.pose);
        self.controllers.store.as_mut().and_then(|s| s.update(active, controls, aims, hands, head, dt))
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
            scene_capture: String::new(),
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
            if index == 0 && self.frames % 72 == 0 {
                crate::store_capture::capture(&self.renderer.device, target, &mut self.scene_capture)?;
            }
            if index == 0 && (ready || self.controllers.store.as_ref().is_some_and(|s| s.active)) && self.frames >= 30 && !self.captured {
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
