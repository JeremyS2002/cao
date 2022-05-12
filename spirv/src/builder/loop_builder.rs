
use std::cell::RefCell;
use super::*;

pub(crate) struct RawLoopBuilder {
    pub(crate) builder: Rc<dyn RawBuilder>,
    pub(crate) instructions: RefCell<Vec<super::Instruction>>,
    pub(crate) condition: usize,
}

impl RawLoopBuilder {
    pub fn new(builder: Rc<dyn RawBuilder>, condition: usize) -> Rc<Self> {
        Rc::new(Self {
            builder,
            instructions: RefCell::new(Vec::new()),
            condition,
        })
    }
}

impl RawBuilder for RawLoopBuilder {
    fn push_instruction(&self, instruction: Instruction) {
        self.instructions.borrow_mut().push(instruction);
    }

    fn get_new_id(&self, ty: DataType) -> usize {
        self.builder.get_new_id(ty)
    }

    fn name_var(&self, ty: DataType, id: usize, name: String) {
        self.builder.name_var(ty, id, name)
    }
}

impl Drop for RawLoopBuilder {
    fn drop(&mut self) {
        self.builder.push_instruction(super::Instruction::Loop { 
            condition: self.condition, 
            body: self.instructions.borrow_mut().drain(..).collect(), 
        })
    }
}

pub struct LoopBuilder {
    pub(crate) raw: Rc<dyn RawBuilder>
}
