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
pub mod copy;
pub mod instance;
pub mod smaa;
pub mod vertices;

pub use camera::*;
pub use copy::*;
pub use instance::*;
pub use smaa::*;
pub use vertices::*;
