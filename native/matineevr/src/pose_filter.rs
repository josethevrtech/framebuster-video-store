use openxr as xr;
use std::f32::consts::TAU;

const POSITION_CUTOFF: f32 = 4.0;
const POSITION_BETA: f32 = 20.0;
const ROTATION_CUTOFF: f32 = 4.0;
const ROTATION_BETA: f32 = 4.0;
const DERIVATIVE_CUTOFF: f32 = 1.0;
const MAX_SAMPLE_GAP: f32 = 0.25;

#[derive(Default)]
pub struct PoseFilter {
    previous: Option<Sample>,
}

struct Sample {
    time: xr::Time,
    position: [f32; 3],
    rotation: [f32; 4],
    raw_position: [f32; 3],
    raw_rotation: [f32; 4],
    velocity: [f32; 3],
    angular_velocity: [f32; 3],
}

impl PoseFilter {
    pub fn update(&mut self, pose: Option<xr::Posef>, time: xr::Time) -> Option<xr::Posef> {
        let Some(pose) = pose else {
            self.previous = None;
            return None;
        };
        let p = pose.position;
        let position = [p.x, p.y, p.z];
        let rotation = normalize(quaternion(pose.orientation));
        let mut next = Sample {
            time,
            position,
            rotation,
            raw_position: position,
            raw_rotation: rotation,
            velocity: [0.0; 3],
            angular_velocity: [0.0; 3],
        };
        if let Some(previous) = &self.previous {
            let dt = (time - previous.time).as_nanos() as f32 * 1e-9;
            if dt > 0.0 && dt <= MAX_SAMPLE_GAP {
                let velocity =
                    std::array::from_fn(|i| (position[i] - previous.raw_position[i]) / dt);
                next.velocity = mix(previous.velocity, velocity, alpha(DERIVATIVE_CUTOFF, dt));
                next.angular_velocity = mix(
                    previous.angular_velocity,
                    angular_velocity(previous.raw_rotation, rotation, dt),
                    alpha(DERIVATIVE_CUTOFF, dt),
                );
                next.position = mix(
                    previous.position,
                    position,
                    alpha(POSITION_CUTOFF + POSITION_BETA * length(next.velocity), dt),
                );
                next.rotation = interpolate(
                    previous.rotation,
                    rotation,
                    alpha(
                        ROTATION_CUTOFF + ROTATION_BETA * length(next.angular_velocity),
                        dt,
                    ),
                );
            }
        }
        let result = make_pose(next.position, next.rotation);
        self.previous = Some(next);
        Some(result)
    }
}

fn alpha(cutoff: f32, dt: f32) -> f32 {
    1.0 / (1.0 + 1.0 / (TAU * cutoff * dt))
}

fn mix<const N: usize>(a: [f32; N], b: [f32; N], weight: f32) -> [f32; N] {
    std::array::from_fn(|i| a[i] + weight * (b[i] - a[i]))
}

fn length<const N: usize>(value: [f32; N]) -> f32 {
    value.iter().map(|v| v * v).sum::<f32>().sqrt()
}

fn normalize(value: [f32; 4]) -> [f32; 4] {
    let norm = length(value);
    value.map(|v| v / norm)
}

fn quaternion(q: xr::Quaternionf) -> [f32; 4] {
    [q.x, q.y, q.z, q.w]
}

fn make_pose([x, y, z]: [f32; 3], rotation: [f32; 4]) -> xr::Posef {
    let [qx, qy, qz, w] = rotation;
    xr::Posef {
        position: xr::Vector3f { x, y, z },
        orientation: xr::Quaternionf {
            x: qx,
            y: qy,
            z: qz,
            w,
        },
    }
}

fn conjugate([x, y, z, w]: [f32; 4]) -> [f32; 4] {
    [-x, -y, -z, w]
}

fn multiply([ax, ay, az, aw]: [f32; 4], [bx, by, bz, bw]: [f32; 4]) -> [f32; 4] {
    [
        aw * bx + ax * bw + ay * bz - az * by,
        aw * by - ax * bz + ay * bw + az * bx,
        aw * bz + ax * by - ay * bx + az * bw,
        aw * bw - ax * bx - ay * by - az * bz,
    ]
}

fn angular_velocity(previous: [f32; 4], current: [f32; 4], dt: f32) -> [f32; 3] {
    let mut delta = multiply(current, conjugate(previous));
    if delta[3] < 0.0 {
        delta = delta.map(|v| -v);
    }
    let axis = [delta[0], delta[1], delta[2]];
    let sine = length(axis);
    let scale = if sine > 1e-6 {
        2.0 * sine.atan2(delta[3]) / sine
    } else {
        2.0
    };
    axis.map(|v| v * scale / dt)
}

fn interpolate(a: [f32; 4], mut b: [f32; 4], weight: f32) -> [f32; 4] {
    let mut dot: f32 = a.iter().zip(b).map(|(a, b)| a * b).sum();
    if dot < 0.0 {
        b = b.map(|v| -v);
        dot = -dot;
    }
    if dot > 0.9995 {
        return normalize(mix(a, b, weight));
    }
    let angle = dot.clamp(0.0, 1.0).acos();
    let left = ((1.0 - weight) * angle).sin();
    let right = (weight * angle).sin();
    normalize(std::array::from_fn(|i| left * a[i] + right * b[i]))
}

#[cfg(test)]
#[path = "pose_filter_tests.rs"]
mod tests;
