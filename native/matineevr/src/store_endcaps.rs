use crate::store_geometry::{Vertex, box_mesh};

pub const POSTERS: [([f32; 3], f32); 4] = [
    ([-4.235, -0.445, 2.0], -std::f32::consts::FRAC_PI_2),
    ([-2.165, -0.445, 2.0], std::f32::consts::FRAC_PI_2),
    ([2.165, -0.445, 2.0], -std::f32::consts::FRAC_PI_2),
    ([4.235, -0.445, 2.0], std::f32::consts::FRAC_PI_2),
];

pub fn geometry(v: &mut Vec<Vertex>) {
    for &(p, yaw) in &POSTERS {
        let normal = yaw.sin();
        box_mesh(v, [p[0] - normal * 0.025, p[1] - 0.045, p[2]],
            [0.04, 1.65, 0.76], [0.035, 0.035, 0.04]);
    }
}
