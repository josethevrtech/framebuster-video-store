use crate::store_geometry::{Vertex,box_mesh};

pub fn append(v: &mut Vec<Vertex>) {
    box_mesh(v,[6.0,3.0,34.0],[84.0,25.0,0.3],[0.003,0.006,0.016]);
    for x in -24..36 {
        for z in 13..34 {
            let center=[x as f32+0.5,-1.59,z as f32+0.5];
            let sidewalk=z<16;
            let mut color=if sidewalk { [0.09,0.085,0.07] } else { [0.015,0.018,0.024] };
            for lamp in [-11.0,11.0,25.0] {
                let distance=(center[0]-lamp).powi(2)+(center[2]-20.0).powi(2);
                let light=0.19*(-distance/24.0).exp();
                color[0]+=light; color[1]+=light*0.62; color[2]+=light*0.19;
            }
            let entrance=0.12*(-(center[2]-13.3).powi(2)/3.0).exp();
            color[0]+=entrance; color[1]+=entrance*0.64; color[2]+=entrance*0.24;
            box_mesh(v,center,[1.0,0.08,1.0],color);
        }
    }
    box_mesh(v,[6.0,-1.48,15.12],[46.0,0.16,0.18],[0.18,0.15,0.095]);
    for x in -4..5 {
        box_mesh(v,[x as f32*2.7,-1.545,19.1],[0.075,0.008,5.0],[0.23,0.17,0.065]);
    }
    for x in [-11.0,11.0,25.0] {
        box_mesh(v,[x,0.4,20.0],[0.12,4.0,0.12],[0.045,0.045,0.05]);
        box_mesh(v,[x,2.38,20.0],[0.95,0.15,0.45],[0.15,0.12,0.055]);
        box_mesh(v,[x,2.294,20.0],[0.80,0.024,0.34],[0.94,0.65,0.22]);
    }
    awning(v);
}
fn awning(v: &mut Vec<Vertex>) {
    for x in -32..56 {
        let left=x as f32*0.5;
        let points=[[left,1.75,13.12],[left+0.5,1.75,13.12],
            [left+0.5,1.38,14.7],[left,1.38,14.7]];
        for i in [0,1,2,0,2,3] {
            let p=points[i];
            v.push([[p[0],p[1],p[2],1.0],[0.0,-0.974,-0.228,0.0],[0.055,0.12,0.24,1.0]]);
        }
        box_mesh(v,[left,1.562,13.91],[0.014,0.014,1.62],[0.09,0.16,0.27]);
    }
    box_mesh(v,[6.0,1.31,14.70],[44.0,0.16,0.04],[0.035,0.08,0.16]);
    for x in [-12.0,-6.0,0.0,6.0,12.0,18.0,24.0] {
        box_mesh(v,[x,1.565,13.18],[0.70,0.065,0.08],[0.95,0.68,0.24]);
    }
}
