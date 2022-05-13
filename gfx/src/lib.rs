pub mod encoder;
pub mod mesh;
pub mod pass;
pub mod prelude;
#[cfg(feature = "reflect")]
pub mod reflect;
pub mod storage;
pub mod texture;
pub mod uniform;

pub use encoder::CommandEncoder;
pub use mesh::*;
pub use prelude::*;
pub use storage::*;
pub use texture::*;
pub use uniform::*;

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
