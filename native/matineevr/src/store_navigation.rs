use crate::input::Controls;
use openxr as xr;

pub struct StoreNavigation {
    pub pose: xr::Posef,
    held: [bool; 2],
    yaw: f32,
    pub scale: f32,
}
impl Default for StoreNavigation {
    fn default() -> Self { Self { pose: xr::Posef::IDENTITY, held: [false; 2], yaw: 0.0, scale: 1.0 } }
}
impl StoreNavigation {
    pub fn update(&mut self, controls: Controls) {
        let [x, y] = controls.sticks[0];
        let pressed = [x.abs() > 0.7, y.abs() > 0.7];
        if pressed[0] && !self.held[0] {
            let angle = -x.signum() * std::f32::consts::PI / 6.0;
            let (s, c) = angle.sin_cos();
            let p = &mut self.pose.position;
            (p.x, p.z) = (c * p.x + s * p.z, -s * p.x + c * p.z);
            self.yaw += angle;
            let (s, c) = (self.yaw / 2.0).sin_cos();
            self.pose.orientation = xr::Quaternionf { x: 0.0, y: s, z: 0.0, w: c };
        }
        if pressed[1] && !self.held[1] {
            self.pose.position.z += y.signum() * 0.6 * self.scale;
            let (p, _) = self.inverse_ray([0.0; 3], [0.0; 3]);
            let (s, c) = self.yaw.sin_cos();
            let x = p[0].clamp(-2.3 * self.scale, 2.3 * self.scale);
            let z = p[2].clamp(-1.3 * self.scale, 4.8 * self.scale);
            self.pose.position.x = -c * x - s * z;
            self.pose.position.z = s * x - c * z;
        }
        self.held = pressed;
    }
    pub fn inverse_ray(&self, p: [f32; 3], d: [f32; 3]) -> ([f32; 3], [f32; 3]) {
        let (s, c) = self.yaw.sin_cos();
        let x = p[0] - self.pose.position.x; let z = p[2] - self.pose.position.z;
        ([c * x - s * z, p[1] - self.pose.position.y, s * x + c * z],
            [c * d[0] - s * d[2], d[1], s * d[0] + c * d[2]])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn movement_requires_release_and_stays_bounded() {
        let mut navigation = StoreNavigation::default();
        let mut held = Controls::default(); held.sticks[0] = [0.0, 1.0];
        navigation.update(held); navigation.update(held);
        assert!((navigation.pose.position.z - 0.6).abs() < 0.001);
        for _ in 0..10 { navigation.update(Controls::default()); navigation.update(held); }
        assert!(navigation.pose.position.z <= 1.3);
        let (p, _) = navigation.inverse_ray([0.0, 0.0, navigation.pose.position.z], [0.0, 0.0, -1.0]);
        assert_eq!(p, [0.0, 0.0, 0.0]);
    }
}
