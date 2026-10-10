use crate::store_geometry::{Vertex, box_mesh};

pub const EXTRA_RACKS: [[f32; 2]; 12] = [
    [-11.0,6.0],[-6.5,6.0],[-2.0,6.0],[2.5,6.0],
    [-11.0,10.0],[-6.5,10.0],[-2.0,10.0],[2.5,10.0],
    [-11.0,14.0],[-6.5,14.0],[-2.0,14.0],[2.5,14.0],
];

pub fn obstacles() -> Vec<[f32; 4]> {
    let mut boxes = vec![[-4.25, -2.15, 1.55, 2.45], [2.15, 4.25, 1.55, 2.45],
        [6.35,7.25,23.55,24.35],
        [-4.15,-3.85,23.54,23.87],[3.85,4.15,23.54,23.87]];
    boxes.extend(crate::store_arcade::obstacles());
    boxes.extend(crate::store_game_racks::obstacles());
    boxes.extend(crate::store_album_racks::obstacles());
    for [x, z] in EXTRA_RACKS { boxes.push([x - 1.04, x + 1.04, z - 0.39, z + 0.39]); }
    boxes
}

pub fn free(p: [f32; 3], radius: f32) -> bool {
    if p[0]<crate::store_bounds::LEFT+0.25+radius || p[0]>crate::store_bounds::RIGHT-0.25-radius
        || p[2] < -4.4 + radius || p[2] > 24.7 - radius { return false; }
    !crate::store_collision::blocked([p[0],p[2]],radius) && !obstacles().iter().any(|b| {
        let x = p[0] - p[0].clamp(b[0], b[1]);
        let z = p[2] - p[2].clamp(b[2], b[3]);
        x*x + z*z < radius*radius
    })
}

pub fn append(v: &mut Vec<Vertex>) {
    crate::store_game_racks::append(v);
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
        for z in [0.0,2.0,6.0,10.0,14.0,21.0,23.0,24.0] { assert!(free([0.0,0.15,z],0.22)); }
        for [x,z] in EXTRA_RACKS { assert!(!free([x,0.15,z],0.22)); }
        for x in [-6.0,-3.0,6.0] {
            for z in [18.0,19.0,20.0,21.0] { assert!(free([x,0.15,z],0.22)); }
        }
        assert!(!free([0.0,0.15,17.0],0.22));
        assert!(!free([0.0,0.15,18.0],0.22));
        assert!(free([0.0,0.15,20.0],0.22));
        assert!(free([-12.8,0.15,20.2],0.22));
        assert!(free([-3.8,0.15,22.3],0.22));
        assert!(free([3.8,0.15,22.3],0.22));
        for z in [5.0,9.0,13.0,17.0,20.0] { assert!(free([9.0,0.15,z],0.22)); }
    }
    #[test]
    fn rental_racks_never_intersect_or_pinching_the_aisles() {
        let mut racks = vec![[-3.2,2.0],[3.2,2.0]]; racks.extend(EXTRA_RACKS);
        for (i,a) in racks.iter().enumerate() {
            for b in &racks[i+1..] {
                assert!((a[0]-b[0]).abs() >= 3.0 || (a[1]-b[1]).abs() >= 1.8);
            }
        }
    }
}
