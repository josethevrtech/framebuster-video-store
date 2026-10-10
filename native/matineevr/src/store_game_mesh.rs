use crate::store_geometry::Vertex;

pub fn placement(index: usize) -> ([f32;3],[f32;3],[f32;3]) {
    let bay=index/18;
    let row=(index%18)/6;
    let column=index%6;
    let y=[-0.828,-0.538,-0.218][row];
    if bay<2 {
        ([14.50,y,11.45+bay as f32*4.0+column as f32*0.21],[-1.0,0.0,0.0],[0.0,0.0,1.0])
    } else {
        ([11.75+column as f32*0.21,y,17.30],[0.0,0.0,-1.0],[1.0,0.0,0.0])
    }
}

pub fn body(types: &[u8]) -> (Vec<u8>,Vec<[f32;3]>) {
    let source=include_bytes!("../assets/cartridge-models.bin");
    assert_eq!(&source[..8],b"FBCART01");
    let counts: [usize;6]=std::array::from_fn(|i| u32::from_le_bytes(source[8+i*4..12+i*4].try_into().unwrap()) as usize);
    let sizes: [[f32;3];6]=std::array::from_fn(|i| std::array::from_fn(|a|
        f32::from_le_bytes(source[32+(i*3+a)*4..36+(i*3+a)*4].try_into().unwrap())));
    let mut data=b"FBPROP01".to_vec(); data.extend(0u32.to_le_bytes());
    let mut dimensions=Vec::new();
    for (index,&kind) in types.iter().enumerate() {
        let kind=kind as usize;
        let start=104+counts[..kind].iter().sum::<usize>()*48;
        let (p,front,right)=placement(index);
        for source in source[start..start+counts[kind]*48].chunks_exact(48) {
            let mut v: [f32;12]=std::array::from_fn(|a| f32::from_le_bytes(source[a*4..a*4+4].try_into().unwrap()));
            let local=[v[0],v[1],v[2]]; let n=[v[4],v[5],v[6]];
            for a in 0..3 {
                v[a]=p[a]+right[a]*local[0]-front[a]*local[2]+if a==1 {local[1]} else {0.0};
                v[4+a]=right[a]*n[0]-front[a]*n[2]+if a==1 {n[1]} else {0.0};
            }
            for value in v {data.extend(value.to_le_bytes());}
        }
        dimensions.push(sizes[kind]);
    }
    let count=((data.len()-12)/48) as u32;
    data[8..12].copy_from_slice(&count.to_le_bytes());
    (data,dimensions)
}

pub fn labels(types: &[u8],sizes: &[[f32;3]]) -> Vec<Vertex> {
    let mut vertices=Vec::new();
    for (index,(&kind,size)) in types.iter().zip(sizes).enumerate() {
        let (mut p,front,_)=placement(index);
        let width=if kind==3 {size[0]*0.92} else if kind==2 {size[0]*0.90} else {size[0]*0.78};
        let height=if kind==3 {size[1]*0.94} else if kind==2 {size[1]*0.85} else {size[1]*0.70};
        p[1]+=size[1]*0.55;
        for a in [0,2] {p[a]+=front[a]*(size[2]/2.0+0.0015);}
        crate::store_cover_mesh::quad(&mut vertices,p,front[0].atan2(front[2]),[width,height],index);
    }
    vertices
}

#[cfg(test)]
mod tests {
    #[test]
    fn rentals_have_metres_for_dimensions_and_labels_outside_the_front() {
        let types=[0,1,2,3,4,5];
        let (mesh,sizes)=super::body(&types);
        assert_eq!(mesh.len(),12+u32::from_le_bytes(mesh[8..12].try_into().unwrap()) as usize*48);
        for size in &sizes {assert!(size[0]>0.05 && size[0]<0.17); assert!(size[1]>0.05 && size[1]<0.20);}
        let labels=super::labels(&types,&sizes);
        for (i,size) in sizes.iter().enumerate() {
            let (p,front,_)=super::placement(i);
            let point=labels[i*6][0];
            let depth=(point[0]-p[0])*front[0]+(point[2]-p[2])*front[2];
            assert!(depth>size[2]/2.0);
        }
    }
}
