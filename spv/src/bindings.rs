
use crate::FromId;
use crate::SpvRustEq;

use std::sync::Arc;
use std::sync::Mutex;
use std::marker::PhantomData;

pub struct PushConstants<T: crate::IsTypeConst> {
    pub(crate) b: Arc<Mutex<crate::BuilderInner>>,
    pub(crate) marker: PhantomData<T>,
}

impl<T: crate::IsTypeConst> PushConstants<T> {
    pub fn load<'a>(&'a self) -> T::T<'a> {
        let mut inner = self.b.lock().unwrap();
        if let Some(scope) = &mut inner.scope {
            let new_id = scope.get_new_id();

            scope.push_instruction(crate::Instruction::LoadStore(crate::OpLoadStore {
                ty: T::TY,
                src: crate::OpLoadStoreData::PushConstant,
                dst: crate::OpLoadStoreData::Variable { id: new_id },
            }));

            T::T::from_id(new_id, &self.b)
        } else {
            panic!("Cannot load push constants when not in function");
        }
    }
}

impl<T: crate::IsTypeConst + crate::IsStructTypeConst> PushConstants<T> {
    pub fn load_field_by_index<'a, R: crate::IsTypeConst>(&'a self, field: u32) -> R::T<'a> {
        let mut inner = self.b.lock().unwrap();
        if let Some(scope) = &mut inner.scope {
            let new_id = scope.get_new_id();

            scope.push_instruction(crate::Instruction::LoadStore(crate::OpLoadStore {
                ty: T::TY,
                src: crate::OpLoadStoreData::PushConstantField { field },
                dst: crate::OpLoadStoreData::Variable { id: new_id },
            }));

            R::T::from_id(new_id, &self.b)
        } else {
            panic!("Cannot load uniform when not in function");
        }
    }

    pub fn load_field<'a, R: crate::IsTypeConst>(&'a self, field: &str) -> R::T<'a> {
        let field = T::STRUCT_TY
            .members
            .iter()
            .enumerate()
            .find(|(_, m)| if let Some(n) = &m.name {
                match n {
                    either::Either::Left(s) => *s == field,
                    either::Either::Right(s) => &**s == field,
                }
            } else {
                false
            }).expect(&format!("No field by name {} on struct", field)).0;
        self.load_field_by_index::<R>(field as u32)
    }
}

pub struct Uniform<T: crate::IsTypeConst> {
    pub(crate) id: usize,
    pub(crate) b: Arc<Mutex<crate::BuilderInner>>,
    pub(crate) marker: PhantomData<T>,
}

impl<T: crate::IsTypeConst> Uniform<T> {
    pub fn load<'a>(&'a self) -> T::T<'a> {
        let mut inner = self.b.lock().unwrap();
        if let Some(scope) = &mut inner.scope {
            let new_id = scope.get_new_id();

            scope.push_instruction(crate::Instruction::LoadStore(crate::OpLoadStore {
                ty: T::TY,
                src: crate::OpLoadStoreData::Uniform { id: self.id },
                dst: crate::OpLoadStoreData::Variable { id: new_id },
            }));

            T::T::from_id(new_id, &self.b)
        } else {
            panic!("Cannot load uniform when not in function");
        }
    }
}

impl<T: crate::IsTypeConst + crate::IsStructTypeConst> Uniform<T> {
    pub fn load_field_by_index<'a, R: crate::IsTypeConst>(&'a self, field: u32) -> R::T<'a> {
        let mut inner = self.b.lock().unwrap();
        if let Some(scope) = &mut inner.scope {
            let new_id = scope.get_new_id();

            scope.push_instruction(crate::Instruction::LoadStore(crate::OpLoadStore {
                ty: T::TY,
                src: crate::OpLoadStoreData::UniformField { field, id: self.id },
                dst: crate::OpLoadStoreData::Variable { id: new_id },
            }));

            R::T::from_id(new_id, &self.b)
        } else {
            panic!("Cannot load uniform when not in function");
        }
    }

    pub fn load_field<'a, R: crate::IsTypeConst>(&'a self, field: &str) -> R::T<'a> {
        let field = T::STRUCT_TY
            .members
            .iter()
            .enumerate()
            .find(|(_, m)| if let Some(n) = &m.name {
                match n {
                    either::Either::Left(s) => *s == field,
                    either::Either::Right(s) => &**s == field,
                }
            } else {
                false
            }).expect(&format!("No field by name {} on struct", field)).0;
        self.load_field_by_index::<R>(field as u32)
    }
}

pub struct Storage<T: crate::IsTypeConst> {
    pub(crate) id: usize,
    pub(crate) b: Arc<Mutex<crate::BuilderInner>>,
    pub(crate) marker: PhantomData<T>,
}

impl<T: crate::IsTypeConst> Storage<T> {
    pub fn load_element<'a>(&'a self, element: impl SpvRustEq<crate::Int<'a>>) -> T::T<'a> {
        let mut inner = self.b.lock().unwrap();
        if let Some(scope) = &mut inner.scope {
            let new_id = scope.get_new_id();

            let element_id = element.id(&mut **scope);
            let element_ty = element.ty();

            scope.push_instruction(crate::Instruction::LoadStore(crate::OpLoadStore {
                ty: T::TY,
                src: crate::OpLoadStoreData::StorageElement { id: self.id, element: (element_id, element_ty) },
                dst: crate::OpLoadStoreData::Variable { id: new_id },
            }));

            T::T::from_id(new_id, &self.b)
        } else {
            panic!("Cannot load storage element when not in function");
        }
    }
}

impl<T: crate::IsTypeConst + crate::IsStructTypeConst> Storage<T> {
    pub fn load_field_by_index<'a, R: crate::IsTypeConst>(&'a self, element: impl SpvRustEq<crate::Int<'a>>, field: u32) -> R::T<'a> {
        let mut inner = self.b.lock().unwrap();
        if let Some(scope) = &mut inner.scope {
            let new_id = scope.get_new_id();

            let element_id = element.id(&mut **scope);
            let element_ty = element.ty();

            scope.push_instruction(crate::Instruction::LoadStore(crate::OpLoadStore {
                ty: T::TY,
                src: crate::OpLoadStoreData::StorageElementField { id: self.id, element: (element_id, element_ty), field },
                dst: crate::OpLoadStoreData::Variable { id: new_id },
            }));

            R::T::from_id(new_id, &self.b)
        } else {
            panic!("Cannot load storage element by index {} when not in function", field);
        }
    }

    pub fn load_field<'a, R: crate::IsTypeConst>(&'a self, element: impl SpvRustEq<crate::Int<'a>>, field: &str) -> R::T<'a> {
        let field = T::STRUCT_TY
            .members
            .iter()
            .enumerate()
            .find(|(_, m)| if let Some(n) = &m.name {
                match n {
                    either::Either::Left(s) => *s == field,
                    either::Either::Right(s) => &**s == field,
                }
            } else {
                false
            }).expect(&format!("No field by name {} on struct", field)).0;
        self.load_field_by_index::<R>(element, field as u32)
    }
}
