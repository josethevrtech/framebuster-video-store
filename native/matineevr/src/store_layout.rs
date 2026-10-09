use crate::store_geometry::{Vertex, box_mesh};

pub const EXTRA_RACKS: [[f32; 2]; 6] = [
    [-3.4, 5.3], [3.4, 5.3], [-3.4, 8.1], [3.4, 8.1], [-7.4, 8.1], [7.4, 8.1],
];

pub fn obstacles() -> Vec<[f32; 4]> {
    let mut boxes = vec![[-4.25, -2.15, 1.55, 2.45], [2.15, 4.25, 1.55, 2.45],
        [-2.6, 2.6, 14.35, 15.9], [6.35, 7.25, 15.55, 16.35], [-9.35,-8.2,10.85,13.75],
        [8.3,9.3,11.65,12.85], [-4.3,-3.3,13.85,14.75], [3.3,4.3,13.85,14.75]];
    for x in [-5.2, 5.2] {
        for z in [1.0, 5.5] { boxes.push([x - 0.68, x + 0.68, z - 1.4, z + 1.4]); }
    }
    for [x, z] in EXTRA_RACKS { boxes.push([x - 1.04, x + 1.04, z - 0.39, z + 0.39]); }
    boxes
}

pub fn free(p: [f32; 3], radius: f32) -> bool {
    if p[0].abs() > 9.75 - radius || p[2] < -4.4 + radius || p[2] > 16.7 - radius { return false; }
    !obstacles().iter().any(|b| {
        let x = p[0] - p[0].clamp(b[0], b[1]);
        let z = p[2] - p[2].clamp(b[2], b[3]);
        x*x + z*z < radius*radius
    })
}

pub fn append(v: &mut Vec<Vertex>) {
    let black = [0.035, 0.035, 0.04];
    for [x, z] in EXTRA_RACKS {
        box_mesh(v, [x, -0.49, z], [2.02, 1.65, 0.09], black);
        box_mesh(v, [x, -1.36, z], [2.06, 0.16, 0.72], black);
        for dx in [-1.01, 1.01] { box_mesh(v, [x+dx, -0.49, z], [0.05, 1.65, 0.72], black); }
        for y in [-1.10, -0.695, -0.29, 0.115] {
            box_mesh(v, [x, y, z], [2.02, 0.04, 0.72], black);
            for side in [-1.0, 1.0] {
                box_mesh(v, [x, y+0.026, z+side*0.36], [2.02, 0.04, 0.025], [0.08, 0.08, 0.09]);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn aisles_are_open_but_shelves_and_checkout_block_walking() {
        for z in [0.0, 2.0, 5.3, 8.3, 10.0] { assert!(free([0.0, 0.15, z], 0.22)); }
        for [x,z] in EXTRA_RACKS { assert!(!free([x, 0.15, z], 0.22)); }
        for x in [-6.0, -3.0, 0.0, 3.0, 6.0] {
            for z in [10.0, 11.0, 12.0, 13.0] { assert!(free([x,0.15,z],0.22)); }
        }
        assert!(!free([0.0,0.15,15.0],0.22));
    }
}
