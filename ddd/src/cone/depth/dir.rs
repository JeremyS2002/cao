/// projection + view matrices and shadow strength for DirLights
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct DirShadowData {
    /// projection matrix, transforms from view space to screen space
    pub projection: glam::Mat4,
    /// view matrix, transforms from world space to view space
    pub view: glam::Mat4,
    /// position of the dir shadow caster
    pub position: glam::Vec3,
    /// glsl interprets position as a vec3 which has the same memory layout as vec4
    pub _padding1: u32,
    /// strength of shadow, how sharp the shadow should be
    pub strength: f32,
    /// match alignment
    pub _padding2: u32,
    pub _padding3: u64,
}

unsafe impl bytemuck::Pod for DirShadowData {}
unsafe impl bytemuck::Zeroable for DirShadowData {}
