
use std::rc::Rc;
use super::RawBuilder;
use crate::data::*;
use std::cell::RefCell;

pub(crate) struct RawConditionBuilder {
    pub(crate) builder: Rc<dyn RawBuilder>,
    pub(crate) instructions: RefCell<Vec<Vec<super::Instruction>>>,
    pub(crate) conditions: RefCell<Vec<usize>>,
    pub(crate) else_instructions: RefCell<Option<Vec<super::Instruction>>>,
}

impl RawConditionBuilder {
    pub fn new(builder: Rc<dyn RawBuilder>, condition: usize) -> Rc<Self> {
        Rc::new(Self {
            builder,
            instructions: RefCell::new(vec![Vec::new()]),
            conditions: RefCell::new(vec![condition]),
            else_instructions: RefCell::new(None),
        })
    }
}

impl RawBuilder for RawConditionBuilder {
    fn push_instruction(&self, instruction: super::Instruction) {
        if let Some(else_instructions) = &mut *self.else_instructions.borrow_mut() {
            else_instructions.push(instruction);
        } else {
            self.instructions.borrow_mut().last_mut().unwrap().push(instruction);
        }
    }

    fn get_new_id(&self) -> usize {
        self.builder.get_new_id()
    }

    fn name_var(&self, id: usize, name: String) {
        self.builder.name_var(id, name)
    }

    fn in_loop(&self) -> bool {
        self.builder.in_loop()
    }
}

impl Drop for RawConditionBuilder {
    fn drop(&mut self) {
        self.builder.push_instruction(super::Instruction::IfChain { 
            conditions: self.conditions.borrow_mut().drain(..).collect(),
            instructions: self.instructions.borrow_mut().drain(..).collect(),
            else_instructions: self.else_instructions.borrow_mut().take(),
        })
    }
}

pub struct ConditionBuilder {
    /// Always a RawConditionBuilder
    pub(crate) raw: Rc<dyn RawBuilder>
}

impl ConditionBuilder {
    pub fn spv_else_if<F: FnOnce(&ConditionBuilder)>(&self, b: impl crate::data::SpvRustEq<Bool>, f: F) {
        let t = self.raw.downcast_ref::<RawConditionBuilder>().unwrap();
        // Important. The id needs to be declared in the super context otherwise the branch
        // instruction can't 
        let id = b.id(&*t.builder);
        t.instructions.borrow_mut().push(Vec::new());
        t.conditions.borrow_mut().push(id);

        f(self);
    }

    pub fn spv_else<F: FnOnce(&ConditionBuilder)>(self, f: F) {
        let t = self.raw.downcast_ref::<RawConditionBuilder>().unwrap();
        *t.else_instructions.borrow_mut() = Some(Vec::new());

        f(&self);
    }

    pub fn end_condition(self) { drop(self) }
}
