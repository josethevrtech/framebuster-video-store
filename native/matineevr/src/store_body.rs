use crate::{store_geometry::{Vertex, box_mesh}, store_motion::StoreNavigation};
use openxr as xr;

pub fn append(v: &mut Vec<Vertex>, navigation: &StoreNavigation,
    head: xr::Posef, hands: [Option<xr::Posef>; 2]) {
    let q = head.orientation;
    let (head, forward) = navigation.inverse_ray(point(head),
        [-2.0*(q.x*q.z+q.w*q.y),0.0,-1.0+2.0*(q.x*q.x+q.y*q.y)]);
    let forward = unit([forward[0],0.0,forward[2]]);
    let right = [-forward[2],0.0,forward[0]];
    let chest = add(add(head,mul(forward,-0.10)),[0.0,-0.40,0.0]);
    let start = v.len();
    box_mesh(v,[0.0,0.0,0.0],[0.34,0.40,0.20],[0.055,0.12,0.16]);
    for vertex in &mut v[start..] {
        let p = vertex[0]; let n = vertex[1];
        let p = add(chest,add(mul(right,p[0]),add([0.0,p[1],0.0],mul(forward,-p[2]))));
        vertex[0] = [p[0],p[1],p[2],1.0];
        let n = add(mul(right,n[0]),add([0.0,n[1],0.0],mul(forward,-n[2])));
        vertex[1] = [n[0],n[1],n[2],0.0];
    }
    for (hand, pose) in hands.iter().enumerate() {
        let Some(pose) = pose else { continue; };
        let side = if hand == 0 { -1.0 } else { 1.0 };
        let shoulder = add(add(head,mul(right,side*0.19)),add(mul(forward,-0.08),[0.0,-0.22,0.0]));
        let wrist = navigation.inverse_ray(point(*pose),[0.0;3]).0;
        let elbow = elbow(shoulder,wrist,add(mul(right,side),[0.0,-1.0,0.0]));
        segment(v,shoulder,elbow,0.055,[0.055,0.12,0.16]);
        segment(v,elbow,wrist,0.038,[0.18,0.21,0.23]);
    }
}

fn elbow(shoulder: [f32;3], wrist: [f32;3], bend: [f32;3]) -> [f32;3] {
    let delta = sub(wrist,shoulder); let distance = dot(delta,delta).sqrt();
    let direction = unit(delta);
    let bend = sub(bend,mul(direction,dot(bend,direction)));
    let bend = if dot(bend,bend) < 0.0001 { cross(direction,[0.0,0.0,1.0]) } else { bend };
    let height = (0.32f32.powi(2)-(distance*0.5).powi(2)).max(0.0).sqrt();
    add(add(shoulder,mul(delta,0.5)),mul(unit(bend),height))
}

fn segment(v: &mut Vec<Vertex>, a: [f32;3], b: [f32;3], radius: f32, color: [f32;3]) {
    let direction = unit(sub(b,a));
    let reference = if direction[1].abs() > 0.9 { [1.0,0.0,0.0] } else { [0.0,1.0,0.0] };
    let u = unit(cross(direction,reference)); let w = cross(direction,u);
    for i in 0..8 {
        let radial = |i| {
            let angle = i as f32*std::f32::consts::TAU/8.0;
            add(mul(u,angle.cos()),mul(w,angle.sin()))
        };
        let n = [radial(i),radial(i+1)];
        let p = [add(a,mul(n[0],radius)),add(a,mul(n[1],radius)),
            add(b,mul(n[1],radius)),add(b,mul(n[0],radius))];
        for j in [0,1,2,0,2,3] {
            let n = n[usize::from(j == 1 || j == 2)]; let p = p[j];
            v.push([[p[0],p[1],p[2],1.0],[n[0],n[1],n[2],0.0],[color[0],color[1],color[2],1.0]]);
        }
    }
}
fn point(p: xr::Posef) -> [f32;3] { [p.position.x,p.position.y,p.position.z] }
fn add(a: [f32;3], b: [f32;3]) -> [f32;3] { std::array::from_fn(|i| a[i]+b[i]) }
fn sub(a: [f32;3], b: [f32;3]) -> [f32;3] { std::array::from_fn(|i| a[i]-b[i]) }
fn mul(a: [f32;3], b: f32) -> [f32;3] { a.map(|x| x*b) }
fn dot(a: [f32;3], b: [f32;3]) -> f32 { a.iter().zip(b).map(|(a,b)| a*b).sum() }
fn unit(a: [f32;3]) -> [f32;3] { mul(a,1.0/dot(a,a).sqrt().max(0.0001)) }
fn cross(a: [f32;3], b: [f32;3]) -> [f32;3] { [a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0]] }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reachable_elbow_keeps_both_bones_at_length_and_stays_finite() {
        for wrist in [[0.1,-0.3,-0.3],[0.0,0.0,0.0],[0.0,-0.5,0.0],[1.0,0.0,0.0]] {
            let e = elbow([0.0;3],wrist,[1.0,-1.0,0.0]);
            assert!(e.iter().all(|v| v.is_finite()));
            if dot(wrist,wrist)>0.001 && dot(wrist,wrist)<0.4096 {
                assert!((dot(e,e).sqrt()-0.32).abs()<0.0001);
                let forearm = sub(wrist,e);
                assert!((dot(forearm,forearm).sqrt()-0.32).abs()<0.0001);
            }
        }
    }
}
