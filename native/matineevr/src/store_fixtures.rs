use crate::store_geometry::{Vertex, box_mesh};

pub fn shop(v: &mut Vec<Vertex>) {
    let blue = [0.035, 0.10, 0.22];
    crate::store_carpet::carpet(v);
    let width=crate::store_bounds::WIDTH;
    let center=crate::store_bounds::CENTER;
    for (p, size) in [([center, 0.25, -5.0], [width, 3.5, 0.15]),
        ([-16.0, 0.25, 10.0], [0.15, 3.5, 30.0]),
        ([crate::store_bounds::RIGHT, 0.25, 10.0], [0.15, 3.5, 30.0])] {
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
    box_mesh(v, [center, 2.05, 10.0], [width, 0.10, 30.0], [0.53, 0.54, 0.56]);
    for x in -8..14 {
        for z in -2..13 {
            box_mesh(v, [x as f32 * 2.0 + 1.0, 1.988, z as f32 * 2.0],
                [1.96, 0.02, 1.96], [0.65, 0.65, 0.62]);
        }
    }
    for x in [-14.0, -10.0, -6.0, -2.0, 2.0, 6.0, 10.0, 14.0,18.0,22.0,26.0] {
        for z in [-2.5, 1.5, 5.5, 9.5, 13.5, 17.5, 21.5] {
            box_mesh(v, [x, 1.91, z], [1.5, 0.055, 0.35], [0.82, 0.87, 0.96]);
        }
    }
    crate::store_frontage::append(v);
    crate::store_decor::append(v);
    crate::store_arcade::append(v);
}
