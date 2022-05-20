
use std::rc::Rc;
use std::cell::RefCell;
use super::{RawBuilder, RawBaseBuilder, Variables};
use crate::data::*;

pub(crate) struct RawFnBuilder {
    /// Always BaseBuilder
    pub(crate) builder: Rc<RawBaseBuilder>,
    pub(crate) id: usize,
    pub(crate) instructions: RefCell<Vec<super::Instruction>>,
    pub(crate) variables: RefCell<Variables>
}


impl super::RawBuilder for RawFnBuilder {
    fn push_instruction(&self, instruction: super::Instruction) {
        self.instructions.borrow_mut().push(instruction);
    }

    fn get_new_id(&self, ty: PrimitiveType) -> usize {
        self.variables.borrow_mut().get_new_id(ty)
    }

    fn name_var(&self, ty: PrimitiveType, id: usize, name: String) {
        self.variables.borrow_mut().name_var(ty, id, name)
    }
}

impl Drop for RawFnBuilder {
    fn drop(&mut self) {
        self.builder.functions.borrow_mut().insert(
            self.id,
            self.instructions.borrow_mut().drain(..).collect()
        );
    }
}

pub struct FnBuilder {
    // Always a RawFnBuilder
    pub(crate) raw: Rc<dyn RawBuilder>,
}

impl FnBuilder {
    /// Define the inputs to the function
    pub fn inputs(_i: &[PrimitiveType]) {
        todo!();
    }

    /// Get the input at index supplied
    /// Returns [`None`] if the T doesn't match the input data type
    pub fn input<T: DataRef>(_index: u32) -> Option<T> {
        todo!();
    }
}
