use crate::store_geometry::{Vertex,box_mesh};

pub use crate::store_room_layout::TV;
pub const CEILING_TVS: [[f32;3];4] = [[-10.0,1.27,4.0],[-5.5,1.27,9.0],[-1.0,1.27,14.0],[3.5,1.27,19.0]];

pub fn obstacles() -> Vec<[f32;4]> {
    crate::store_room_layout::LOUNGE_OBSTACLES.to_vec()
}

pub fn append(v: &mut Vec<Vertex>) {
    crate::store_console::append(v);
    for p in CEILING_TVS {
        box_mesh(v,[p[0],1.72,p[2]+0.04],[0.035,0.52,0.035],[0.10,0.10,0.12]);
        box_mesh(v,[p[0],1.976,p[2]+0.04],[0.13,0.016,0.13],[0.10,0.10,0.12]);
        for x in [-0.245,0.245] {
            box_mesh(v,[p[0]+x,1.28,p[2]+0.005],[0.018,0.54,0.28],[0.10,0.10,0.12]);
            box_mesh(v,[p[0]+x*0.51,1.006,p[2]+0.005],[0.25,0.018,0.28],[0.10,0.10,0.12]);
        }
    }
}
