use crate::{graphics::Graphics, input::Controls, store_geometry::{self, Vertex, CARDS}};
use anyhow::{Result, ensure};
use matineevr::{renderer::IN_FLIGHT, vk_memory::Buffer};
use openxr as xr;
use std::rc::Rc;

pub struct StoreScene {
    pub active: bool,
    pub room: Buffer,
    pub count: u32,
    pub frames: Vec<Buffer>,
    dynamic: Vec<Vertex>,
    held: [bool; 2],
    selected: Option<usize>,
    pub navigation: crate::store_navigation::StoreNavigation,
}

impl StoreScene {
    pub fn new(device: Rc<Graphics>) -> Result<Self> {
        let vertices = store_geometry::room();
        let room = Buffer::new(device.clone(), vertices.len() * 48)?;
        upload(&room, &vertices)?;
        Ok(Self { active: true, room, count: vertices.len() as u32,
            frames: (0..IN_FLIGHT).map(|_| Buffer::new(device.clone(), 64 * 1024)).collect::<Result<_>>()?,
            dynamic: Vec::new(), held: [false; 2], selected: None, navigation: Default::default() })
    }

    pub fn update(&mut self, active: bool, controls: Controls, aims: [Option<xr::Posef>; 2]) -> Option<usize> {
        self.active = active;
        self.dynamic.clear();
        let mut launch = false;
        if active { self.navigation.update(controls); }
        for (hand, pose) in aims.into_iter().enumerate() {
            let pressed = controls.triggers[hand] > 0.65;
            if active && let Some(pose) = pose {
                let (origin, direction) = ray(pose);
                let (origin, direction) = self.navigation.inverse_ray(origin, direction);
                let hit = CARDS.iter().enumerate().filter_map(|(i, center)|
                    hit_card(origin, direction, *center).map(|t| (i, t)))
                    .min_by(|a, b| a.1.total_cmp(&b.1));
                let color = if hit.is_some() { [1.0, 0.65, 0.12] } else { [0.25, 0.65, 1.0] };
                store_geometry::beam(&mut self.dynamic, origin, direction, hit.map_or(4.0, |h| h.1), color);
                if let Some((i, _)) = hit {
                    outline(&mut self.dynamic, CARDS[i], color);
                    if pressed && !self.held[hand] { self.selected = Some(i); launch = true; }
                }
            }
            self.held[hand] = pressed;
        }
        if active && let Some(i) = self.selected { outline(&mut self.dynamic, CARDS[i], [0.2, 1.0, 0.4]); }
        (active && (launch || controls.a)).then_some(self.selected.unwrap_or(0))
    }

    pub fn upload(&self, slot: usize) -> Result<u32> {
        upload(&self.frames[slot], &self.dynamic)?;
        Ok(self.dynamic.len() as u32)
    }
}

fn upload(buffer: &Buffer, vertices: &[Vertex]) -> Result<()> {
    let bytes = std::mem::size_of_val(vertices);
    ensure!(bytes <= buffer.size, "Store geometry exceeds buffer capacity");
    unsafe { std::ptr::copy_nonoverlapping(vertices.as_ptr() as *const u8, buffer.pointer, bytes); }
    Ok(())
}

fn outline(v: &mut Vec<Vertex>, p: [f32; 3], color: [f32; 3]) {
    for x in [-0.32, 0.32] {
        store_geometry::box_mesh(v, [p[0] + x, p[1], p[2] + 0.075], [0.025, 0.79, 0.025], color);
    }
    for y in [-0.395, 0.395] {
        store_geometry::box_mesh(v, [p[0], p[1] + y, p[2] + 0.075], [0.665, 0.025, 0.025], color);
    }
}

fn ray(p: xr::Posef) -> ([f32; 3], [f32; 3]) {
    let q = p.orientation;
    ([p.position.x, p.position.y, p.position.z],
        [-2.0 * (q.x * q.z + q.w * q.y), 2.0 * (q.w * q.x - q.y * q.z),
            -1.0 + 2.0 * (q.x * q.x + q.y * q.y)])
}

fn hit_card(origin: [f32; 3], direction: [f32; 3], center: [f32; 3]) -> Option<f32> {
    if direction[2] >= -0.001 { return None; }
    let t = (center[2] + 0.04 - origin[2]) / direction[2];
    (t > 0.0 && (origin[0] + t * direction[0] - center[0]).abs() <= 0.30
        && (origin[1] + t * direction[1] - center[1]).abs() <= 0.37).then_some(t)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pointing_hits_only_cards_in_front() {
        assert!(hit_card([0.0, -0.25, 0.0], [0.0, 0.0, -1.0], CARDS[1]).is_some());
        assert!(hit_card([0.0, -0.25, 0.0], [0.0, 0.0, 1.0], CARDS[1]).is_none());
        assert!(hit_card([0.0, 1.5, 0.0], [0.0, 0.0, -1.0], CARDS[1]).is_none());
        let (_, d) = ray(xr::Posef::IDENTITY);
        assert_eq!(d, [0.0, 0.0, -1.0]);
        assert!(store_geometry::room().iter().flatten().flatten().all(|v| v.is_finite()));
    }
}
