use crate::store_geometry::{Vertex,box_mesh};
use std::sync::atomic::{AtomicUsize,Ordering};
pub const MAX_BAYS: usize=24;
pub const DECK: [f32;3]=[-11.5,-0.68,24.0];
pub const COVER: [f32;3]=[-11.5,-0.22,23.968];
static ACTIVE: AtomicUsize=AtomicUsize::new(0);
pub fn activate(count: usize) {ACTIVE.store(count,Ordering::Relaxed);}
pub fn yaw(bay: usize) -> f32 {if bay<6 {0.0} else {std::f32::consts::FRAC_PI_2}}
pub fn center(bay: usize) -> [f32;3] {
    if bay<6 {[-14.65+bay as f32*1.45,-0.45,-4.55]}
    else {[-15.5,-0.45,-2.0+(bay-6) as f32*1.5]}
}
pub fn point(bay: usize,p: [f32;3]) -> [f32;3] {
    let center=center(bay);let (s,c)=yaw(bay).sin_cos();
    [center[0]+c*p[0]+s*p[2],p[1],center[2]-s*p[0]+c*p[2]]
}
pub fn card(bay: usize,index: usize) -> [f32;3] {
    point(bay,[(index%6) as f32*0.20-0.50,0.24-(index/6) as f32*0.20,0.12])
}
fn placed(v: &mut Vec<Vertex>,bay: usize,p: [f32;3],size: [f32;3],color: [f32;3]) {
    let start=v.len();box_mesh(v,p,size,color);let (s,c)=yaw(bay).sin_cos();
    for vertex in &mut v[start..] {
        let p=point(bay,[vertex[0][0],vertex[0][1],vertex[0][2]]);vertex[0][..3].copy_from_slice(&p);
        let [x,y,z,_]=vertex[1];vertex[1]=[c*x+s*z,y,-s*x+c*z,0.0];
    }
}
pub fn rack(v: &mut Vec<Vertex>,bay: usize,count: usize) {
    let black=[0.025,0.025,0.03];
    placed(v,bay,[0.0,-0.47,-0.10],[1.34,1.94,0.05],black);
    placed(v,bay,[0.0,-1.46,0.0],[1.35,0.08,0.36],black);
    for row in 0..9 {placed(v,bay,[0.0,0.14-row as f32*0.20,0.0],[1.30,0.025,0.34],black);}
    for x in [-0.66,0.66] {placed(v,bay,[x,-0.47,0.0],[0.025,1.94,0.35],black);}
    placed(v,bay,[0.0,0.54,0.07],[1.34,0.20,0.05],black);
    for index in 0..count {
        let p=[(index%6) as f32*0.20-0.50,0.24-(index/6) as f32*0.20,0.12];
        placed(v,bay,p,[0.175,0.175,0.012],[0.12,0.13,0.14]);
        placed(v,bay,[p[0]-0.076,p[1],p[2]+0.007],[0.018,0.173,0.003],[0.015,0.015,0.019]);
    }
}
pub fn obstacles() -> Vec<[f32;4]> {
    let mut boxes=vec![[-12.22,-10.78,23.75,24.25]];
    for bay in 0..ACTIVE.load(Ordering::Relaxed) {
        let p=center(bay);let [x,z]=if bay<6 {[0.675,0.19]} else {[0.19,0.675]};
        boxes.push([p[0]-x,p[0]+x,p[2]-z,p[2]+z]);
    }
    boxes
}
pub fn labels(bay: usize,count: usize) -> Vec<Vertex> {
    let mut v=Vec::new();let (s,c)=yaw(bay).sin_cos();
    for index in 0..count {
        let mut p=card(bay,index);p[0]+=s*0.008;p[2]+=c*0.008;
        let start=v.len();crate::store_cover_mesh::quad(&mut v,p,yaw(bay),[0.153,0.153],index);
        for vertex in &mut v[start..] {vertex[1][1]+=if vertex[1][1]==vfloat(index,false) {-48.0/3240.0} else {48.0/3240.0};}
    }
    v
}
fn vfloat(index: usize,top: bool) -> f32 {
    let bottom=(index/18)*1080+56+((index%18)/6)*360;
    (3240-bottom-if top {288} else {0}) as f32/3240.0
}
pub fn hit(p: [f32;3],d: [f32;3],bay: usize,index: usize) -> Option<f32> {
    let (s,c)=yaw(bay).sin_cos();let q=card(bay,index);let denom=d[0]*s+d[2]*c;
    if denom>=-0.001 {return None;}
    let t=((q[0]+s*0.008-p[0])*s+(q[2]+c*0.008-p[2])*c)/denom;
    let x=(p[0]+t*d[0]-q[0])*c-(p[2]+t*d[2]-q[2])*s;
    (t>0.0 && x.abs()<0.0875 && (p[1]+t*d[1]-q[1]).abs()<0.0875).then_some(t)
}
pub fn deck(v: &mut Vec<Vertex>) {
    box_mesh(v,[-11.5,-1.12,24.0],[1.44,0.76,0.50],[0.13,0.085,0.05]);
    box_mesh(v,DECK,[1.22,0.12,0.30],[0.13,0.14,0.15]);
    box_mesh(v,[-11.5,-0.10,24.0],[0.70,1.02,0.06],[0.03,0.03,0.035]);
    for x in [-11.90,-11.5,-11.10] {box_mesh(v,[x,-0.675,23.839],[0.075,0.038,0.010],[0.52,0.54,0.56]);}
}
pub fn controls(p: [f32;3],d: [f32;3]) -> Option<(&'static str,f32)> {
    if d[2]<=0.001 {return None;}let t=(23.832-p[2])/d[2];
    if t<=0.0 || (p[1]+t*d[1]+0.675).abs()>0.035 {return None;}
    [-11.90,-11.5,-11.10].into_iter().enumerate().find(|(_,x)|(p[0]+t*d[0]-x).abs()<0.10)
        .map(|(i,_)| (["music-previous","music-toggle","music-next"][i],t))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_cd_has_square_art_and_selects_its_own_case_from_the_front() {
        for bay in 0..MAX_BAYS {
            let (s,c)=yaw(bay).sin_cos();let labels=labels(bay,54);
            for i in 0..54 {
                let p=card(bay,i);assert!((p[1]-0.0875)>-1.5);
                let eye=[p[0]+s,p[1],p[2]+c];let direction=[-s,0.0,-c];
                assert!((hit(eye,direction,bay,i).unwrap()-0.992).abs()<0.0001);
                assert!(hit(eye,direction,bay,(i+1)%54).is_none());
                let vertices=&labels[i*6..i*6+6];
                assert!(((vertices[0][1][1]-vertices[2][1][1])*3240.0-192.0).abs()<0.01);
            }
        }
    }
    #[test]
    fn music_racks_fit_clear_of_movie_shelves_and_each_other() {
        for bay in 0..MAX_BAYS {
            let p=center(bay);
            if bay<6 {assert!(p[0]+0.675< -6.24);}else {assert!(p[0]+0.19< -12.04);assert!(p[2]+0.675<24.7);}
            assert!(p[0]-if bay<6 {0.675} else {0.19}>crate::store_bounds::LEFT);
        }
    }
}
