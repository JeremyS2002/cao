use either::Either;
use std::collections::HashMap;

use super::*;
use std::cell::RefCell;

pub(crate) struct RawBaseBuilder {
    pub(crate) inputs: RefCell<
        Vec<(
            PrimitiveType,
            Either<(u32, bool), rspirv::spirv::BuiltIn>,
            Option<&'static str>,
        )>,
    >,
    pub(crate) outputs: RefCell<
        Vec<(
            PrimitiveType,
            Either<(u32, bool), rspirv::spirv::BuiltIn>,
            Option<&'static str>,
        )>,
    >,
    pub(crate) push_constant: RefCell<Option<(DataType, u32, Option<&'static str>)>>,
    pub(crate) uniforms: RefCell<Vec<(DataType, u32, u32, Option<&'static str>)>>,
    pub(crate) storages: RefCell<Vec<(DataType, u32, u32, Option<&'static str>)>>,
    pub(crate) textures: RefCell<
        Vec<(
            rspirv::spirv::Dim,
            crate::texture::Component,
            bool,
            u32,
            u32,
            Option<&'static str>,
        )>,
    >,
    pub(crate) samplers: RefCell<Vec<(u32, u32, Option<&'static str>)>>,
    pub(crate) sampled_textures: RefCell<
        Vec<(
            rspirv::spirv::Dim,
            crate::texture::Component,
            bool,
            u32,
            u32,
            Option<&'static str>,
        )>,
    >,
    pub(crate) functions: RefCell<HashMap<usize, Vec<Instruction>>>,
    pub(crate) main: RefCell<Vec<Instruction>>,
    #[cfg(feature = "gpu")]
    pub(crate) map:
        RefCell<HashMap<(u32, u32), (gpu::DescriptorLayoutEntry, Option<&'static str>)>>,
}

impl RawBaseBuilder {
    pub(crate) fn new() -> Self {
        Self {
            inputs: RefCell::new(Vec::new()),
            outputs: RefCell::new(Vec::new()),
            push_constant: RefCell::new(None),
            uniforms: RefCell::new(Vec::new()),
            storages: RefCell::new(Vec::new()),
            textures: RefCell::new(Vec::new()),
            samplers: RefCell::new(Vec::new()),
            sampled_textures: RefCell::new(Vec::new()),
            functions: RefCell::new(HashMap::new()),
            main: RefCell::new(Vec::new()),
            #[cfg(feature = "gpu")]
            map: RefCell::new(HashMap::new()),
        }
    }
}
