pub trait TangentVertex {
    fn position(&self) -> glam::Vec3;

    fn uv(&self) -> glam::Vec2;

    fn set_tangent(&mut self, tangent: glam::Vec3);
}

/// Calculate the tangent and bitangent fields for each vertex assuming that
/// the vertices represent a triangle mesh witout adjacency (The mesh will be drawn with topology TriangeList)
pub fn calc_tangent<V: TangentVertex>(vertices: &mut [V]) {
    for tri in vertices.chunks_mut(3) {
        let edge1 = tri[1].position() - tri[0].position();
        let edge2 = tri[2].position() - tri[1].position();

        let duv1 = tri[1].uv() - tri[0].uv();
        let duv2 = tri[2].uv() - tri[0].uv();

        let f = 1.0 / glam::mat2(duv1, duv2).determinant();

        let tangent = glam::vec3(
            f * (duv2.y * edge1.x - duv1.y * edge2.x),
            f * (duv2.y * edge1.y - duv1.y * edge2.y),
            f * (duv2.y * edge1.z - duv1.y * edge2.z),
        );

        // let bitangent = glam::vec3(
        //     f * (-duv2.x * edge1.x + duv1.x * edge2.x),
        //     f * (-duv2.x * edge1.y + duv1.x * edge2.y),
        //     f * (-duv2.x * edge1.z + duv1.x * edge2.z),
        // );

        tri[0].set_tangent(tangent);
        tri[1].set_tangent(tangent);
        tri[2].set_tangent(tangent);
    }
}

pub fn calc_tangent_indexed<V: TangentVertex>(vertices: &mut [V], indices: &[u32]) {
    for tri in indices.chunks(3) {
        let edge1 = vertices[tri[1] as usize].position() - vertices[tri[0] as usize].position();
        let edge2 = vertices[tri[2] as usize].position() - vertices[tri[1] as usize].position();

        let duv1 = vertices[tri[1] as usize].uv() - vertices[tri[0] as usize].uv();
        let duv2 = vertices[tri[2] as usize].uv() - vertices[tri[0] as usize].uv();

        let f = 1.0 / glam::mat2(duv1, duv2).determinant();

        let tangent = glam::vec3(
            f * (duv2.y * edge1.x - duv1.y * edge2.x),
            f * (duv2.y * edge1.y - duv1.y * edge2.y),
            f * (duv2.y * edge1.z - duv1.y * edge2.z),
        );

        // let bitangent = glam::vec3(
        //     f * (-duv2.x * edge1.x + duv1.x * edge2.x),
        //     f * (-duv2.x * edge1.y + duv1.x * edge2.y),
        //     f * (-duv2.x * edge1.z + duv1.x * edge2.z),
        // );

        vertices[tri[0] as usize].set_tangent(tangent);
        vertices[tri[1] as usize].set_tangent(tangent);
        vertices[tri[2] as usize].set_tangent(tangent);
    }
}
