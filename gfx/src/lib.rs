#![feature(vec_into_raw_parts)]

//! Built on top of [`gpu`] to simplify various things
//!

pub mod encoder;
pub mod mesh;
pub mod pass;
pub mod prelude;
pub mod storage;
pub mod texture;
pub mod uniform;

#[cfg(feature = "reflect")]
pub mod reflect;

pub use encoder::CommandEncoder;
pub use mesh::*;
pub use prelude::*;
pub use storage::*;
pub use texture::*;
pub use uniform::*;

#[cfg(feature = "reflect")]
pub use reflect::*;

pub use image;

#[derive(Debug, Clone, PartialEq)]
pub struct Attachment<'a> {
    pub raw: gpu::Attachment<'a>,
    pub load: gpu::LoadOp,
    pub store: gpu::StoreOp,
}

impl<'a> std::borrow::Borrow<gpu::Attachment<'a>> for Attachment<'a> {
    fn borrow(&self) -> &gpu::Attachment<'a> {
        &self.raw
    }
}
