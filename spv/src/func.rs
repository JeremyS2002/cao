
use crate::FromId;

use std::sync::Arc;
use std::sync::Mutex;
use std::marker::PhantomData;


pub struct Func<T: crate::IsTypeConst> {
    pub(crate) id: usize,
    pub(crate) inner: Arc<Mutex<crate::BuilderInner>>,
    pub(crate) marker: PhantomData<T>,
}

impl<T: crate::IsTypeConst> Func<T> {
    pub fn call<'a>(&'a self, args: impl IntoIterator<Item=&'a dyn crate::AsType>) -> T::T<'a> {
        let mut inner = self.inner.lock().unwrap();
        if let Some(scope) = &mut inner.scope {
            let new_id = scope.get_new_id();

            let args = args.into_iter()
                .map(|t| (t.id(&mut **scope), t.ty()))
                .collect::<Vec<_>>();

            scope.push_instruction(crate::Instruction::FuncCall(crate::OpFuncCall {
                func: self.id,
                store_ty: T::TY,
                store: new_id,
                args,
            }));

            drop(scope);
            drop(inner);
        
            T::T::from_id(new_id, &self.inner)
        } else {
            panic!("Cannot call function when not in function")
        }
    }
}
