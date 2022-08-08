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
pub mod display;

pub use camera::*;
pub use instance::*;
pub use vertices::*;
pub use smaa::*;
pub use display::*;

bitflags::bitflags!(
    pub struct DisplayFlags: u32 {
        /// Clip HDR images to 0-1 range any value outside will be set to 1
        const CLIP         = 0b001;
        /// Apply reinhard tonemapping
        const REINHARD     = 0b010;
        /// Apply aces tonemapping
        const ACES         = 0b100;
    }
);