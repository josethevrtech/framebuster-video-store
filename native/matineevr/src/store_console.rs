use crate::{graphics::Graphics,store_geometry::{Vertex,box_mesh}};
use anyhow::Result;
use ash::vk;
use std::rc::Rc;

pub struct StoreConsole {
    props: crate::store_material_props::MaterialProps,
    counts: [u32;5],
    pub selected: usize,
}
impl StoreConsole {
    pub fn new(device: Rc<Graphics>) -> Result<Self> {
        let (props,counts)=crate::store_material_props::MaterialProps::consoles(device)?;
        Ok(Self { props,counts,selected:3 })
    }
    pub fn prepare(&mut self,command: vk::CommandBuffer) { self.props.prepare(command); }
    pub fn draw(&self,command: vk::CommandBuffer,parameters: [[f32;4];5],device: &Graphics) {
        self.props.draw_range(command,parameters,device,self.counts[..self.selected].iter().sum(),self.counts[self.selected]);
    }
    pub fn lights(&self,v: &mut Vec<Vertex>) {
        let p=crate::store_room_layout::SELECTOR;
        let x=p[0]-0.36+self.selected as f32*0.18;
        box_mesh(v,[x,p[1]+0.035,p[2]-0.006],[0.045,0.009,0.009],[0.12,0.85,0.27]);
    }
}
pub fn append(v: &mut Vec<Vertex>) {
    let p=crate::store_room_layout::SELECTOR;
    box_mesh(v,p,[1.0,0.105,0.012],[0.10,0.10,0.11]);
    for i in 0..5 {
        let x=p[0]-0.36+i as f32*0.18;
        box_mesh(v,[x,p[1],p[2]-0.013],[0.105,0.052,0.02],[0.21,0.21,0.22]);
        for dot in 0..=i {
            box_mesh(v,[x-0.030+dot as f32*0.015,p[1]-0.006,p[2]-0.025],[0.005,0.005,0.002],[0.75,0.75,0.68]);
        }
    }
    let tv=crate::store_arcade::TV;
    box_mesh(v,[tv[0]+0.92,tv[1]+0.009,tv[2]],[0.31,0.014,0.23],[0.07,0.07,0.08]);
}
pub fn hit(origin: [f32;3],direction: [f32;3]) -> Option<(usize,f32)> {
    if direction[2]<=0.001 { return None; }
    let p=crate::store_room_layout::SELECTOR;
    let t=(p[2]-0.030-origin[2])/direction[2];
    if t<=0.0 || (origin[1]+t*direction[1]-p[1]).abs()>0.032 { return None; }
    let x=origin[0]+t*direction[0];
    (0..5).find(|i| (x-(p[0]-0.36+*i as f32*0.18)).abs()<0.060).map(|i| (i,t))
}
#[cfg(test)]
mod tests {
    #[test]
    fn cabinet_inputs_select_distinct_consoles_from_the_front() {
        for i in 0..5 {
            let p=crate::store_room_layout::SELECTOR;
            let ray=[p[0]-0.36+i as f32*0.18,p[1],p[2]-2.0];
            assert_eq!(super::hit(ray,[0.0,0.0,1.0]).unwrap().0,i);
            assert!(super::hit(ray,[0.0,0.0,-1.0]).is_none());
        }
    }
}
