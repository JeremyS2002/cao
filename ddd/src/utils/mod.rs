//! Utilities
//! 
//! Shared camera, instancing and vertices between [`crate::cone`] and [`crate::clay`]
//! 
//! See sub modules for more specifics 
//!  - [`camera`]
//!  - [`instance`]
//!  - [`vertices`]
//!  - [`smaa`]

pub mod camera;
pub mod instance;
pub mod vertices;
pub mod smaa;
pub mod copy;

pub use camera::*;
pub use instance::*;
pub use vertices::*;
pub use smaa::*;
pub use copy::*;
