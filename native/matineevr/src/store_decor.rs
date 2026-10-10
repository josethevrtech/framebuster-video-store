use crate::store_geometry::{Vertex, box_mesh};

pub fn append(v: &mut Vec<Vertex>) {
    let trim = [0.085, 0.075, 0.09];
    for x in [crate::store_bounds::LEFT+0.11, crate::store_bounds::RIGHT-0.11] {
        box_mesh(v,[x,-1.37,10.0],[0.05,0.22,29.8],trim);
        box_mesh(v,[x,0.60,10.0],[0.06,0.065,29.8],[0.57,0.39,0.16]);
        for z in [-3.0,2.0,7.0,12.0,17.0,22.0] {
            box_mesh(v,[x,-0.30,z],[0.10,2.35,0.18],[0.045,0.075,0.12]);
        }
    }
    for x in [-12.0,-4.0,4.0,12.0,20.0,26.0] {
        for z in [0.0,8.0,16.0,22.0] {
            box_mesh(v,[x,1.968,z],[0.75,0.025,0.48],[0.18,0.19,0.20]);
            for offset in -5..6 {
                box_mesh(v,[x+offset as f32*0.06,1.95,z],[0.024,0.018,0.42],[0.34,0.35,0.34]);
            }
        }
    }
}
