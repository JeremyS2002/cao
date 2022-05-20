
use std::rc::Rc;
use super::RawBuilder;
use crate::data::*;
use std::cell::RefCell;

pub(crate) struct RawConditionBuilder {
    pub(crate) builder: Rc<dyn RawBuilder>,
    pub(crate) instructions: RefCell<Vec<Vec<super::Instruction>>>,
    pub(crate) conditions: RefCell<Vec<usize>>,
}

impl RawConditionBuilder {
    pub fn new(builder: Rc<dyn RawBuilder>, condition: usize) -> Rc<Self> {
        Rc::new(Self {
            builder,
            instructions: RefCell::new(vec![Vec::new()]),
            conditions: RefCell::new(vec![condition]),
        })
    }
}

impl RawBuilder for RawConditionBuilder {
    fn push_instruction(&self, instruction: super::Instruction) {
        self.instructions.borrow_mut().last_mut().unwrap().push(instruction);
    }

    fn get_new_id(&self, ty: PrimitiveType) -> usize {
        self.builder.get_new_id(ty)
    }

    fn name_var(&self, ty: PrimitiveType, id: usize, name: String) {
        self.builder.name_var(ty, id, name)
    }
}

impl Drop for RawConditionBuilder {
    fn drop(&mut self) {
        self.builder.push_instruction(super::Instruction::IfChain { 
            conditions: self.conditions.borrow_mut().drain(..).collect(),
            instructions: self.instructions.borrow_mut().drain(..).collect(),
        })
    }
}

pub struct ConditionBuilder {
    /// Always a RawConditionBuilder
    pub(crate) raw: Rc<dyn RawBuilder>
}

impl ConditionBuilder {
    pub fn spv_else(&self, b: &Bool) {
        let t = self.raw.downcast_ref::<RawConditionBuilder>().unwrap();
        t.instructions.borrow_mut().push(Vec::new());
        t.conditions.borrow_mut().push(b.id);
    }

    pub fn end_condition(self) { drop(self) }
}
