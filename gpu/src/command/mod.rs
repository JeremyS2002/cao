//! [`CommandBuffer`]'s are used for recording commands for the gpu

pub mod buffer;
pub(crate) mod raw;
pub(crate) mod garbage;

pub use buffer::*;

pub(crate) use garbage::*;
