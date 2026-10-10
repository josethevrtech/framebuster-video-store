use std::sync::OnceLock;
type Point=[f32;2];
fn cross(a: Point,b: Point,c: Point) -> f32 {(b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])}
pub fn checkout() -> &'static [Point] {
    static HULL: OnceLock<Vec<Point>>=OnceLock::new();
    HULL.get_or_init(|| {
        let mut points: Vec<_>=include_bytes!("../assets/halcyon-fixtures.bin")[12..].chunks_exact(48)
            .map(|v| [f32::from_le_bytes(v[0..4].try_into().unwrap()),f32::from_le_bytes(v[8..12].try_into().unwrap())]).collect();
        points.sort_by(|a,b| a[0].total_cmp(&b[0]).then(a[1].total_cmp(&b[1])));points.dedup();
        let mut hull=Vec::new();
        for sequence in [points.clone(),points.into_iter().rev().collect()] {
            let mut half=Vec::new();
            for p in sequence {
                while half.len()>1 && cross(half[half.len()-2],half[half.len()-1],p)<=0.0 {half.pop();}
                half.push(p);
            }
            half.pop();hull.extend(half);
        }
        hull
    })
}
fn nearest(p: Point,a: Point,b: Point) -> Point {
    let d=[b[0]-a[0],b[1]-a[1]];
    let t=(((p[0]-a[0])*d[0]+(p[1]-a[1])*d[1])/(d[0]*d[0]+d[1]*d[1]).max(0.000001)).clamp(0.0,1.0);
    [a[0]+d[0]*t,a[1]+d[1]*t]
}
pub fn blocked(p: Point,radius: f32) -> bool {
    let hull=checkout();
    hull.iter().enumerate().all(|(i,&a)| cross(a,hull[(i+1)%hull.len()],p)>=0.0)
        || hull.iter().enumerate().any(|(i,&a)| {let q=nearest(p,a,hull[(i+1)%hull.len()]);(p[0]-q[0]).hypot(p[1]-q[1])<radius})
}
pub fn recover(p: [f32;3],radius: f32) -> [f32;3] {
    if crate::store_layout::free(p,radius) {return p;}
    let margin=radius+0.002;
    let mut candidates=Vec::new();
    candidates.push([p[0].clamp(crate::store_bounds::LEFT+0.25+margin,crate::store_bounds::RIGHT-0.25-margin),p[1],p[2].clamp(-4.4+margin,24.7-margin)]);
    for b in crate::store_layout::obstacles() {
        let q=[p[0].clamp(b[0],b[1]),p[2].clamp(b[2],b[3])];
        let d=[p[0]-q[0],p[2]-q[1]];let length=d[0].hypot(d[1]);
        if length>0.00001 {candidates.push([q[0]+d[0]*margin/length,p[1],q[1]+d[1]*margin/length]);}
        for x in [b[0]-margin,b[1]+margin] {candidates.push([x,p[1],p[2]]);}
        for z in [b[2]-margin,b[3]+margin] {candidates.push([p[0],p[1],z]);}
    }
    let hull=checkout();
    for (i,&a) in hull.iter().enumerate() {
        let b=hull[(i+1)%hull.len()];let q=nearest([p[0],p[2]],a,b);
        let d=[b[0]-a[0],b[1]-a[1]];let length=d[0].hypot(d[1]);
        candidates.push([q[0]+d[1]*margin/length,p[1],q[1]-d[0]*margin/length]);
        let delta=[p[0]-q[0],p[2]-q[1]];let length=delta[0].hypot(delta[1]);
        if length>0.00001 {candidates.push([q[0]+delta[0]*margin/length,p[1],q[1]+delta[1]*margin/length]);}
    }
    candidates.into_iter().filter(|&q| crate::store_layout::free(q,radius))
        .min_by(|a,b| ((p[0]-a[0]).hypot(p[2]-a[2])).total_cmp(&(p[0]-b[0]).hypot(p[2]-b[2]))).unwrap_or(p)
}
pub fn slide(start: [f32;3],target: [f32;3]) -> [f32;3] {
    let mut p=recover(start,0.20);
    let delta=[target[0]-start[0],target[2]-start[2]];
    let steps=(delta[0].hypot(delta[1])/0.025).ceil().max(1.0) as usize;
    for _ in 0..steps {
        let q=[p[0]+delta[0]/steps as f32,p[1],p[2]+delta[1]/steps as f32];
        let resolved=recover(q,0.20);
        if crate::store_layout::free(resolved,0.20) && (resolved[0]-p[0]).hypot(resolved[2]-p[2])<0.05 {p=resolved;}
    }
    p
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn embedded_head_recovers_from_every_cabinet_and_checkout() {
        for b in crate::store_layout::obstacles() {
            let p=[(b[0]+b[1])*0.5,0.0,(b[2]+b[3])*0.5];
            assert!(crate::store_layout::free(slide(p,p),0.20),"{p:?}");
        }
        let p=[0.0,0.0,17.0];assert!(blocked([p[0],p[2]],0.20));
        assert!(crate::store_layout::free(slide(p,p),0.20));
    }
    #[test]
    fn diagonal_motion_slides_along_corners_without_crossing_counter() {
        let hull=checkout();
        let (a,b)=hull.iter().enumerate().map(|(i,&a)| (a,hull[(i+1)%hull.len()]))
            .find(|(a,b)| (a[1]+b[1])*0.5<17.0 && (a[0]-b[0]).hypot(a[1]-b[1])>1.0).unwrap();
        let length=(b[0]-a[0]).hypot(b[1]-a[1]);
        let tangent=[(b[0]-a[0])/length,(b[1]-a[1])/length];
        let normal=[tangent[1],-tangent[0]];
        let start=[(a[0]+b[0])*0.5+normal[0]*0.202,0.0,(a[1]+b[1])*0.5+normal[1]*0.202];
        let mut p=start;
        for _ in 0..25 {
            p=slide(p,[p[0]+tangent[0]*0.01-normal[0]*0.005,p[1],p[2]+tangent[1]*0.01-normal[1]*0.005]);
            assert!(crate::store_layout::free(p,0.20));
        }
        assert!((p[0]-start[0]).hypot(p[2]-start[2])>0.23);
    }
}
