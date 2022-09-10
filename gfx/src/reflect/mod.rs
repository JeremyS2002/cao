//! Utilities for creating pipelines
//!
//! If reflect feature is enabled then there are methods for creating pipeline layouts from spir-v data
//!
//! If spirv feature is enabled then there are methods for creating pipeline layouts from [`spv::Builder`] objects
//!
//! This isn't as fast as hard coding the values but speeds up prototyping a lot for me.
//!
//! [`ReflectedGraphics`] wraps a [`gpu::GraphicsPipeline`] and also manages [`gpu::RenderPass`] so that one ReflectedGraphics can render to different targets
//!
//! [`ReflectedCompute`] wraps a [`gpu::ComputePipeline`]
//!
//! [`Bundle`] manages [`gpu::DescriptorSet`] and [`BundleBuilder`] is used to assign resources to locations by name

pub mod bundle;
pub mod compute;
pub mod error;
pub mod graphics;
pub mod resource;
pub mod data;

#[cfg(feature = "reflect")]
mod reflect_raw;

#[cfg(feature = "spirv")]
mod spirv_raw;

pub use bundle::*;
pub use compute::ReflectedCompute;
pub use error::*;
pub use graphics::ReflectedGraphics;
pub use resource::*;
