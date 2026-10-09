use crate::store_geometry::{Vertex, box_mesh};

pub const BAYS: [([f32; 3], f32); 10] = [
    ([-5.25, -0.445, -4.72], 0.0), ([-3.15, -0.445, -4.72], 0.0),
    ([-1.05, -0.445, -4.72], 0.0), ([1.05, -0.445, -4.72], 0.0),
    ([3.15, -0.445, -4.72], 0.0), ([5.25, -0.445, -4.72], 0.0),
    ([-3.2, -0.445, 1.76], std::f32::consts::PI), ([-3.2, -0.445, 2.24], 0.0),
    ([3.2, -0.445, 1.76], std::f32::consts::PI), ([3.2, -0.445, 2.24], 0.0),
];

pub fn point(p: [f32; 3], bay: usize) -> [f32; 3] {
    let (center, yaw) = BAYS[bay];
    let (s, c) = yaw.sin_cos();
    [center[0] + c * p[0] + s * p[2], center[1] + p[1], center[2] - s * p[0] + c * p[2]]
}

pub fn card(index: usize) -> [f32; 3] {
    let slot = index % 18;
    point([(slot % 6) as f32 * 0.27 - 0.675, (slot / 6) as f32 * 0.405 - 0.405, -0.055], index / 18)
}

pub fn placed_box(v: &mut Vec<Vertex>, bay: usize, p: [f32; 3], size: [f32; 3], color: [f32; 3]) {
    let start = v.len();
    box_mesh(v, p, size, color);
    let (s, c) = BAYS[bay].1.sin_cos();
    for vertex in &mut v[start..] {
        let p = point([vertex[0][0], vertex[0][1], vertex[0][2]], bay);
        vertex[0][..3].copy_from_slice(&p);
        let [x,y,z,_] = vertex[1];
        vertex[1] = [c*x+s*z, y, -s*x+c*z, 0.0];
    }
}

pub fn shelves(v: &mut Vec<Vertex>) {
    let black = [0.035, 0.035, 0.04];
    for bay in 0..BAYS.len() {
        placed_box(v, bay, [0.0, -0.045, -0.13], [1.98, 1.65, 0.08], black);
        for y in [-0.625, -0.22, 0.185, 0.59] {
            placed_box(v, bay, [0.0, y, -0.055], [1.98, 0.035, 0.25], black);
            placed_box(v, bay, [0.0, y+0.027, 0.08], [1.98, 0.04, 0.022], [0.08,0.08,0.09]);
        }
        for x in [-0.98,0.98] {
            placed_box(v, bay, [x,-0.045,-0.055], [0.035,1.65,0.25], black);
        }
        for slot in 0..18 {
            placed_box(v, bay, [(slot%6) as f32*0.27-0.675, (slot/6) as f32*0.405-0.405, -0.055],
                [0.222,0.374,0.045], black);
        }
    }
}

pub fn hit(origin: [f32;3], direction: [f32;3], index: usize) -> Option<f32> {
    let center=card(index);
    let (s,c)=BAYS[index/18].1.sin_cos();
    let x=origin[0]-center[0]; let z=origin[2]-center[2];
    let p=[c*x-s*z,origin[1]-center[1],s*x+c*z];
    let d=[c*direction[0]-s*direction[2],direction[1],s*direction[0]+c*direction[2]];
    if d[2] >= -0.001 { return None; }
    let t=(0.04-p[2])/d[2];
    (t>0.0 && (p[0]+t*d[0]).abs()<=0.116 && (p[1]+t*d[1]).abs()<=0.19).then_some(t)
}

pub fn outline(v: &mut Vec<Vertex>, index: usize, color: [f32;3]) {
    let slot=index%18;
    let x=(slot%6) as f32*0.27-0.675; let y=(slot/6) as f32*0.405-0.405;
    for dx in [-0.116,0.116] { placed_box(v,index/18,[x+dx,y,0.02],[0.012,0.385,0.012],color); }
    for dy in [-0.1925,0.1925] { placed_box(v,index/18,[x,y+dy,0.02],[0.244,0.012,0.012],color); }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_display_cases_hit_their_own_front_and_keep_catalog_identity() {
        for i in 0..BAYS.len()*18 {
            let p=card(i); let (s,c)=BAYS[i/18].1.sin_cos();
            assert!(hit([p[0]+s,p[1],p[2]+c],[-s,0.0,-c],i).is_some());
            assert!(hit([p[0]+s,p[1],p[2]+c],[s,0.0,c],i).is_none());
            assert!(i%54<54);
        }
    }
}
