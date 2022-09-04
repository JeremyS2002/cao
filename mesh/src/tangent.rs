use crate::Vertex;

/// Calculate the tangent and bitangent fields for each vertex assuming that
/// the vertices represent a triangle mesh witout adjacency (The mesh will be drawn with topology TriangeList)
pub fn calc_tangent<V: Vertex>(vertices: &mut [V]) {
    for tri in vertices.chunks_mut(3) {
        let edge1 = tri[1].pos() - tri[0].pos();
        let edge2 = tri[2].pos() - tri[1].pos();

        let duv1 = tri[1].uv().unwrap() - tri[0].uv().unwrap();
        let duv2 = tri[2].uv().unwrap() - tri[0].uv().unwrap();

        let f = 1.0 / glam::mat2(duv1, duv2).determinant();

        let u = glam::vec3(
            f * (duv2.y * edge1.x - duv1.y * edge2.x),
            f * (duv2.y * edge1.y - duv1.y * edge2.y),
            f * (duv2.y * edge1.z - duv1.y * edge2.z),
        );

        let v = glam::vec3(
            f * (-duv2.x * edge1.x + duv1.x * edge2.x),
            f * (-duv2.x * edge1.y + duv1.x * edge2.y),
            f * (-duv2.x * edge1.z + duv1.x * edge2.z),
        );

        tri[0].set_tangents(u, v);
        tri[1].set_tangents(u, v);
        tri[2].set_tangents(u, v);
    }
}

pub fn calc_tangent_indexed<V: Vertex>(vertices: &mut [V], indices: &[u32]) {
    for tri in indices.chunks(3) {
        let edge1 = vertices[tri[1] as usize].pos() - vertices[tri[0] as usize].pos();
        let edge2 = vertices[tri[2] as usize].pos() - vertices[tri[1] as usize].pos();

        let duv1 =
            vertices[tri[1] as usize].uv().unwrap() - vertices[tri[0] as usize].uv().unwrap();
        let duv2 =
            vertices[tri[2] as usize].uv().unwrap() - vertices[tri[0] as usize].uv().unwrap();

        let f = 1.0 / glam::mat2(duv1, duv2).determinant();

        let u = glam::vec3(
            f * (duv2.y * edge1.x - duv1.y * edge2.x),
            f * (duv2.y * edge1.y - duv1.y * edge2.y),
            f * (duv2.y * edge1.z - duv1.y * edge2.z),
        );

        let v = glam::vec3(
            f * (-duv2.x * edge1.x + duv1.x * edge2.x),
            f * (-duv2.x * edge1.y + duv1.x * edge2.y),
            f * (-duv2.x * edge1.z + duv1.x * edge2.z),
        );

        vertices[tri[0] as usize].set_tangents(u, v);
        vertices[tri[1] as usize].set_tangents(u, v);
        vertices[tri[2] as usize].set_tangents(u, v);
    }
}
