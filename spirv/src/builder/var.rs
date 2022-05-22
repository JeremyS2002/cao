
use slab::Slab;

#[derive(Default, Clone, Debug)]
pub(crate) struct Variables {
    pub(crate) vars: Slab<Option<String>>,
}

impl Variables {
    pub fn get_new_id(&mut self) -> usize {
        self.vars.insert(None)
    }

    pub fn name_var(&mut self, id: usize, name: String) {
        *self.vars.get_mut(id).unwrap() = Some(name);
    }
}