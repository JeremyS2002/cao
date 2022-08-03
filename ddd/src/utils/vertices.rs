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

impl mesh::BasicVertex for BasicVertex {
    fn new(pos: glam::Vec3) -> Self {
        Self(pos)
    }

    fn pos(&self) -> glam::Vec3 {
        self.0
    }
}
