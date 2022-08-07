//! Forward rendering for debugging applications

pub mod solid;

pub use solid::*;


#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pub pos: glam::Vec3,
    pub normal: glam::Vec3,
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

impl gfx::Vertex for Vertex {
    fn get(name: &str) -> Option<(u32, gpu::VertexFormat)> {
        match name {
            "in_pos" => Some((0, gpu::VertexFormat::Vec3)),
            "in_normal" => Some((
                std::mem::size_of::<glam::Vec3>() as u32,
                gpu::VertexFormat::Vec3,
            )),
            "pos" => Some((0, gpu::VertexFormat::Vec3)),
            "normal" => Some((
                std::mem::size_of::<glam::Vec3>() as u32,
                gpu::VertexFormat::Vec3,
            )),
            _ => None,
        }
    }
}

impl mesh::Vertex for Vertex {
    fn new(
        pos: glam::Vec3,
        _: glam::Vec2,
        normal: glam::Vec3,
        _: Option<glam::Vec3>,
        _: Option<glam::Vec3>,
    ) -> Self {
        Self { pos, normal }
    }

    fn set_tangents(&mut self, _: glam::Vec3, _: glam::Vec3) {
        println!("Call to set tangents of ddd::clay::Vertex, no tangent fields so no action taken")
    }

    fn pos(&self) -> glam::Vec3 {
        self.pos
    }

    fn uv(&self) -> Option<glam::Vec2> {
        None
    }

    fn normal(&self) -> Option<glam::Vec3> {
        Some(self.normal)
    }

    fn tangent_u(&self) -> Option<glam::Vec3> {
        None
    }

    fn tangent_v(&self) -> Option<glam::Vec3> {
        None
    }
}
