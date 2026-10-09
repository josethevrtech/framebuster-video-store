use crate::store_geometry::{Vertex, box_mesh};

pub fn shop(v: &mut Vec<Vertex>) {
    let blue = [0.035, 0.10, 0.22];
    let wood = [0.035, 0.035, 0.04];
    crate::store_carpet::carpet(v);
    for (p, size) in [([0.0, 0.25, -5.0], [20.0, 3.5, 0.15]),
        ([-10.0, 0.25, 6.0], [0.15, 3.5, 22.0]),
        ([10.0, 0.25, 6.0], [0.15, 3.5, 22.0])] {
        box_mesh(v, p, size, blue);
        let side = size[0] < size[2];
        let band = if side { [p[0] - p[0].signum() * 0.11, 1.35, p[2]] }
            else { [p[0], 1.35, p[2] - p[2].signum() * 0.11] };
        box_mesh(v, band, if side { [0.025, 0.18, size[2] - 0.25] }
            else { [size[0] - 0.25, 0.18, 0.025] }, [0.025, 0.025, 0.03]);
        let length = if side { size[2] } else { size[0] };
        for i in 0..(length / 0.22) as usize {
            for y in [1.29, 1.41] {
                let mut p = band;
                p[1] = y;
                p[if side { 2 } else { 0 }] += -length / 2.0 + 0.22 * i as f32 + 0.11;
                p[if side { 0 } else { 2 }] -= p[if side { 0 } else { 2 }].signum() * 0.019;
                box_mesh(v, p, if side { [0.01, 0.035, 0.055] } else { [0.055, 0.035, 0.01] }, [0.65, 0.58, 0.43]);
            }
        }
    }
    box_mesh(v, [0.0, 2.05, 6.0], [20.0, 0.10, 22.0], [0.53, 0.54, 0.56]);
    for x in -5..5 {
        for z in -2..9 {
            box_mesh(v, [x as f32 * 2.0 + 1.0, 1.988, z as f32 * 2.0],
                [1.96, 0.02, 1.96], [0.65, 0.65, 0.62]);
        }
    }
    for x in [-5.2, 5.2] {
        for z in [1.0, 5.5] {
            box_mesh(v, [x, -0.65, z], [0.75, 1.65, 2.5], wood);
            for y in [-1.15, -0.65, -0.15, 0.25] {
                for side in [-1.0, 1.0] {
                    box_mesh(v, [x + side * 0.56, y + 0.035, z],
                        [0.035, 0.025, 2.65], wood);
                }
            }
            for (row, y) in [-1.15, -0.65, -0.15].into_iter().enumerate() {
                for tape in 0..10 {
                    let color = [[0.13, 0.26, 0.44], [0.40, 0.11, 0.10],
                        [0.19, 0.30, 0.19], [0.32, 0.18, 0.35]][(tape + row) % 4];
                    for side in [-1.0, 1.0] {
                        let p = [x + side * 0.51, y + 0.20, z - 1.10 + tape as f32 * 0.23];
                        box_mesh(v, p, [0.07, 0.32, 0.19], color);
                        box_mesh(v, [p[0] + side * 0.10, p[1] - 0.08, p[2]],
                            [0.014, 0.05, 0.15], [0.79, 0.74, 0.56]);
                    }
                }
            }
        }
    }
    for x in [-6.0, -2.0, 2.0, 6.0] {
        for z in [-2.5, 1.5, 5.5, 9.5, 13.5] {
            box_mesh(v, [x, 1.91, z], [1.5, 0.055, 0.35], [0.82, 0.87, 0.96]);
        }
    }
    crate::store_frontage::append(v);
    crate::store_decor::append(v);
}
