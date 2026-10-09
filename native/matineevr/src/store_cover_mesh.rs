use crate::{store_display, store_endcaps, store_geometry::Vertex};

pub fn mesh(count: usize) -> Vec<Vertex> {
    let mut result = Vec::new();
    for case in 0..store_display::BAYS.len()*18 {
        let index = case%54;
        if index >= count { continue; }
        let mut p = store_display::card(case); p[1] += 0.0225;
        let yaw = store_display::BAYS[case/18].1;
        let (s,c) = yaw.sin_cos(); p[0] += s*0.055; p[2] += c*0.055;
        quad(&mut result,p,yaw,[0.216,0.324],index);
    }
    for (i,&(p,yaw)) in store_endcaps::POSTERS.iter().enumerate().take(count) {
        quad(&mut result,p,yaw,[0.70,1.05],i);
    }
    for (i,&p) in crate::store_arcade::CEILING_TVS.iter().enumerate() {
        if count == 0 { break; }
        quad(&mut result,[p[0],p[1]+0.025,p[2]-0.348],std::f32::consts::PI,
            [0.3067,0.46],i%count);
    }
    result
}

fn quad(v: &mut Vec<Vertex>, p: [f32;3], yaw: f32, size: [f32;2], index: usize) {
    let left = 24+(index%6)*240;
    let bottom = (index/18)*1080+56+((index%18)/6)*360;
    let uv = [[left as f32/1440.0,(3240-bottom) as f32/3240.0],
        [(left+192) as f32/1440.0,(3240-bottom-288) as f32/3240.0]];
    let (s,c) = yaw.sin_cos();
    let points: [_;4] = std::array::from_fn(|i| {
        let x = [-0.5,0.5,0.5,-0.5][i]*size[0]; let y = [-0.5,-0.5,0.5,0.5][i]*size[1];
        let u = uv[usize::from(i == 1 || i == 2)][0]; let t = uv[usize::from(i >= 2)][1];
        [[p[0]+c*x,p[1]+y,p[2]-s*x,1.0],[u,t,0.0,0.0],[1.0;4]]
    });
    for i in [0,1,2,0,2,3] { v.push(points[i]); }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cover_uvs_use_the_same_catalog_entry_as_each_case() {
        let v = mesh(54);
        assert_eq!(v.len(),(180+4+4)*6);
        for i in 0..180 {
            let uv = v[i*6][1]; let item = i%54;
            assert!((uv[0]*1440.0-(24+(item%6)*240) as f32).abs()<0.01);
            assert!((uv[1]*3240.0-(3240-(item/18)*1080-56-((item%18)/6)*360) as f32).abs()<0.01);
        }
    }
}
