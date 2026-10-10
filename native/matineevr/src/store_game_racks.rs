use crate::store_geometry::{Vertex,box_mesh};

pub const CAPACITY: usize=54;
pub const MAX_BAYS: usize=28;
pub fn side(bay: usize) -> f32 { if bay%2==0 {1.0} else {-1.0} }
pub fn yaw(bay: usize) -> f32 { if bay%2==0 {0.0} else {std::f32::consts::PI} }
pub fn center(bay: usize) -> [f32;3] {
    let rack=bay/2;
    let column=rack/8;
    let row=if column%2==0 {rack%8} else {7-rack%8};
    [19.0+column as f32*5.0,-0.45,-2.0+row as f32*3.0]
}
pub fn card(bay: usize,index: usize) -> [f32;3] {
    let p=center(bay);
    [p[0]+side(bay)*((index%9) as f32*0.19-0.76),-1.215+(index/9) as f32*0.29,p[2]+side(bay)*0.18]
}
pub fn obstacles() -> Vec<[f32;4]> {
    (0..MAX_BAYS).step_by(2).map(|i| {let p=center(i);[p[0]-0.92,p[0]+0.92,p[2]-0.37,p[2]+0.37]}).collect()
}
pub fn append(v: &mut Vec<Vertex>) {
    let black=[0.025,0.025,0.03];
    for bay in (0..MAX_BAYS).step_by(2) {
        let p=center(bay);
        box_mesh(v,[p[0],-0.46,p[2]],[1.84,1.90,0.055],black);
        box_mesh(v,[p[0],-1.43,p[2]],[1.85,0.14,0.74],black);
        for y in [-1.355,-1.065,-0.775,-0.485,-0.195,0.095] {
            box_mesh(v,[p[0],y,p[2]],[1.80,0.034,0.70],black);
            for side in [-1.0,1.0] {box_mesh(v,[p[0],y+0.025,p[2]+side*0.35],[1.8,0.034,0.015],[0.06,0.06,0.065]);}
        }
        for x in [-0.91,0.91] {box_mesh(v,[p[0]+x,-0.46,p[2]],[0.035,1.90,0.72],black);}
        for side in [-1.0,1.0] {box_mesh(v,[p[0],0.47,p[2]+side*0.25],[1.84,0.23,0.055],black);}
    }
    box_mesh(v,[22.0,1.15,21.0],[4.94,0.66,0.08],black);
    for x in [20.5,23.5] {box_mesh(v,[x,1.745,21.0],[0.018,0.53,0.018],black);}
}

#[cfg(test)]
mod tests {
    #[test]
    fn all_game_racks_have_clear_aisles_and_fit_the_expanded_room() {
        let racks=super::obstacles();
        for (i,a) in racks.iter().enumerate() {
            assert!(a[0]>16.0 && a[1]<crate::store_bounds::RIGHT-0.075);
            for b in &racks[i+1..] {
                assert!(a[1]+0.95<=b[0] || b[1]+0.95<=a[0] || a[3]+2.0<=b[2] || b[3]+2.0<=a[2]);
            }
            let p=super::center(i*2);
            assert!(crate::store_layout::free([p[0],0.0,p[2]+1.1],0.22));
            for slot in 0..54 {
                let c=super::card(i*2,slot);
                assert!(c[0]-0.083>a[0] && c[0]+0.083<a[1]);
                assert!(c[1]-0.121>-1.5 && c[1]+0.121<0.365);
            }
        }
    }
}
