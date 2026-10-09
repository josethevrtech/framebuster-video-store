use crate::store_geometry::Vertex;

pub fn signs(v: &mut Vec<Vertex>) {
    crate::store_endcaps::geometry(v);
}
