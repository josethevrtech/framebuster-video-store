use crate::{store_geometry::Vertex,store_game_catalog::GameBay};
pub const HEIGHT: u32=include_bytes!("../assets/video-games-signs.rgba").len() as u32/(1024*4);
pub const ROWS: usize=HEIGHT as usize/128;

pub fn mesh(bays: &[GameBay]) -> Vec<Vertex> {
    let mut vertices=Vec::new();
    quad(&mut vertices,[22.0,1.15,21.042],0.0,[4.8,0.60],0);
    quad(&mut vertices,[22.0,1.15,20.958],std::f32::consts::PI,[4.8,0.60],0);
    for section in bays {
        let p=crate::store_game_racks::center(section.bay);
        quad(&mut vertices,[p[0],0.47,p[2]+0.105],0.0,[1.78,0.215],section.system+1);
    }
    vertices
}
fn quad(v: &mut Vec<Vertex>,p: [f32;3],yaw: f32,size: [f32;2],row: usize) {
    let (s,c)=yaw.sin_cos();
    let points: [_;4]=std::array::from_fn(|i| {
        let x=[-0.5,0.5,0.5,-0.5][i]*size[0];
        let y=[-0.5,-0.5,0.5,0.5][i]*size[1];
        let u=if i==1||i==2 {0.998} else {0.002};
        let t=(row as f32+if i>=2 {0.01} else {0.99})/ROWS as f32;
        [[p[0]+c*x,p[1]+y,p[2]-s*x,1.0],[u,t,0.0,0.0],[1.0;4]]
    });
    for i in [0,1,2,0,2,3] {v.push(points[i]);}
}
