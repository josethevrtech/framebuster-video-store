#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lens {
    pub centers: [[f32; 2]; 2],
    pub distortion: [f32; 4],
}

impl Default for Lens {
    fn default() -> Self {
        Self {
            centers: [[0.5; 2]; 2],
            distortion: [0.0; 4],
        }
    }
}

impl Lens {
    pub fn parameters(self, degrees: u16, eye: usize) -> [f32; 4] {
        let angle = (degrees as f32).to_radians() * 0.5;
        let squared = angle * angle;
        let [k1, k2, k3, k4] = self.distortion;
        let radius =
            angle * (1.0 + squared * (k1 + squared * (k2 + squared * (k3 + squared * k4))));
        [self.centers[eye][0], self.centers[eye][1], angle, radius]
    }
}

#[cfg(test)]
#[path = "fisheye_tests.rs"]
mod tests;
