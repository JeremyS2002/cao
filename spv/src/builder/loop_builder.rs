use super::*;
use std::cell::RefCell;

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

    fn get_new_id(&self) -> usize {
        self.builder.get_new_id()
    }

    fn name_var(&self, id: usize, name: String) {
        self.builder.name_var(id, name)
    }

    fn in_loop(&self) -> bool {
        true
    }

    fn push_constant(&self) -> Option<(DataType, u32, Option<&'static str>)> {
        self.builder.push_constant()
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
    pub(crate) raw: Rc<dyn RawBuilder>,
}
