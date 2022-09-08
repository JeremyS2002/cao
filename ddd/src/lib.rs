#![feature(maybe_uninit_uninit_array, maybe_uninit_array_assume_init, int_roundings)]

//! DDD (3D) rendering library
//!
//! Modules:
//!  - [`cone`]: physically inspired deferred rendering
//!  - [`clay`]: debugging forward renderer
//!  - [`utils`]: common objects between Cone and Clay
//!
//! See the module documentation for more information

pub use glam;

pub mod clay;
pub mod cone;
pub mod prelude;
pub mod utils;

pub use utils::*;