#![allow(dead_code)]

use crate::data::{IsDataType, IsPrimitiveType};
use std::marker::PhantomData;

#[derive(Clone, Copy, Debug)]
pub struct Input<T: IsPrimitiveType> {
    pub(crate) index: usize,
    pub(crate) _marker: PhantomData<T>,
}

#[derive(Clone, Copy, Debug)]
pub struct Output<T: IsPrimitiveType> {
    pub(crate) index: usize,
    pub(crate) _marker: PhantomData<T>,
}


/// A Uniform buffer
/// 
/// Uniform<T> is roughly equivalent to:
/// ```c
/// struct T { .. }
/// 
/// layout(set = _, binding = _) uniform UT {
///     T t;
/// };
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Uniform<T: IsDataType> {
    pub(crate) index: usize,
    pub(crate) _marker: PhantomData<T>,
}

#[derive(Clone, Copy, Debug)]
pub struct StorageAccessDesc {
    pub read: bool,
    pub write: bool,
    pub atomic: bool,
}

/// A Storage buffer
/// 
/// Storage<T> is roughly equivalent to:
/// ```c
/// struct T { .. }
/// 
/// layout(set = _, binding = _) buffer ST {
///     T ts[];
/// };
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Storage<T: IsDataType> {
    pub(crate) index: usize,
    pub(crate) _marker: PhantomData<T>,
}
