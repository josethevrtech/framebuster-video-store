pub type Vertex = [[f32; 4]; 3];

pub const CARDS: [[f32; 3]; 6] = [
    [-1.2, -0.25, -2.5], [0.0, -0.25, -2.5], [1.2, -0.25, -2.5],
    [-1.2, 0.65, -2.5], [0.0, 0.65, -2.5], [1.2, 0.65, -2.5],
];

pub fn box_mesh(vertices: &mut Vec<Vertex>, center: [f32; 3], size: [f32; 3], color: [f32; 3]) {
    for axis in 0..3 {
        let a = (axis + 1) % 3;
        let b = (axis + 2) % 3;
        for sign in [-1.0, 1.0] {
            let mut normal = [0.0; 4]; normal[axis] = sign;
            let points: [_; 4] = std::array::from_fn(|i| {
                let mut p = [center[0], center[1], center[2], 1.0];
                p[axis] += sign * size[axis] / 2.0;
                p[a] += [-1.0, 1.0, 1.0, -1.0][i] * size[a] / 2.0;
                p[b] += [-1.0, -1.0, 1.0, 1.0][i] * size[b] / 2.0;
                p
            });
            for i in [0, 1, 2, 0, 2, 3] {
                vertices.push([points[i], normal, [color[0], color[1], color[2], 1.0]]);
            }
        }
    }
}

pub fn room() -> Vec<Vertex> {
    let mut v = Vec::new();
    crate::store_fixtures::shop(&mut v);
    for y in [-0.7, 0.2, 1.1] {
        box_mesh(&mut v, [0.0, y, -2.55], [4.2, 0.06, 0.45], [0.22, 0.12, 0.045]);
        box_mesh(&mut v, [0.0, y + 0.04, -2.30], [4.2, 0.025, 0.025], [0.7, 0.38, 0.045]);
    }
    for x in [-2.2, 2.2] {
        box_mesh(&mut v, [x, 0.2, -2.55], [0.12, 2.0, 0.5], [0.14, 0.08, 0.035]);
    }
    for (i, center) in CARDS.into_iter().enumerate() {
        box_mesh(&mut v, center, [0.58, 0.72, 0.07],
            [[0.07, 0.33, 0.60], [0.62, 0.21, 0.08], [0.19, 0.46, 0.22]][i % 3]);
        box_mesh(&mut v, [center[0], center[1] - 0.23, center[2] + 0.05],
            [0.43, 0.04, 0.018], [0.85, 0.73, 0.40]);
    }
    v
}

pub fn beam(v: &mut Vec<Vertex>, origin: [f32; 3], direction: [f32; 3], distance: f32, color: [f32; 3]) {
    for axis in [0, 1] {
        let mut points = [[0.0; 4]; 4];
        for (i, p) in points.iter_mut().enumerate() {
            let t = if i < 2 { 0.08 } else { distance };
            *p = [origin[0] + direction[0] * t, origin[1] + direction[1] * t,
                origin[2] + direction[2] * t, 1.0];
            p[axis] += if i % 2 == 0 { -0.004 } else { 0.004 };
        }
        for i in [0, 1, 2, 1, 3, 2] {
            v.push([points[i], [0.0, 0.0, 1.0, 0.0], [color[0], color[1], color[2], 1.0]]);
        }
    }
}
