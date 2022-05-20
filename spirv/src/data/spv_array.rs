
use std::{marker::PhantomData, rc::Rc};

use crate::builder::RawBaseBuilder;

use super::AsDataType;

pub struct SpvArray<T: AsDataType> {
    builder: Rc<RawBaseBuilder>,
    _marker: PhantomData<T>,
}

