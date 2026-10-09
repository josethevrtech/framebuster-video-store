use crate::{graphics::Graphics, hud_text::WIDTH, panel::Panel};
use anyhow::Result;
use openxr as xr;
use std::rc::Rc;

const SIZE: usize = 128;
const WIDTH_METERS: f32 = 0.09;

pub struct ControllerVisuals {
    panels: [Panel; 2],
}

impl ControllerVisuals {
    pub fn new(session: &xr::Session<xr::Vulkan>, graphics: Rc<Graphics>) -> Result<Self> {
        let mut panels = [
            Panel::new(session, graphics.clone(), [SIZE, SIZE])?,
            Panel::new(session, graphics, [SIZE, SIZE])?,
        ];
        for (hand, panel) in panels.iter_mut().enumerate() {
            panel.upload(&pixels(hand))?;
        }
        Ok(Self { panels })
    }

    pub fn layers<'a>(
        &'a self,
        space: &'a xr::Space,
        hands: [Option<xr::Posef>; 2],
        facing: xr::Quaternionf,
    ) -> Vec<xr::CompositionLayerQuad<'a, xr::Vulkan>> {
        hands.iter().enumerate().filter_map(|(hand, pose)| {
            let pose = pose.as_ref()?;
            Some(self.panels[hand].layer(space, xr::Posef {
                orientation: facing,
                position: pose.position,
            }, WIDTH_METERS * WIDTH as f32 / SIZE as f32))
        }).collect()
    }
}

fn pixels(hand: usize) -> Vec<u8> {
    let mut pixels = vec![0; SIZE * SIZE * 4];
    let accent = if hand == 0 { [63, 208, 255, 255] } else { [255, 177, 75, 255] };
    for y in 0..SIZE {
        for x in 0..SIZE {
            let dx = x as i32 - 64;
            let dy = y as i32 - 39;
            let ring = (24 * 24..=33 * 33).contains(&(dx * dx + dy * dy));
            let body = (45..83).contains(&x) && (45..116).contains(&y);
            let stick = (x as i32 - 64).pow(2) + (y as i32 - 56).pow(2) < 8 * 8;
            let button = (x as i32 - 64).pow(2) + (y as i32 - 82).pow(2) < 5 * 5;
            let color = if ring || stick || button { accent }
                else if body { [35, 44, 59, 255] }
                else { [0; 4] };
            pixels[(y * SIZE + x) * 4..(y * SIZE + x + 1) * 4].copy_from_slice(&color);
        }
    }
    pixels
}
