use crate::{store_geometry::{Vertex,box_mesh},store_album_catalog::AlbumBay};
pub const HEIGHT: u32=include_bytes!("../assets/music-signs.rgba").len() as u32/(1024*4);
pub const GENRES: usize=HEIGHT as usize/128-5;
fn quad(v: &mut Vec<Vertex>,p: [f32;3],yaw: f32,size: [f32;2],row: usize) {
    let (s,c)=yaw.sin_cos();let rows=HEIGHT as f32/128.0;
    let points: [_;4]=std::array::from_fn(|i| {
        let x=[-0.5,0.5,0.5,-0.5][i]*size[0];let y=[-0.5,-0.5,0.5,0.5][i]*size[1];
        [[p[0]+c*x,p[1]+y,p[2]-s*x,1.0],[if i==1||i==2 {0.998} else {0.002},(row as f32+if i>=2 {0.01} else {0.99})/rows,0.0,0.0],[1.0;4]]
    });for i in [0,1,2,0,2,3] {v.push(points[i]);}
}
pub fn mesh(bays: &[AlbumBay]) -> Vec<Vertex> {
    let mut v=Vec::new();quad(&mut v,[-10.5,1.20,-4.74],0.0,[4.2,0.55],0);
    for b in bays {
        quad(&mut v,crate::store_album_racks::point(b.bay,[0.0,0.54,0.098]),crate::store_album_racks::yaw(b.bay),[1.29,0.18],b.genre+1);
    }
    let yaw=std::f32::consts::PI;
    quad(&mut v,[-11.5,0.30,23.965],yaw,[0.67,0.12],GENRES+1);
    for (i,x) in [-11.90,-11.5,-11.10].into_iter().enumerate() {quad(&mut v,[x,-0.85,23.744],yaw,[0.36,0.065],GENRES+2+i);}
    v
}
pub fn board(v: &mut Vec<Vertex>) {box_mesh(v,[-10.5,1.20,-4.77],[4.3,0.61,0.05],[0.025,0.025,0.03]);}
