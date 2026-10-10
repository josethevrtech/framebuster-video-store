use crate::presentation::Stereo;

pub const STICK_DEAD_ZONE: f32 = 0.15;
pub const STICK_EXPONENT: f32 = 2.0;
pub const ROTATION_SPEED: f32 = 45.0;
pub const ZOOM_SPEED: f32 = 0.75;
pub const STEREO_SPEED: f32 = 0.02;
pub const POSITION_SPEED: f32 = 0.4;
pub const PITCH_LIMIT: f32 = 90.0;
pub const ZOOM_LIMIT: f32 = 2.0;
pub const STEREO_LIMIT: f32 = 0.1;
pub const POSITION_LIMIT: f32 = 0.5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Orientation,
    Position,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Alignment {
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
    pub zoom_log2: f32,
    pub stereo_offset: [f32; 2],
    pub position: [f32; 2],
}

pub fn response(value: f32) -> f32 {
    value.signum()
        * ((value.abs() - STICK_DEAD_ZONE) / (1.0 - STICK_DEAD_ZONE))
            .clamp(0.0, 1.0)
            .powf(STICK_EXPONENT)
}

impl Alignment {
    pub fn adjust(&mut self, mode: Mode, sticks: [[f32; 2]; 2], seconds: f32, stereo: Stereo) {
        let [[x, y], [rx, ry]] = sticks.map(|stick| stick.map(|v| response(v) * seconds));
        match mode {
            Mode::Orientation => {
                self.yaw = wrap(self.yaw - x * ROTATION_SPEED);
                self.pitch = (self.pitch - y * ROTATION_SPEED).clamp(-PITCH_LIMIT, PITCH_LIMIT);
                self.roll = wrap(self.roll - rx * ROTATION_SPEED);
                self.zoom_log2 = (self.zoom_log2 + ry * ZOOM_SPEED).clamp(-ZOOM_LIMIT, ZOOM_LIMIT);
            }
            Mode::Position => {
                if stereo != Stereo::Mono {
                    for (value, movement) in self.stereo_offset.iter_mut().zip([x, y]) {
                        *value =
                            (*value + movement * STEREO_SPEED).clamp(-STEREO_LIMIT, STEREO_LIMIT);
                    }
                }
                for (value, movement) in self.position.iter_mut().zip([rx, ry]) {
                    *value =
                        (*value - movement * POSITION_SPEED).clamp(-POSITION_LIMIT, POSITION_LIMIT);
                }
            }
        }
    }

    pub fn rotation(self, head: [f32; 4], recenter: f32) -> [f32; 4] {
        let mut rotation = head;
        for (axis, angle) in [
            (1, self.yaw.to_radians() - recenter),
            (0, -self.pitch.to_radians()),
            (2, self.roll.to_radians()),
        ] {
            let (sin, cos) = (angle * 0.5).sin_cos();
            let mut turn = [0.0, 0.0, 0.0, cos];
            turn[axis] = sin;
            rotation = multiply(turn, rotation);
        }
        rotation
    }
}

fn wrap(degrees: f32) -> f32 {
    (degrees + 180.0).rem_euclid(360.0) - 180.0
}

fn multiply([x, y, z, w]: [f32; 4], [a, b, c, d]: [f32; 4]) -> [f32; 4] {
    [
        w * a + x * d + y * c - z * b,
        w * b - x * c + y * d + z * a,
        w * c + x * b - y * a + z * d,
        w * d - x * a - y * b - z * c,
    ]
}

#[cfg(test)]
#[path = "alignment_tests.rs"]
mod tests;
