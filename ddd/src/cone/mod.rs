//! A defered, rasterized, physically based rendering library

pub mod depth;
pub mod gbuffer;
pub mod lights;
pub mod material;
pub mod postprocess;

pub use depth::*;
pub use gbuffer::*;
pub use lights::*;
pub use material::*;
pub use postprocess::*;


#[derive(Debug, Clone, Copy, gfx::Vertex)]
#[allow(dead_code)]
#[repr(C)]
pub struct Vertex {
    pub pos: glam::Vec3,
    pub normal: glam::Vec3,
    pub tangent_u: glam::Vec3,
    pub tangent_v: glam::Vec3,
    pub uv: glam::Vec2,
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

// impl gfx::Vertex for Vertex {
//     fn get(name: &str) -> Option<(u32, gpu::VertexFormat)> {
//         match name {
//             "in_pos" => Some((0, gpu::VertexFormat::Vec3)),
//             "in_normal" => Some((
//                 std::mem::size_of::<glam::Vec3>() as u32,
//                 gpu::VertexFormat::Vec3,
//             )),
//             "in_tangent" => Some((
//                 std::mem::size_of::<glam::Vec3>() as u32 * 2,
//                 gpu::VertexFormat::Vec3,
//             )),
//             "in_tangent_u" => Some((
//                 std::mem::size_of::<glam::Vec3>() as u32 * 2,
//                 gpu::VertexFormat::Vec3,
//             )),
//             "in_tangent_v" => Some((
//                 std::mem::size_of::<glam::Vec3>() as u32 * 3,
//                 gpu::VertexFormat::Vec3,
//             )),
//             "in_uv" => Some((
//                 std::mem::size_of::<glam::Vec3>() as u32 * 4,
//                 gpu::VertexFormat::Vec2,
//             )),
//             "pos" => Some((0, gpu::VertexFormat::Vec3)),
//             "normal" => Some((
//                 std::mem::size_of::<glam::Vec3>() as u32,
//                 gpu::VertexFormat::Vec3,
//             )),
//             "tangent" => Some((
//                 std::mem::size_of::<glam::Vec3>() as u32 * 2,
//                 gpu::VertexFormat::Vec3,
//             )),
//             "tangent_u" => Some((
//                 std::mem::size_of::<glam::Vec3>() as u32 * 2,
//                 gpu::VertexFormat::Vec3,
//             )),
//             "tangent_v" => Some((
//                 std::mem::size_of::<glam::Vec3>() as u32 * 3,
//                 gpu::VertexFormat::Vec3,
//             )),
//             "uv" => Some((
//                 std::mem::size_of::<glam::Vec3>() as u32 * 4,
//                 gpu::VertexFormat::Vec2,
//             )),
//             _ => None,
//         }
//     }
// }

impl mesh::Vertex for Vertex {
    fn new(
        pos: glam::Vec3,
        uv: glam::Vec2,
        normal: glam::Vec3,
        tangent_u: Option<glam::Vec3>,
        tangent_v: Option<glam::Vec3>,
    ) -> Self {
        Self {
            pos,
            uv,
            normal,
            tangent_u: tangent_u.unwrap_or(glam::Vec3::ZERO),
            tangent_v: tangent_v.unwrap_or(glam::Vec3::ZERO),
        }
    }

    fn set_tangents(&mut self, u: glam::Vec3, v: glam::Vec3) {
        self.tangent_u = u;
        self.tangent_v = v;
    }

    fn pos(&self) -> glam::Vec3 {
        self.pos
    }

    fn uv(&self) -> Option<glam::Vec2> {
        Some(self.uv)
    }

    fn normal(&self) -> Option<glam::Vec3> {
        Some(self.normal)
    }

    fn tangent_u(&self) -> Option<glam::Vec3> {
        Some(self.tangent_u)
    }

    fn tangent_v(&self) -> Option<glam::Vec3> {
        Some(self.tangent_v)
    }
}
