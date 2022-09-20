//! Object instancing utilities
//!
//! When meshes are loaded from file their vertices will be in local posiiton. To move those vertices in world space they are multiplied by the model matrix.
//! By using a storage buffer and indexing per instance index the same geometry can be drawn in multiple positions in one draw call, rather than
//! swapping uniform and repeating draw calls.

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, spv::AsStructType)]
pub struct InstanceData {
    pub model: glam::Mat4,
}

impl From<glam::Mat4> for InstanceData {
    fn from(model: glam::Mat4) -> Self {
        Self { model }
    }
}

unsafe impl bytemuck::Pod for InstanceData {}
unsafe impl bytemuck::Zeroable for InstanceData {}

pub type Instances = gfx::Storage<InstanceData>;
