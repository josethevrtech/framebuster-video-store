use crate::store_geometry::{Vertex,box_mesh};

pub const CENTER: [f32;3] = [crate::store_bounds::RIGHT-0.50,-0.5,22.20];
pub const COVER: [f32;3] = [CENTER[0]-0.245,-0.10,22.20];
pub fn place(p: [f32;3]) -> [f32;3] { [CENTER[0]+p[2],p[1],CENTER[2]-p[0]] }
pub fn rotate(n: [f32;3]) -> [f32;3] { [n[2],n[1],-n[0]] }
pub const SPEAKERS: [[f32;3];6] = [
    [-6.0,1.45,0.0],[6.0,1.45,0.0],[-6.0,1.45,11.0],
    [6.0,1.45,11.0],[-6.0,1.45,22.0],[6.0,1.45,22.0],
];
pub fn obstacle() -> [f32;4] { [CENTER[0]-0.34,CENTER[0]+0.32,21.67,22.73] }
pub fn append(v: &mut Vec<Vertex>) {
    let chrome=[0.61,0.67,0.72];
    for (i,x) in [-0.23,0.0,0.23].into_iter().enumerate() {
        let p=place([x,-0.61,-0.252]);
        box_mesh(v,p,[0.015,0.09,0.14],[0.045,0.05,0.055]);
        if i==1 {
            for dx in [-0.018,0.018] { box_mesh(v,[p[0]-0.009,p[1],p[2]-dx],[0.005,0.043,0.01],chrome); }
        } else {
            let sign=if i==0 { -1.0 } else { 1.0 };
            let points=[[-sign*0.025,-0.025],[sign*0.025,0.0],[-sign*0.025,0.025]];
            for [x,y] in points { v.push([[p[0]-0.011,p[1]+y,p[2]-x,1.0],[-1.0,0.0,0.0,0.0],[chrome[0],chrome[1],chrome[2],1.0]]); }
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
    if d[0]<=0.001 { return None; }
    let t=(CENTER[0]-0.265-p[0])/d[0];
    if t<=0.0 || (p[1]+t*d[1]+0.61).abs()>0.06 { return None; }
    let x=CENTER[2]-p[2]-t*d[2];
    [-0.23,0.0,0.23].into_iter().enumerate().find(|(_,button)| (x-button).abs()<0.075)
        .map(|(i,_)| (["music-previous","music-toggle","music-next"][i],t))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn jukebox_controls_use_their_physical_button_positions() {
        for (x,action) in [(-0.23,"music-previous"),(0.0,"music-toggle"),(0.23,"music-next")] {
            let p=[CENTER[0]-1.0,-0.61,CENTER[2]-x];
            assert_eq!(hit(p,[1.0,0.0,0.0]).unwrap().0,action);
            assert!(hit(p,[-1.0,0.0,0.0]).is_none());
        }
    }
}
