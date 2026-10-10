use crate::{
    graphics::Graphics,
    playback::Playback,
    playback_render,
    presentation::Presentation,
    preview,
    renderer::{Completed, Renderer, Target},
    vk_memory::Image,
    vk_pipeline::FORMAT,
    vk_readback,
};
use anyhow::{Result, ensure};
use ash::vk::{self, Handle};
use openxr as xr;
use std::f32::consts::FRAC_PI_4;

pub struct Offscreen {
    pub renderer: Renderer,
    pub preview_renderer: Renderer,
    targets: Vec<Image>,
    size: (i32, i32),
    last: Target,
}

impl Offscreen {
    pub fn new(size: (i32, i32), eyes: usize) -> Result<Self> {
        let instance = Graphics::instance()?;
        let system = instance.system(xr::FormFactor::HEAD_MOUNTED_DISPLAY)?;
        let device = Graphics::new(&instance, system)?;
        ensure!(eyes > 0 && eyes <= 2, "Expected one or two eyes");
        let max = device.limits.max_image_dimension2_d as i32;
        ensure!(
            size.0 > 0 && size.1 > 0 && size.0 <= max && size.1 <= max,
            "Render dimensions must be positive and at most {max}"
        );
        let targets = std::iter::repeat_n(size, eyes)
            .chain([(preview::SIZE[0] as i32, preview::SIZE[1] as i32)])
            .map(|size| {
                Image::new(
                    device.clone(),
                    (size.0 as u32, size.1 as u32),
                    FORMAT,
                    vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_SRC,
                )
            })
            .collect::<Result<Vec<_>>>()?;
        let last = Target {
            image: targets[0].handle.as_raw(),
            size,
        };
        Ok(Self {
            renderer: Renderer::new(device.clone())?,
            preview_renderer: Renderer::new(device)?,
            targets,
            size,
            last,
        })
    }

    pub fn draw(&mut self, presentation: Presentation) -> Result<()> {
        let view = xr::View {
            pose: xr::Posef::IDENTITY,
            fov: xr::Fovf {
                angle_left: -FRAC_PI_4,
                angle_right: FRAC_PI_4,
                angle_up: FRAC_PI_4,
                angle_down: -FRAC_PI_4,
            },
        };
        for (eye, image) in self.targets[..self.targets.len() - 1].iter().enumerate() {
            self.last = Target {
                image: image.handle.as_raw(),
                size: self.size,
            };
            self.renderer
                .draw(self.last, &view, eye as i32, presentation, 0.0)?;
        }
        Ok(())
    }

    pub fn finish(&mut self) -> Result<Vec<Completed>> {
        let mut completed = self.renderer.finish()?;
        completed.extend(self.preview_renderer.finish()?);
        Ok(completed)
    }

    pub fn synchronize(&mut self, player: Option<&mut Playback>) -> Result<()> {
        playback_render::synchronize(
            player,
            &mut [&mut self.renderer, &mut self.preview_renderer],
        )
    }

    pub fn pixels(&mut self) -> Result<Vec<u8>> {
        self.finish()?;
        vk_readback::read(
            &self.renderer.device,
            vk::Image::from_raw(self.last.image),
            self.last.size,
        )
    }

    #[cfg(test)]
    pub(crate) fn pixel(&mut self, [x, y]: [i32; 2]) -> [u8; 4] {
        let offset = ((self.last.size.1 - 1 - y) * self.last.size.0 + x) as usize * 4;
        self.pixels().unwrap()[offset..offset + 4]
            .try_into()
            .unwrap()
    }

    pub fn draw_preview(&mut self, presentation: Presentation) -> Result<preview::Timing> {
        self.last = Target {
            image: self.targets.last().unwrap().handle.as_raw(),
            size: (preview::SIZE[0] as i32, preview::SIZE[1] as i32),
        };
        preview::draw(&mut self.preview_renderer, self.last.image, presentation)
    }
}
