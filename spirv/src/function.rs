
use std::marker::PhantomData;

pub struct Function<R> {
    pub(crate) id: usize,
    pub(crate) _marker: PhantomData<R>,
}