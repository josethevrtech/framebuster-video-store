use crate::store_geometry::{Vertex, box_mesh};

pub fn signs(v: &mut Vec<Vertex>) {
    for x in [-5.8, 5.8] {
        box_mesh(v, [x, 0.15, 12.88], [1.12, 1.68, 0.06], [0.035, 0.035, 0.04]);
    }
    for x in [-0.90, 0.90] {
        box_mesh(v, [x, 1.73, -1.8], [0.012, 0.52, 0.012], [0.3, 0.32, 0.35]);
    }
}
