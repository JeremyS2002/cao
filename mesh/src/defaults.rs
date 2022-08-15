
use crate::Vertex;

use std::collections::HashMap;

/// Create a mesh in the shape of a square plane
/// 
/// Centered on the origin side length 2 oriented on the xy axes normal positive z axis
pub fn xy_plane<V: Vertex>(
    encoder: &mut gfx::CommandEncoder<'_>,
    device: &gpu::Device,
    name: Option<&str>,
) -> Result<gfx::IndexedMesh<V>, gpu::Error> {
    gfx::IndexedMesh::new(
        encoder,
        device,
        &[
            V::new(
                glam::vec3(-1.0, -1.0, 0.0),
                glam::vec2(0.0, 0.0),
                glam::Vec3::Z,
                Some(glam::Vec3::X),
                Some(glam::Vec3::Y),
            ),
            V::new(
                glam::vec3(1.0, -1.0, 0.0),
                glam::vec2(1.0, 0.0),
                glam::Vec3::Z,
                Some(glam::Vec3::X),
                Some(glam::Vec3::Y),
            ),
            V::new(
                glam::vec3(1.0, 1.0, 0.0),
                glam::vec2(1.0, 1.0),
                glam::Vec3::Z,
                Some(glam::Vec3::X),
                Some(glam::Vec3::Y),
            ),
            V::new(
                glam::vec3(-1.0, 1.0, 0.0),
                glam::vec2(0.0, 1.0),
                glam::Vec3::Z,
                Some(glam::Vec3::X),
                Some(glam::Vec3::Y),
            ),
        ],
        &[0, 1, 2, 2, 3, 0],
        name,
    )
}

/// Create a mesh in the shape of a square plane
/// 
/// Centered on the origin side length 2 oriented on the xz axes normal positive y axis
pub fn xz_plane<V: Vertex>(
    encoder: &mut gfx::CommandEncoder<'_>,
    device: &gpu::Device,
    name: Option<&str>,
) -> Result<gfx::IndexedMesh<V>, gpu::Error> {
    gfx::IndexedMesh::new(
        encoder,
        device,
        &[
            V::new(
                glam::vec3(-1.0, 0.0, -1.0),
                glam::vec2(0.0, 0.0),
                glam::Vec3::Y,
                Some(glam::Vec3::X),
                Some(glam::Vec3::Z),
            ),
            V::new(
                glam::vec3(1.0, 0.0, -1.0),
                glam::vec2(1.0, 0.0),
                glam::Vec3::Y,
                Some(glam::Vec3::X),
                Some(glam::Vec3::Z),
            ),
            V::new(
                glam::vec3(1.0, 0.0, 1.0),
                glam::vec2(1.0, 1.0),
                glam::Vec3::Y,
                Some(glam::Vec3::X),
                Some(glam::Vec3::Z),
            ),
            V::new(
                glam::vec3(-1.0, 0.0, 1.0),
                glam::vec2(0.0, 1.0),
                glam::Vec3::Y,
                Some(glam::Vec3::X),
                Some(glam::Vec3::Z),
            ),
        ],
        &[0, 1, 2, 2, 3, 0],
        name,
    )
}

/// Create a mesh in the shape of a square plane
/// 
/// Centered on the origin side length 2 oriented on the yz axes normal positive x axis
pub fn yz_plane<V: Vertex>(
    encoder: &mut gfx::CommandEncoder<'_>,
    device: &gpu::Device,
    name: Option<&str>,
) -> Result<gfx::IndexedMesh<V>, gpu::Error> {
    gfx::IndexedMesh::new(
        encoder,
        device,
        &[
            V::new(
                glam::vec3(0.0, -1.0, -1.0),
                glam::vec2(0.0, 0.0),
                glam::Vec3::X,
                Some(glam::Vec3::Y),
                Some(glam::Vec3::Z),
            ),
            V::new(
                glam::vec3(0.0, 1.0, -1.0),
                glam::vec2(1.0, 0.0),
                glam::Vec3::X,
                Some(glam::Vec3::Y),
                Some(glam::Vec3::Z),
            ),
            V::new(
                glam::vec3(0.0, 1.0, 1.0),
                glam::vec2(1.0, 1.0),
                glam::Vec3::X,
                Some(glam::Vec3::Y),
                Some(glam::Vec3::Z),
            ),
            V::new(
                glam::vec3(0.0, -1.0, 1.0),
                glam::vec2(0.0, 1.0),
                glam::Vec3::X,
                Some(glam::Vec3::Y),
                Some(glam::Vec3::Z),
            ),
        ],
        &[0, 1, 2, 2, 3, 0],
        name,
    )
}

/// Create a mesh in the shape of a sphere
pub fn ico_sphere<V: Vertex>(
    encoder: &mut gfx::CommandEncoder<'_>,
    device: &gpu::Device,
    subdivisions: u32,
    name: Option<&str>,
) -> Result<gfx::IndexedMesh<V>, gpu::Error> {
    let mut vertices = Vec::new();

    // golden ratio
    let t = (1.0 + 5.0f32.sqrt()) / 2.0;

    let add = |vertices: &mut Vec<V>, mut t: glam::Vec3| {
        t = t.normalize();
        // t = (cosusinv, sinusinv, cosv); // maps u, v: (0, pi)x(0, 2pi) -> S2
        let v = t.z.acos();
        let u = (t.x / v.sin()).asin();
        //  differentiate t with respecte to u and v to get tu and tv respectivly
        let tu = glam::vec3(-u.sin() * v.sin(), u.cos() * v.sin(), 0.0);
        let tv = glam::vec3(u.cos() * v.cos(), u.sin() * v.cos(), - v.sin());
        vertices.push(
            V::new(
                t,
                glam::vec2(u, v),
                t,
                Some(tu),
                Some(tv),
            )
        );
    };

    add(&mut vertices, glam::vec3(-1.0, t, 0.0));
    add(&mut vertices, glam::vec3(1.0, t, 0.0));
    add(&mut vertices, glam::vec3(-1.0, -t, 0.0));
    add(&mut vertices, glam::vec3(1.0, -t, 0.0));

    add(&mut vertices, glam::vec3(0.0, -1.0, t));
    add(&mut vertices, glam::vec3(0.0, 1.0, t));
    add(&mut vertices, glam::vec3(0.0, -1.0, -t));
    add(&mut vertices, glam::vec3(0.0, 1.0, -t));

    add(&mut vertices, glam::vec3(t, -0.0, -1.0));
    add(&mut vertices, glam::vec3(t, 0.0, 1.0));
    add(&mut vertices, glam::vec3(-t, -0.0, -1.0));
    add(&mut vertices, glam::vec3(-t, 0.0, 1.0));

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
            let v1 = chunk[0];
            let v2 = chunk[1];
            let v3 = chunk[2];

            let a = get_middle(add, &mut cache, &mut vertices, v1, v2);
            let b = get_middle(add, &mut cache, &mut vertices, v2, v3);
            let c = get_middle(add, &mut cache, &mut vertices, v3, v1);

            tmp_indices.extend(&[v1, a, c]);
            tmp_indices.extend(&[v2, b, a]);
            tmp_indices.extend(&[v3, c, b]);
            tmp_indices.extend(&[a, b, c]);
        }
        indices = tmp_indices;
    }

    gfx::IndexedMesh::new(encoder, device, &vertices, &indices, name)
}

fn get_middle<V: Vertex>(add: fn(&mut Vec<V>, glam::Vec3) -> (), c: &mut HashMap<u64, u32>, vertices: &mut Vec<V>, mut p1: u32, mut p2: u32) -> u32 {
    if p2 > p1 {
        std::mem::swap(&mut p1, &mut p2);
    }
    let key = ((p1 as u64) << 32) + p2 as u64;

    if let Some(&idx) = c.get(&key) {
        idx
    } else {
        let v1 = vertices[p1 as usize].pos();
        let v2 = vertices[p2 as usize].pos();
    
        add(vertices, (v1 + v2) / 2.0);
        c.insert(key, vertices.len() as u32 - 1);
        vertices.len() as u32 - 1
    }
}

/// Create a mesh in the shape of a cube
pub fn cube<V: Vertex>(
    encoder: &mut gfx::CommandEncoder<'_>,
    device: &gpu::Device,
    name: Option<&str>,
) -> Result<gfx::BasicMesh<V>, gpu::Error> {
    gfx::BasicMesh::new(
        encoder,
        device,
        &[
            // back face
            V::new(
                glam::vec3(-1.0, -1.0, -1.0),
                glam::vec2(0.0, 0.0),
                glam::Vec3::NEG_Z,
                Some(glam::Vec3::X),
                Some(glam::Vec3::Y), 
            ),
            V::new(
                glam::vec3(1.0, -1.0, -1.0),
                glam::vec2(1.0, 0.0),
                glam::Vec3::NEG_Z,
                Some(glam::Vec3::X),
                Some(glam::Vec3::Y), 
            ),
            V::new(
                glam::vec3(1.0, 1.0, -1.0),
                glam::vec2(1.0, 1.0),
                glam::Vec3::NEG_Z,
                Some(glam::Vec3::X),
                Some(glam::Vec3::Y), 
            ),
            V::new(
                glam::vec3(-1.0, 1.0, -1.0),
                glam::vec2(0.0, 1.0),
                glam::Vec3::NEG_Z,
                Some(glam::Vec3::X),
                Some(glam::Vec3::Y), 
            ),
            V::new(
                glam::vec3(-1.0, -1.0, -1.0),
                glam::vec2(0.0, 0.0),
                glam::Vec3::NEG_Z,
                Some(glam::Vec3::X),
                Some(glam::Vec3::Y), 
            ),
            V::new(
                glam::vec3(1.0, 1.0, -1.0),
                glam::vec2(1.0, 1.0),
                glam::Vec3::NEG_Z,
                Some(glam::Vec3::X),
                Some(glam::Vec3::Y), 
            ),
            // front face
            V::new(
                glam::vec3(-1.0, -1.0, 1.0),
                glam::vec2(0.0, 0.0),
                glam::Vec3::Z,
                Some(glam::Vec3::X),
                Some(glam::Vec3::Y),
            ),
            V::new(
                glam::vec3(1.0, -1.0, 1.0),
                glam::vec2(1.0, 0.0),
                glam::Vec3::Z,
                Some(glam::Vec3::X),
                Some(glam::Vec3::Y),
            ),
            V::new(
                glam::vec3(1.0, 1.0, 1.0),
                glam::vec2(1.0, 1.0),
                glam::Vec3::Z,
                Some(glam::Vec3::X),
                Some(glam::Vec3::Y),
            ),
            V::new(
                glam::vec3(-1.0, 1.0, 1.0),
                glam::vec2(0.0, 1.0),
                glam::Vec3::Z,
                Some(glam::Vec3::X),
                Some(glam::Vec3::Y),
            ),
            V::new(
                glam::vec3(-1.0, -1.0, 1.0),
                glam::vec2(0.0, 0.0),
                glam::Vec3::Z,
                Some(glam::Vec3::X),
                Some(glam::Vec3::Y),
            ),
            V::new(
                glam::vec3(1.0, 1.0, 1.0),
                glam::vec2(1.0, 1.0),
                glam::Vec3::Z,
                Some(glam::Vec3::X),
                Some(glam::Vec3::Y),
            ),
            // top face
            V::new(
                glam::vec3(-1.0, 1.0, -1.0),
                glam::vec2(0.0, 0.0),
                glam::Vec3::Y,
                Some(glam::Vec3::X),
                Some(glam::Vec3::Z),
            ),
            V::new(
                glam::vec3(1.0, 1.0, -1.0),
                glam::vec2(1.0, 0.0),
                glam::Vec3::Y,
                Some(glam::Vec3::X),
                Some(glam::Vec3::Z),
            ),
            V::new(
                glam::vec3(1.0, 1.0, 1.0),
                glam::vec2(1.0, 1.0),
                glam::Vec3::Y,
                Some(glam::Vec3::X),
                Some(glam::Vec3::Z),
            ),
            V::new(
                glam::vec3(-1.0, 1.0, 1.0),
                glam::vec2(0.0, 1.0),
                glam::Vec3::Y,
                Some(glam::Vec3::X),
                Some(glam::Vec3::Z),
            ),
            V::new(
                glam::vec3(-1.0, 1.0, -1.0),
                glam::vec2(0.0, 0.0),
                glam::Vec3::Y,
                Some(glam::Vec3::X),
                Some(glam::Vec3::Z),
            ),
            V::new(
                glam::vec3(1.0, 1.0, 1.0),
                glam::vec2(1.0, 1.0),
                glam::Vec3::Y,
                Some(glam::Vec3::X),
                Some(glam::Vec3::Z),
            ),
            // bottom face
            V::new(
                glam::vec3(-1.0, -1.0, -1.0),
                glam::vec2(0.0, 0.0),
                glam::Vec3::NEG_Y,
                Some(glam::Vec3::X),
                Some(glam::Vec3::Z),
            ),
            V::new(
                glam::vec3(1.0, -1.0, -1.0),
                glam::vec2(1.0, 0.0),
                glam::Vec3::NEG_Y,
                Some(glam::Vec3::X),
                Some(glam::Vec3::Z),
            ),
            V::new(
                glam::vec3(1.0, -1.0, 1.0),
                glam::vec2(1.0, 1.0),
                glam::Vec3::NEG_Y,
                Some(glam::Vec3::X),
                Some(glam::Vec3::Z),
            ),
            V::new(
                glam::vec3(-1.0, -1.0, 1.0),
                glam::vec2(0.0, 1.0),
                glam::Vec3::NEG_Y,
                Some(glam::Vec3::X),
                Some(glam::Vec3::Z),
            ),
            V::new(
                glam::vec3(-1.0, -1.0, -1.0),
                glam::vec2(0.0, 0.0),
                glam::Vec3::NEG_Y,
                Some(glam::Vec3::X),
                Some(glam::Vec3::Z),
            ),
            V::new(
                glam::vec3(1.0, -1.0, 1.0),
                glam::vec2(1.0, 1.0),
                glam::Vec3::NEG_Y,
                Some(glam::Vec3::X),
                Some(glam::Vec3::Z),            
            ),
            // left face
            V::new(
                glam::vec3(-1.0, -1.0, -1.0),
                glam::vec2(0.0, 0.0),
                glam::Vec3::NEG_X,
                Some(glam::Vec3::Y),
                Some(glam::Vec3::Z),            
            ),
            V::new(
                glam::vec3(-1.0, -1.0, 1.0),
                glam::vec2(0.0, 1.0),
                glam::Vec3::NEG_X,
                Some(glam::Vec3::Y),
                Some(glam::Vec3::Z),            
            ),
            V::new(
                glam::vec3(-1.0, 1.0, 1.0),
                glam::vec2(1.0, 1.0),
                glam::Vec3::NEG_X,
                Some(glam::Vec3::Y),
                Some(glam::Vec3::Z),            
            ),
            V::new(
                glam::vec3(-1.0, 1.0, -1.0),
                glam::vec2(1.0, 0.0),
                glam::Vec3::NEG_X,
                Some(glam::Vec3::Y),
                Some(glam::Vec3::Z),            
            ),
            V::new(
                glam::vec3(-1.0, -1.0, -1.0),
                glam::vec2(0.0, 0.0),
                glam::Vec3::NEG_X,
                Some(glam::Vec3::Y),
                Some(glam::Vec3::Z),            
            ),
            V::new(
                glam::vec3(-1.0, 1.0, 1.0),
                glam::vec2(1.0, 1.0),
                glam::Vec3::NEG_X,
                Some(glam::Vec3::Y),
                Some(glam::Vec3::Z),            
            ),
            // right face
            V::new(
                glam::vec3(1.0, -1.0, -1.0),
                glam::vec2(0.0, 0.0),
                glam::Vec3::X,
                Some(glam::Vec3::Y),
                Some(glam::Vec3::Z),            
            ),
            V::new(
                glam::vec3(1.0, -1.0, 1.0),
                glam::vec2(0.0, 1.0),
                glam::Vec3::X,
                Some(glam::Vec3::Y),
                Some(glam::Vec3::Z),            
            ),
            V::new(
                glam::vec3(1.0, 1.0, 1.0),
                glam::vec2(1.0, 1.0),
                glam::Vec3::X,
                Some(glam::Vec3::Y),
                Some(glam::Vec3::Z),            
            ),
            V::new(
                glam::vec3(1.0, 1.0, -1.0),
                glam::vec2(1.0, 0.0),
                glam::Vec3::X,
                Some(glam::Vec3::Y),
                Some(glam::Vec3::Z),            
            ),
            V::new(
                glam::vec3(1.0, -1.0, -1.0),
                glam::vec2(0.0, 0.0),
                glam::Vec3::X,
                Some(glam::Vec3::Y),
                Some(glam::Vec3::Z),            
            ),
            V::new(
                glam::vec3(1.0, 1.0, 1.0),
                glam::vec2(1.0, 1.0),
                glam::Vec3::X,
                Some(glam::Vec3::Y),
                Some(glam::Vec3::Z),            
            ),
        ],
        name,
    )
}
