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
        let x=11.74+self.selected as f32*0.18;
        box_mesh(v,[x,-0.915,12.94],[0.045,0.009,0.009],[0.12,0.85,0.27]);
    }
}
pub fn append(v: &mut Vec<Vertex>) {
    box_mesh(v,[12.10,-0.95,12.959],[1.0,0.105,0.012],[0.10,0.10,0.11]);
    for i in 0..5 {
        let x=11.74+i as f32*0.18;
        box_mesh(v,[x,-0.95,12.946],[0.105,0.052,0.02],[0.21,0.21,0.22]);
        for dot in 0..=i {
            box_mesh(v,[x-0.030+dot as f32*0.015,-0.956,12.934],[0.005,0.005,0.002],[0.75,0.75,0.68]);
        }
    }
    let tv=crate::store_arcade::TV;
    box_mesh(v,[tv[0]+0.92,tv[1]+0.009,tv[2]],[0.31,0.014,0.23],[0.07,0.07,0.08]);
}
pub fn hit(origin: [f32;3],direction: [f32;3]) -> Option<(usize,f32)> {
    if direction[2]<=0.001 { return None; }
    let t=(12.929-origin[2])/direction[2];
    if t<=0.0 || (origin[1]+t*direction[1]+0.95).abs()>0.032 { return None; }
    let x=origin[0]+t*direction[0];
    (0..5).find(|i| (x-(11.74+*i as f32*0.18)).abs()<0.060).map(|i| (i,t))
}
#[cfg(test)]
mod tests {
    #[test]
    fn cabinet_inputs_select_distinct_consoles_from_the_front() {
        for i in 0..5 {
            let ray=[11.74+i as f32*0.18,-0.95,11.0];
            assert_eq!(super::hit(ray,[0.0,0.0,1.0]).unwrap().0,i);
            assert!(super::hit(ray,[0.0,0.0,-1.0]).is_none());
        }
    }
}
