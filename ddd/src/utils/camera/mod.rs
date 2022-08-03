// use embers_gfx as gfx;
// use embers_gpu as gpu;

use crate::*;

// use parking_lot::RwLock;
// use std::collections::HashMap;

pub mod controller;
pub use controller::*;

pub type Camera = gfx::Uniform<CameraData>;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraData {
    /// projection matrix, transforms view space to unit cube centered on origin
    pub projection: glam::Mat4,
    /// view matrix, transforms world space to camera space
    pub view: glam::Mat4,
    /// position, used for some lighting calculations
    pub position: glam::Vec3,
}

unsafe impl bytemuck::Pod for CameraData {}
unsafe impl bytemuck::Zeroable for CameraData {}

unsafe impl spv::AsSpvStruct for CameraData {
    const DESC: spv::StructDesc = spv::StructDesc {
        name: "CameraData",
        names: &["projection", "view", "position"],
        fields: &[
            spv::DataType::Primitive(spv::PrimitiveType::Mat4),
            spv::DataType::Primitive(spv::PrimitiveType::Mat4),
            spv::DataType::Primitive(spv::PrimitiveType::Vec3),
        ],
    };

    fn fields<'a>(&'a self) -> Vec<&'a dyn spv::AsData> {
        vec![&self.projection, &self.view, &self.position]
    }
}
