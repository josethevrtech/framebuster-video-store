use crate::store_geometry::Vertex;

pub fn append(vertices: &mut Vec<Vertex>) {
    decode(vertices, include_bytes!("../assets/kenney-furniture.bin"));
}

fn decode(vertices: &mut Vec<Vertex>, bytes: &[u8]) {
    assert_eq!(&bytes[..8], b"FBPROP01");
    let count = u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;
    assert_eq!(bytes.len(), 12 + count * 48);
    for chunk in bytes[12..].chunks_exact(48) {
        let numbers: [f32; 12] = std::array::from_fn(|i|
            f32::from_le_bytes(chunk[i * 4..i * 4 + 4].try_into().unwrap()));
        vertices.push(std::array::from_fn(|row| std::array::from_fn(|column| numbers[row * 4 + column])));
    }
}
