use crate::store_geometry::Vertex;

pub const SCALE: f32 = 1.0;
pub const FLOOR: f32 = -1.5;
pub const EYE_HEIGHT: f32 = 1.65;

pub fn eye_offset(head_y: f32) -> f32 {
    head_y - EYE_HEIGHT - FLOOR
}
pub fn point(p: [f32; 3], scale: f32) -> [f32; 3] {
    [p[0] * scale, (p[1] - FLOOR) * scale + FLOOR, p[2] * scale]
}
pub fn inverse(p: [f32; 3], d: [f32; 3], scale: f32) -> ([f32; 3], [f32; 3]) {
    ([p[0] / scale, (p[1] - FLOOR) / scale + FLOOR, p[2] / scale],
        d.map(|value| value / scale))
}
pub fn vertices(v: &mut [Vertex], scale: f32) {
    for vertex in v {
        let p = point([vertex[0][0], vertex[0][1], vertex[0][2]], scale);
        vertex[0][..3].copy_from_slice(&p);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scaling_preserves_floor_and_pointer_intersections() {
        assert_eq!(point([0.0, FLOOR, 0.0], SCALE), [0.0, FLOOR, 0.0]);
        let p = [1.2, 0.65, -2.5];
        let (restored, direction) = inverse(point(p, SCALE), [0.0, 0.0, -SCALE], SCALE);
        for i in 0..3 { assert!((restored[i] - p[i]).abs() < 0.00001); }
        assert_eq!(direction, [0.0, 0.0, -1.0]);
        for head_y in [0.0, 1.7, -0.2] {
            assert!((head_y - (eye_offset(head_y) + FLOOR) - EYE_HEIGHT).abs() < 0.00001);
        }
    }
}
