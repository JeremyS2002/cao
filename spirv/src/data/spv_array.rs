
use std::{marker::PhantomData, rc::Rc};

use crate::builder::RawBaseBuilder;

use super::AsDataType;

pub struct SpvArray<T: AsDataType> {
    pub(crate) builder: Rc<RawBaseBuilder>,
    pub(crate) _marker: PhantomData<T>,
}

