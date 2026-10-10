use crate::store_geometry::{Vertex,box_mesh};

pub fn boxes(bay: usize,count: usize) -> Vec<Vertex> {
    let mut vertices=Vec::new();
    for index in 0..count {
        box_mesh(&mut vertices,crate::store_game_racks::card(bay,index),[0.164,0.242,0.025],[0.022,0.022,0.026]);
    }
    vertices
}
pub fn labels(bay: usize,count: usize) -> Vec<Vertex> {
    let mut vertices=Vec::new();
    for index in 0..count {
        let mut p=crate::store_game_racks::card(bay,index);p[2]+=crate::store_game_racks::side(bay)*0.014;
        crate::store_cover_mesh::quad(&mut vertices,p,crate::store_game_racks::yaw(bay),[0.155,0.2325],index);
    }
    vertices
}

#[cfg(test)]
mod tests {
    #[test]
    fn every_game_cover_is_in_front_of_its_own_box_with_the_correct_atlas_slot() {
        for bay in 0..32 {
            let labels=super::labels(bay,54);
            assert_eq!(super::boxes(bay,54).len(),54*36);
            for index in 0..54 {
                let p=crate::store_game_racks::card(bay,index);
                let cover=labels[index*6];
                let side=crate::store_game_racks::side(bay);
                assert!((cover[0][2]-p[2])*side>0.0125);
                assert!((cover[0][0]+side*0.0775-p[0]).abs()<0.0001);
                assert!((cover[1][0]*1440.0-(24+(index%6)*240) as f32).abs()<0.01);
            }
        }
    }
}
