use crate::store_geometry::{Vertex, box_mesh};

fn stripe(v: &mut Vec<Vertex>, a: [f32; 2], b: [f32; 2], color: [f32; 3]) {
    let dx = b[0] - a[0]; let dz = b[1] - a[1];
    let length = dx.hypot(dz);
    let side = [-dz / length * 0.022, dx / length * 0.022];
    let points = [[a[0] + side[0], a[1] + side[1]], [a[0] - side[0], a[1] - side[1]],
        [b[0] - side[0], b[1] - side[1]], [b[0] + side[0], b[1] + side[1]]];
    for i in [0, 1, 2, 0, 2, 3] {
        v.push([[points[i][0], -1.495, points[i][1], 1.0], [0.0, 1.0, 0.0, 0.0],
            [color[0], color[1], color[2], 1.0]]);
    }
}

pub fn carpet(v: &mut Vec<Vertex>) {
    box_mesh(v, [crate::store_bounds::CENTER, -1.525, 10.0], [crate::store_bounds::WIDTH, 0.05, 30.0], [0.035, 0.035, 0.09]);
    for x in -16..28 {
        for z in -5..25 {
            let color = [[0.28, 0.07, 0.30], [0.06, 0.29, 0.32],
                [0.31, 0.22, 0.08]][(x + z * 3i32).rem_euclid(3) as usize];
            let p = [x as f32 + 0.15, z as f32 + 0.20];
            let points = [[p[0], p[1]], [p[0] + 0.28, p[1] + 0.22],
                [p[0] + 0.18, p[1] + 0.40], [p[0] + 0.45, p[1] + 0.57]];
            for segment in points.windows(2) { stripe(v, segment[0], segment[1], color); }
            let p = [x as f32 + 0.70, z as f32 + 0.17];
            stripe(v, [p[0] - 0.05, p[1]], [p[0] + 0.05, p[1]], [0.30, 0.10, 0.28]);
            stripe(v, [p[0], p[1] - 0.05], [p[0], p[1] + 0.05], [0.30, 0.10, 0.28]);
        }
    }
}
