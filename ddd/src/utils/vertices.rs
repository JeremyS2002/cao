//! Vertices used internally

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BasicVertex(pub glam::Vec3);

impl std::ops::Deref for BasicVertex {
    type Target = glam::Vec3;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for BasicVertex {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Into<glam::Vec3>> From<T> for BasicVertex {
    fn from(v: T) -> Self {
        Self(v.into())
    }
}

unsafe impl bytemuck::Pod for BasicVertex {}
unsafe impl bytemuck::Zeroable for BasicVertex {}

impl gfx::Vertex for BasicVertex {
    fn get(name: &str) -> Option<(u32, gpu::VertexFormat)> {
        match name {
            "in_pos" => Some((0, gpu::VertexFormat::Vec3)),
            _ => None,
        }
    }
}

impl mesh::Vertex for BasicVertex {
    fn new(
        pos: glam::Vec3,
        _: glam::Vec2,
        _: glam::Vec3,
        _: Option<glam::Vec3>,
        _: Option<glam::Vec3>,
    ) -> Self {
        Self(pos)
    }

    fn set_tangents(&mut self, _: glam::Vec3, _: glam::Vec3) {}

    fn pos(&self) -> glam::Vec3 {
        self.0
    }

    fn normal(&self) -> Option<glam::Vec3> {
        None
    }

    fn tangent_u(&self) -> Option<glam::Vec3> {
        None
    }

    fn tangent_v(&self) -> Option<glam::Vec3> {
        None
    }

    fn uv(&self) -> Option<glam::Vec2> {
        None
    }
}
