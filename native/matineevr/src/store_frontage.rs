use crate::store_geometry::{Vertex, box_mesh};

const BLUE: [f32; 3] = [0.035, 0.10, 0.22];
const FRAME: [f32; 3] = [0.075, 0.085, 0.10];

pub fn append(v: &mut Vec<Vertex>) {
    let start = v.len();
    box_mesh(v, [0.0, 1.57, 13.0], [32.0, 0.86, 0.18], BLUE);
    for x in [-8.65, 8.65] {
        box_mesh(v, [x, -1.34, 13.0], [14.7, 0.32, 0.18], BLUE);
    }
    for x in [-15.9, -1.4, 1.4, 15.9] {
        box_mesh(v, [x, -0.02, 13.0], [0.20, 2.32, 0.24], BLUE);
    }
    box_mesh(v, [0.0, 1.29, 12.87], [32.0, 0.12, 0.04], [0.69, 0.47, 0.16]);
    for x in [-8.65, 8.65] {
        window(v, x, 14.3);
    }
    for x in [-1.25, 0.0, 1.25] {
        box_mesh(v, [x, -0.15, 12.97], [0.065, 2.7, 0.09], FRAME);
    }
    for y in [-1.47, 1.18] {
        box_mesh(v, [0.0, y, 12.97], [2.56, 0.07, 0.09], FRAME);
    }
    for x in [-0.16, 0.16] {
        box_mesh(v, [x, -0.25, 12.84], [0.045, 0.48, 0.06], [0.65, 0.66, 0.64]);
    }
    box_mesh(v, [0.0, -1.478, 12.1], [2.55, 0.035, 1.35], [0.075, 0.055, 0.09]);
    for x in -9..10 {
        box_mesh(v, [x as f32 * 0.12, -1.457, 12.1], [0.012, 0.006, 1.2], [0.20, 0.15, 0.24]);
    }
    box_mesh(v, [6.8, -0.99, 11.95], [0.8, 1.02, 0.65], [0.16, 0.17, 0.19]);
    box_mesh(v, [6.8, -0.7, 11.60], [0.55, 0.095, 0.07], [0.02, 0.025, 0.035]);
    box_mesh(v,[22.0,1.57,13.0],[12.0,0.86,0.18],BLUE);
    box_mesh(v,[22.0,-1.34,13.0],[12.0,0.32,0.18],BLUE);
    box_mesh(v,[22.0,1.29,12.87],[12.0,0.12,0.04],[0.69,0.47,0.16]);
    for x in [16.0,20.0,24.0,27.9] {box_mesh(v,[x,-0.02,13.0],[0.20,2.32,0.24],BLUE);}
    for x in [18.0,22.0,26.0] {window(v,x,3.7);}
    crate::store_night_frontage::append(v);
    for vertex in &mut v[start..] { vertex[0][2] += 12.0; }
}

fn window(v: &mut Vec<Vertex>, x: f32, width: f32) {
    for y in [-1.15, 1.12] {
        box_mesh(v, [x, y, 12.97], [width, 0.07, 0.09], FRAME);
    }
    for offset in [-0.5, -0.1667, 0.1667, 0.5] {
        box_mesh(v, [x + offset * width, -0.015, 12.97], [0.055, 2.27, 0.09], FRAME);
    }
    box_mesh(v, [x, -0.50, 12.97], [width, 0.045, 0.09], FRAME);
    for offset in [-0.33, 0.0, 0.33] {
        box_mesh(v, [x + offset * width + 0.69, 0.35, 12.965],
            [0.018, 1.28, 0.008], [0.43, 0.51, 0.56]);
    }
}
