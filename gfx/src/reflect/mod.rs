pub mod bundle;
pub mod compute;
pub mod error;
pub mod graphics;
pub mod resource;

mod raw;

pub use bundle::*;
pub use compute::*;
pub use error::*;
pub use graphics::*;
pub use resource::*;

#[derive(Debug)]
pub(crate) struct ResourceType {
    pub ty: spirv_reflect::types::descriptor::ReflectDescriptorType,
    pub count: u32,
}
