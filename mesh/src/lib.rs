pub mod tangent;

pub use tangent::*;

use std::collections::HashMap;

pub trait BasicVertex: gfx::Vertex {
    fn new(pos: glam::Vec3) -> Self;

    fn pos(&self) -> glam::Vec3;
}

/// Create a mesh in the shape of a plane
pub fn create_plane<V: BasicVertex>(
    encoder: &mut gfx::CommandEncoder<'_>,
    device: &gpu::Device,
    name: Option<&str>,
) -> Result<gfx::IndexedMesh<V>, gpu::Error> {
    gfx::IndexedMesh::new(
        encoder,
        device,
        &[
            V::new([-1.0, -1.0, 0.0].into()),
            V::new([1.0, -1.0, 0.0].into()),
            V::new([1.0, 1.0, 0.0].into()),
            V::new([-1.0, 1.0, 0.0].into()),
        ],
        &[0, 1, 2, 2, 3, 0],
        name,
    )
}

/// Create a mesh in the shape of a sphere
pub fn create_ico_sphere<V: BasicVertex>(
    encoder: &mut gfx::CommandEncoder<'_>,
    device: &gpu::Device,
    subdivisions: u32,
    name: Option<&str>,
) -> Result<gfx::IndexedMesh<V>, gpu::Error> {
    let mut vertices = Vec::new();

    // golden ratio
    let t = (1.0 + 5.0f32.sqrt()) / 2.0;

    let add = |v: &mut Vec<V>, t: [f32; 3]| {
        let mut t: glam::Vec3 = t.into();
        t = t.normalize();
        v.push(V::new(t.into()));
    };

    add(&mut vertices, [-1.0, t, 0.0]);
    add(&mut vertices, [1.0, t, 0.0]);
    add(&mut vertices, [-1.0, -t, 0.0]);
    add(&mut vertices, [1.0, -t, 0.0]);

    add(&mut vertices, [0.0, -1.0, t]);
    add(&mut vertices, [0.0, 1.0, t]);
    add(&mut vertices, [0.0, -1.0, -t]);
    add(&mut vertices, [0.0, 1.0, -t]);

    add(&mut vertices, [t, -0.0, -1.0]);
    add(&mut vertices, [t, 0.0, 1.0]);
    add(&mut vertices, [-t, -0.0, -1.0]);
    add(&mut vertices, [-t, 0.0, 1.0]);

    let mut indices: Vec<u32> = Vec::new();

    indices.extend(&[0, 11, 5]);
    indices.extend(&[0, 5, 1]);
    indices.extend(&[0, 1, 7]);
    indices.extend(&[0, 7, 10]);
    indices.extend(&[0, 10, 11]);

    indices.extend(&[1, 5, 9]);
    indices.extend(&[5, 11, 4]);
    indices.extend(&[11, 10, 2]);
    indices.extend(&[10, 7, 6]);
    indices.extend(&[7, 1, 8]);

    indices.extend(&[3, 9, 4]);
    indices.extend(&[3, 4, 2]);
    indices.extend(&[3, 2, 6]);
    indices.extend(&[3, 6, 8]);
    indices.extend(&[3, 8, 9]);

    indices.extend(&[4, 9, 5]);
    indices.extend(&[2, 4, 11]);
    indices.extend(&[6, 2, 10]);
    indices.extend(&[8, 6, 7]);
    indices.extend(&[9, 8, 1]);

    let mut cache: HashMap<u64, u32> = HashMap::new();

    for _ in 0..subdivisions {
        let mut tmp_indices: Vec<u32> = Vec::new();

        for chunk in indices.chunks(3) {
            let get_middle =
                |c: &mut HashMap<u64, u32>, v: &mut Vec<V>, mut p1: u32, mut p2: u32| -> u32 {
                    if p2 > p1 {
                        std::mem::swap(&mut p1, &mut p2);
                    }
                    let key = ((p1 as u64) << 32) + p2 as u64;

                    if let Some(&idx) = c.get(&key) {
                        idx
                    } else {
                        let v1 = v[p1 as usize].pos();
                        let v2 = v[p2 as usize].pos();
                        let middle = V::new(((v1 + v2) / 2.0).normalize());
                        v.push(middle);
                        c.insert(key, v.len() as u32 - 1);
                        v.len() as u32 - 1
                    }
                };

            let v1 = chunk[0];
            let v2 = chunk[1];
            let v3 = chunk[2];

            let a = get_middle(&mut cache, &mut vertices, v1, v2);
            let b = get_middle(&mut cache, &mut vertices, v2, v3);
            let c = get_middle(&mut cache, &mut vertices, v3, v1);

            tmp_indices.extend(&[v1, a, c]);
            tmp_indices.extend(&[v2, b, a]);
            tmp_indices.extend(&[v3, c, b]);
            tmp_indices.extend(&[a, b, c]);
        }
        indices = tmp_indices;
    }

    gfx::IndexedMesh::new(encoder, device, &vertices, &indices, name)
}

/// Create a mesh in the shape of a cube
pub fn create_cube<V: BasicVertex>(
    encoder: &mut gfx::CommandEncoder<'_>,
    device: &gpu::Device,
    name: Option<&str>,
) -> Result<gfx::BasicMesh<V>, gpu::Error> {
    gfx::BasicMesh::new(
        encoder,
        device,
        &[
            // back face
            V::new([-1.0, -1.0, -1.0].into()),
            V::new([1.0, -1.0, -1.0].into()),
            V::new([1.0, 1.0, -1.0].into()),
            V::new([-1.0, 1.0, -1.0].into()),
            V::new([-1.0, -1.0, -1.0].into()),
            V::new([1.0, 1.0, -1.0].into()),
            // front face
            V::new([-1.0, -1.0, 1.0].into()),
            V::new([1.0, -1.0, 1.0].into()),
            V::new([1.0, 1.0, 1.0].into()),
            V::new([-1.0, 1.0, 1.0].into()),
            V::new([-1.0, -1.0, 1.0].into()),
            V::new([1.0, 1.0, 1.0].into()),
            // top face
            V::new([-1.0, 1.0, -1.0].into()),
            V::new([1.0, 1.0, -1.0].into()),
            V::new([1.0, 1.0, 1.0].into()),
            V::new([-1.0, 1.0, 1.0].into()),
            V::new([-1.0, 1.0, -1.0].into()),
            V::new([1.0, 1.0, 1.0].into()),
            // bottom face
            V::new([-1.0, -1.0, -1.0].into()),
            V::new([1.0, -1.0, -1.0].into()),
            V::new([1.0, -1.0, 1.0].into()),
            V::new([-1.0, -1.0, 1.0].into()),
            V::new([-1.0, -1.0, -1.0].into()),
            V::new([1.0, -1.0, 1.0].into()),
            // left face
            V::new([-1.0, -1.0, -1.0].into()),
            V::new([-1.0, -1.0, 1.0].into()),
            V::new([-1.0, 1.0, 1.0].into()),
            V::new([-1.0, 1.0, -1.0].into()),
            V::new([-1.0, -1.0, -1.0].into()),
            V::new([-1.0, 1.0, 1.0].into()),
            // right face
            V::new([1.0, -1.0, -1.0].into()),
            V::new([1.0, -1.0, 1.0].into()),
            V::new([1.0, 1.0, 1.0].into()),
            V::new([1.0, 1.0, -1.0].into()),
            V::new([1.0, -1.0, -1.0].into()),
            V::new([1.0, 1.0, 1.0].into()),
        ],
        name,
    )
}
