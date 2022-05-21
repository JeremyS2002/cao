
use std::rc::Rc;
use super::{RawBuilder, RawBaseBuilder, Variables};
use crate::data::*;
use std::cell::RefCell;

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

    fn get_new_id(&self, ty: PrimitiveType) -> usize {
        self.variables.borrow_mut().get_new_id(ty)
    }

    fn name_var(&self, ty: PrimitiveType, id: usize, name: String) {
        self.variables.borrow_mut().name_var(ty, id, name);
    }

    fn in_loop(&self) -> bool {
        false
    }
}

impl Drop for RawMainBuilder {
    fn drop(&mut self) {
        // std::mem::swap(self.instructions.borrow_mut(), self.builder.instructions.borrow_mut());
        *self.builder.main.borrow_mut() = self.instructions.borrow_mut().drain(..).collect();
    }
}

pub struct MainBuilder {
    // will always be a RawMainBuilder
    pub(crate) raw: Rc<dyn RawBuilder>,
}

impl MainBuilder {
    
}