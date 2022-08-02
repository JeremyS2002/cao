//! Reflected Graphics and Compute passes
//!
//! [`BasicGraphicsPass`], [`ReflectedGraphicsPass`], [`BasicComputePass`] and [`ReflectedComputePass`] cannot be created by themselves
//!  
//! instead they are created from a [`crate::CommandEncoder`] via one of
//! - [`crate::CommandEncoder::graphics_pass_ref`],
//! - [`crate::CommandEncoder::graphics_pass_owned`],
//! - [`crate::CommandEncoder::graphics_pass_reflected`],
//! - [`crate::CommandEncoder::compute_pass_reflected_ref`],
//! - [`crate::CommandEncoder::compute_pass_reflected_owned`]
//!
//! See [`GraphicsPass`] or [`ComputePass`] for the methods that the passes implement
//!
//! import these via [`crate::prelude`] ```import gfx::prelude::*```

pub mod compute;
pub mod graphics;

pub use compute::*;
pub use graphics::*;
