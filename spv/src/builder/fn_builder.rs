use super::{RawBaseBuilder, RawBuilder, Variables};
use crate::data::*;
use std::cell::RefCell;
use std::rc::Rc;

pub(crate) struct RawFnBuilder {
    /// Always BaseBuilder
    pub(crate) builder: Rc<RawBaseBuilder>,
    pub(crate) id: usize,
    pub(crate) instructions: RefCell<Vec<super::Instruction>>,
    pub(crate) variables: RefCell<Variables>,
}

impl super::RawBuilder for RawFnBuilder {
    fn push_instruction(&self, instruction: super::Instruction) {
        self.instructions.borrow_mut().push(instruction);
    }

    fn get_new_id(&self) -> usize {
        self.variables.borrow_mut().get_new_id()
    }

    fn name_var(&self, id: usize, name: String) {
        self.variables.borrow_mut().name_var(id, name)
    }

    fn in_loop(&self) -> bool {
        false
    }

    fn push_constant(&self) -> Option<(DataType, Option<&'static str>)> {
        (*self.builder.push_constant.borrow()).clone()
    }

    
}

impl Drop for RawFnBuilder {
    fn drop(&mut self) {
        self.builder
            .functions
            .borrow_mut()
            .insert(self.id, self.instructions.borrow_mut().drain(..).collect());
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
