pub type DirLight = gfx::Uniform<DirLightData>;
pub type DirLights = gfx::Storage<DirLightData>;

/// TODO
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DirLightData {}

unsafe impl bytemuck::Pod for DirLightData {}
unsafe impl bytemuck::Zeroable for DirLightData {}
