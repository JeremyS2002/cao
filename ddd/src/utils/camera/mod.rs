//! Camera and Camera Controlling
//! 
//! A camera is just
//!  - projection: Mat4
//!  - view: Mat4
//!  - position: Mat4
//! 
//! The camera's matrices transform between different corrdinate systems.
//! The fourth component of the matrix is used to translate corrdinates.
//! 
//! Objects vertices have positions in "world space", this is the coordinate system that is most intuitive and is what mesh's positions are given in.
//! The view matrix transforms the world space into view space, it moves the origin to the camera's position and rotates the axes to be oriented to the camera
//! The projection matrix transforms the view space to screen space, this is essentiall the final position the object appears on the screen
//! 
//! For more information on camera transforms see: <https://learnopengl.com/Getting-started/Coordinate-Systems>
//! 
//! Since it's not intuitive to work directly with matrices camera controllers produce the view and projection matrices from and internal state that is easier
//! to understand and control based on user input. See [`controller`] for more infomation.

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
