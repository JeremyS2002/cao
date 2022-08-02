use super::{RawBaseBuilder, RawBuilder, Variables};
use std::cell::RefCell;
use std::rc::Rc;

pub(crate) struct RawMainBuilder {
    // Always BaseBuilder
    pub(crate) builder: Rc<RawBaseBuilder>,
    pub(crate) instructions: RefCell<Vec<super::Instruction>>,
    pub(crate) variables: RefCell<Variables>,
}

impl RawBuilder for RawMainBuilder {
    fn push_instruction(&self, instruction: super::Instruction) {
        self.instructions.borrow_mut().push(instruction);
    }

    fn get_new_id(&self) -> usize {
        self.variables.borrow_mut().get_new_id()
    }

    fn name_var(&self, id: usize, name: String) {
        self.variables.borrow_mut().name_var(id, name);
    }

    fn in_loop(&self) -> bool {
        false
    }

    fn push_constant(&self) -> Option<(crate::DataType, u32, Option<&'static str>)> {
        (*self.builder.push_constant.borrow()).clone()
    }

    
}

impl Drop for RawMainBuilder {
    fn drop(&mut self) {
        // std::mem::swap(self.instructions.borrow_mut(), self.builder.instructions.borrow_mut());
        *self.builder.main.borrow_mut() = self.instructions.borrow_mut().drain(..).collect();
    }
}

pub struct MainHandle {
    // will always be a RawMainBuilder
    pub(crate) raw: Rc<dyn RawBuilder>,
}

impl MainHandle {}
