use crate::hud_text::{Canvas, MARGIN, WIDTH, timestamp};
use openxr as xr;
use std::time::{Duration, Instant};

pub const SIZE: [usize; 2] = [WIDTH, 64];
pub const VISIBLE_FOR: Duration = Duration::from_secs(2);
pub const WIDTH_METERS: f32 = 1.2;
pub const POSE: xr::Posef = xr::Posef {
    position: xr::Vector3f {
        x: 0.0,
        y: -0.65,
        z: -1.4,
    },
    ..xr::Posef::IDENTITY
};
const BAR_TOP: usize = 36;
const STROKE: usize = 2;

pub fn visible(until: Option<Instant>, seeking: bool, now: Instant) -> bool {
    until.is_some_and(|until| seeking || now < until)
}

pub fn marker(position: f64, length: Option<f64>) -> Option<usize> {
    length.map(|length| {
        MARGIN
            + ((position / length).clamp(0.0, 1.0) * (WIDTH - 2 * MARGIN - STROKE) as f64) as usize
    })
}

impl Canvas {
    pub fn seek(&mut self, position: f64, length: Option<f64>) {
        self.pixels.fill(0);
        self.line(
            0,
            &format!("{} / {}", timestamp(Some(position)), timestamp(length)),
            [255; 4],
        );
        let [width, height] = self.size;
        let right = width - MARGIN;
        let bottom = height - MARGIN;
        let marker = marker(position, length);
        for y in BAR_TOP..bottom {
            for x in MARGIN..right {
                if x < MARGIN + STROKE
                    || x >= right - STROKE
                    || y < BAR_TOP + STROKE
                    || y >= bottom - STROKE
                    || marker.is_some_and(|marker| (marker..marker + STROKE).contains(&x))
                {
                    let offset = ((height - 1 - y) * width + x) * 4;
                    self.pixels[offset..offset + 4].fill(255);
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "seek_view_tests.rs"]
mod tests;
