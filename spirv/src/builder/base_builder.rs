use either::Either;
use std::collections::HashMap;

use super::*;
use std::cell::RefCell;

pub(crate) struct RawBaseBuilder {
    pub(crate) inputs: RefCell<
        Vec<(
            PrimitiveType,
            Either<u32, rspirv::spirv::BuiltIn>,
            Option<&'static str>,
        )>,
    >,
    pub(crate) outputs: RefCell<
        Vec<(
            PrimitiveType,
            Either<u32, rspirv::spirv::BuiltIn>,
            Option<&'static str>,
        )>,
    >,
    pub(crate) uniforms: RefCell<Vec<DataType>>,
    pub(crate) storages: RefCell<Vec<DataType>>,
    pub(crate) functions: RefCell<HashMap<usize, Vec<Instruction>>>,
    pub(crate) main: RefCell<Vec<Instruction>>,
}

impl RawBaseBuilder {
    pub(crate) fn new() -> Self {
        Self {
            inputs: RefCell::new(Vec::new()),
            outputs: RefCell::new(Vec::new()),
            uniforms: RefCell::new(Vec::new()),
            storages: RefCell::new(Vec::new()),
            functions: RefCell::new(HashMap::new()),
            main: RefCell::new(Vec::new()),
        }
    }
}
