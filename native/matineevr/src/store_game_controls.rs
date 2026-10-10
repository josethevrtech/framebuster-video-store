use crate::store_geometry::{Vertex,box_mesh};

pub fn append(v: &mut Vec<Vertex>) {
    for (index,z) in [11.45,12.5].into_iter().enumerate() {
        box_mesh(v,[14.32,-0.055,z],[0.027,0.09,0.10],[0.11,0.11,0.12]);
        let direction=if index==0 {-1.0} else {1.0};
        for i in 0..3 {
            box_mesh(v,[14.301,-0.055+(i as f32-1.0)*0.014,z-direction*(i as f32-1.0).abs()*0.011],
                [0.005,0.01,0.02],[0.78,0.75,0.64]);
        }
    }
}
pub fn hit(origin: [f32;3],direction: [f32;3]) -> Option<(&'static str,f32)> {
    if direction[0]<=0.001 {return None;}
    let t=(14.298-origin[0])/direction[0];
    if t<=0.0 || (origin[1]+t*direction[1]+0.055).abs()>0.055 {return None;}
    let z=origin[2]+t*direction[2];
    [11.45,12.5].into_iter().enumerate().find(|(_,p)| (z-p).abs()<0.06)
        .map(|(i,_)|(if i==0 {"game-previous"} else {"game-next"},t))
}
