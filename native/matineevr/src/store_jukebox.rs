use crate::store_geometry::{Vertex,box_mesh};

pub const CENTER: [f32;3] = [7.8,-0.5,3.0];
pub const COVER: [f32;3] = [7.8,-0.10,2.755];
pub const SPEAKERS: [[f32;3];6] = [
    [-6.0,1.45,0.0],[6.0,1.45,0.0],[-6.0,1.45,11.0],
    [6.0,1.45,11.0],[-6.0,1.45,22.0],[6.0,1.45,22.0],
];
pub fn obstacle() -> [f32;4] { [7.13,8.47,2.49,3.49] }
pub fn append(v: &mut Vec<Vertex>) {
    let chrome=[0.61,0.67,0.72];
    for (i,x) in [-0.23,0.0,0.23].into_iter().enumerate() {
        let p=[CENTER[0]+x,-0.61,2.748];
        box_mesh(v,p,[0.14,0.09,0.015],[0.045,0.05,0.055]);
        if i==1 {
            for dx in [-0.018,0.018] { box_mesh(v,[p[0]+dx,p[1],p[2]-0.009],[0.01,0.043,0.005],chrome); }
        } else {
            let sign=if i==0 { -1.0 } else { 1.0 };
            let points=[[-sign*0.025,-0.025],[sign*0.025,0.0],[-sign*0.025,0.025]];
            for [x,y] in points { v.push([[p[0]+x,p[1]+y,p[2]-0.011,1.0],[0.0,0.0,-1.0,0.0],[chrome[0],chrome[1],chrome[2],1.0]]); }
        }
    }
    for p in SPEAKERS {
        box_mesh(v,[p[0],1.83,p[2]],[0.08,0.40,0.08],chrome);
        box_mesh(v,p,[0.48,0.42,0.32],[0.025,0.025,0.03]);
        for row in 0..14 {
            for col in 0..18 {
                box_mesh(v,[p[0]-0.215+col as f32*0.025,p[1]-0.175+row as f32*0.025,p[2]-0.164],
                    [0.005,0.005,0.003],[0.14,0.15,0.16]);
            }
        }
    }
}
pub fn hit(p: [f32;3],d: [f32;3]) -> Option<(&'static str,f32)> {
    if d[2]<=0.001 { return None; }
    let t=(2.735-p[2])/d[2];
    if t<=0.0 || (p[1]+t*d[1]+0.61).abs()>0.06 { return None; }
    let x=p[0]+t*d[0]-CENTER[0];
    [-0.23,0.0,0.23].into_iter().enumerate().find(|(_,button)| (x-button).abs()<0.075)
        .map(|(i,_)| (["music-previous","music-toggle","music-next"][i],t))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn jukebox_controls_use_their_physical_button_positions() {
        for (x,action) in [(-0.23,"music-previous"),(0.0,"music-toggle"),(0.23,"music-next")] {
            let p=[CENTER[0]+x,-0.61,2.0];
            assert_eq!(hit(p,[0.0,0.0,1.0]).unwrap().0,action);
            assert!(hit(p,[0.0,0.0,-1.0]).is_none());
        }
    }
}
