
use std::collections::HashMap;
use super::*;
use std::cell::RefCell;

pub(crate) struct RawBaseBuilder {
    pub(crate) functions: RefCell<HashMap<usize, Vec<Instruction>>>,
    pub(crate) main: RefCell<Vec<Instruction>>,
}

impl RawBaseBuilder {
    pub(crate) fn new() -> Self {
        Self {
            functions: RefCell::new(HashMap::new()),
            main: RefCell::new(Vec::new()),
        }
    }
}