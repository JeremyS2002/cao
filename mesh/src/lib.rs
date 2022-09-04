pub mod defaults;
#[cfg(feature = "loading")]
pub mod loading;
pub mod tangent;

pub use defaults::*;
#[cfg(feature = "loading")]
pub use loading::*;
pub use tangent::*;

/// Vertex trait specific to 3D use cases
pub trait Vertex: gfx::Vertex {
    /// Create a new vertex from possible values the vertex could have
    fn new(
        pos: glam::Vec3,
        uv: glam::Vec2,
        normal: glam::Vec3,
        tangent_u: Option<glam::Vec3>,
        tangent_v: Option<glam::Vec3>,
    ) -> Self;

    /// Should set the tangent vectors of this vertex if any
    fn set_tangents(&mut self, u: glam::Vec3, v: glam::Vec3);

    /// Get the position of the vertex
    fn pos(&self) -> glam::Vec3;

    /// Get the uv coordinate of this vertex if any
    fn uv(&self) -> Option<glam::Vec2>;

    /// Get the normal vector of this vertex if any
    fn normal(&self) -> Option<glam::Vec3>;

    /// Get the tangent in the u direction if any
    fn tangent_u(&self) -> Option<glam::Vec3>;

    /// Get the tangent in the v direction if any
    fn tangent_v(&self) -> Option<glam::Vec3>;
}
