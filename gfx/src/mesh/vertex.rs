/// Allows for using vertex buffers of different types
/// with the same shader modules by recreating the graphics pipeline
/// with a different vertex state
pub trait Vertex: bytemuck::Pod + bytemuck::Zeroable {
    /// based on name get the offset and format of that vertex
    fn get(name: &str) -> Option<(u32, gpu::VertexFormat)>;
}

impl Vertex for () {
    fn get(_: &str) -> Option<(u32, gpu::VertexFormat)> {
        None
    }
}
