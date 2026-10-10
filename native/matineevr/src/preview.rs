use crate::{
    presentation::Presentation,
    renderer::{Renderer, Target},
};
use anyhow::Result;
use openxr as xr;
use std::{f32::consts::FRAC_PI_4, time::Instant};

pub const SIZE: [usize; 2] = [256, 144];

pub struct Timing {
    pub ready: Instant,
    pub gpu_ms: f64,
    pub submit_ms: f64,
    pub wait_ms: f64,
}

pub fn draw(renderer: &mut Renderer, image: u64, presentation: Presentation) -> Result<Timing> {
    let vertical = (SIZE[1] as f32 / SIZE[0] as f32).atan();
    let started = Instant::now();
    renderer.draw(
        Target {
            image,
            size: (SIZE[0] as i32, SIZE[1] as i32),
        },
        &xr::View {
            pose: xr::Posef::IDENTITY,
            fov: xr::Fovf {
                angle_left: -FRAC_PI_4,
                angle_right: FRAC_PI_4,
                angle_down: -vertical,
                angle_up: vertical,
            },
        },
        0,
        presentation,
        0.0,
    )?;
    let submitted = Instant::now();
    let (_, times) = renderer.wait()?.unwrap();
    let ready = Instant::now();
    renderer.reset()?;
    Ok(Timing {
        ready,
        gpu_ms: times[1],
        submit_ms: submitted.duration_since(started).as_secs_f64() * 1000.0,
        wait_ms: ready.duration_since(submitted).as_secs_f64() * 1000.0,
    })
}

#[cfg(test)]
#[path = "preview_render_tests.rs"]
mod tests;
