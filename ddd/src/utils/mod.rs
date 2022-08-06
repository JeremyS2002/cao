//! Utilities
//! 
//! Shared camera, instancing and vertices between [`crate::cone`] and [`crate::clay`]
//! 
//! See sub modules for more specifics 
//!  - [`camera`]
//!  - [`instance`]
//!  - [`vertices`]

pub mod camera;
pub mod instance;
pub mod vertices;

pub use camera::*;
pub use instance::*;
pub use vertices::*;
