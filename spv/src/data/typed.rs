
use either::*;
use std::marker::PhantomData;
use std::rc::Rc;
use std::cell::RefCell;

#[rustfmt::skip]
use crate::{
    GlamIVec2,
    GlamIVec3,
    GlamIVec4,
    GlamUVec2,
    GlamUVec3,
    GlamUVec4,
    GlamVec2,
    GlamVec3,
    GlamVec4,
    GlamDVec2,
    GlamDVec3, 
    GlamDVec4,
    GlamMat2,
    GlamMat3,
    GlamMat4,
    GlamDMat2,
    GlamDMat3,
    GlamDMat4,
};

pub trait SpvRustEq<T>: AsType {
    fn as_ty<'a>(&'a self) -> &'a dyn AsType;
}

pub trait AsScalarTypeConst {
    const SCALAR_TY: crate::ScalarType;
}

pub trait IsScalarTypeConst: AsScalarTypeConst { }

pub trait AsScalarType {
    fn scalar_ty(&self) -> crate::ScalarType;

    fn scalar_id(&self, s: &mut dyn crate::Scope) -> usize;

    fn as_scalar_ty_ref<'a>(&'a self) -> &'a dyn AsScalarType;
}

pub trait IsScalarType: AsScalarType { }

pub trait AsVectorTypeConst {
    const VECTOR_TY: crate::VectorType;
}

pub trait IsVectorTypeConst: AsVectorTypeConst { }

pub trait AsVectorType {
    fn vector_ty(&self) -> crate::VectorType;

    fn vector_id(&self, s: &mut dyn crate::Scope) -> usize;

    fn as_vector_ty_ref<'a>(&'a self) -> &'a dyn AsVectorType;
}

pub trait IsVectorType: AsVectorType { }

pub trait AsMatrixTypeConst {
    const MATRIX_TY: crate::MatrixType;
}

pub trait IsMatrixTypeConst: AsMatrixTypeConst { }

pub trait AsMatrixType {
    fn matrix_ty(&self) -> crate::MatrixType;

    fn matrix_id(&self, s: &mut dyn crate::Scope) -> usize;

    fn as_matrix_ty_ref<'a>(&'a self) -> &'a dyn AsMatrixType;
}

pub trait IsMatrixType: AsMatrixType { }

pub trait AsTypeConst {
    const TY: crate::Type;
}

pub trait IsTypeConst: AsTypeConst {
    type T<'a>: FromId<'a>;
}

pub trait FromId<'a>: IsTypeConst {
    fn from_id(id: usize, b: &'a Rc<RefCell<crate::BuilderInner>>) -> Self;
}

pub trait AsType {
    fn ty(&self) -> crate::Type;

    fn id(&self, s: &mut dyn crate::Scope) -> usize;

    fn as_ty_ref<'a>(&'a self) -> &'a dyn AsType;

    // fn as_ty_mut<'a>(&'a self) -> &'a mut dyn AsType;

    // fn as_ty_box<'a>(self: Box<Self>) -> Box<dyn AsType>;
}

pub trait IsType: AsType { }

pub struct Void { }

impl AsTypeConst for Void {
    const TY: crate::Type = crate::Type::Void;
}

impl IsTypeConst for Void { 
    type T<'a> = Void;
}

impl<'a> FromId<'a> for Void {
    fn from_id(_: usize, _: &'a Rc<RefCell<crate::BuilderInner>>) -> Self {
        panic!("Cannot construct instance of void");
    }
}

// impl scalar types
// ================================================================================
// ================================================================================
// ================================================================================

macro_rules! impl_scalar_ty {
    ($($name:ident, $rust:ident, $t:ident$(($l:literal))*,)*) => {
        $(

            #[derive(Clone, Copy)]
            pub struct $name<'a> {
                pub(crate) id: usize,
                pub(crate) b: &'a Rc<RefCell<crate::BuilderInner>>,
            }

            impl<'a> AsScalarTypeConst for $name<'a> {
                const SCALAR_TY: crate::ScalarType = crate::ScalarType::$t$(($l))*;
            }

            impl<'a> IsScalarTypeConst for $name<'a> { }

            impl<'a> AsScalarType for $name<'a> {
                fn scalar_ty(&self) -> crate::ScalarType {
                    <Self as AsScalarTypeConst>::SCALAR_TY
                }

                fn scalar_id(&self, _: &mut dyn crate::Scope) -> usize {
                    self.id
                }

                fn as_scalar_ty_ref<'b>(&'b self) -> &'b dyn AsScalarType {
                    self
                }
            }

            impl<'a> IsScalarType for $name<'a> { }

            impl<'a> AsTypeConst for $name<'a> {
                const TY: crate::Type = crate::Type::Scalar(Self::SCALAR_TY);
            }

            impl<'a> IsTypeConst for $name<'a> { 
                type T<'b> = $name<'b>;
            }

            impl<'a> FromId<'a> for $name<'a> {
                fn from_id(id: usize, b: &'a Rc<RefCell<crate::BuilderInner>>) -> Self {
                    Self {
                        id,
                        b,
                    }
                }
            }

            impl<'a> AsType for $name<'a> {
                fn ty(&self) -> crate::Type {
                    <Self as AsTypeConst>::TY
                }
            
                fn id(&self, s: &mut dyn crate::Scope) -> usize {
                    self.scalar_id(s)
                }

                fn as_ty_ref<'b>(&'b self) -> &'b dyn AsType {
                    self
                }
            }

            impl<'a> IsType for $name<'a> { }

            impl<'a, 'b> SpvRustEq<$name<'b>> for $name<'a> { 
                fn as_ty<'c>(&'c self) -> &'c dyn AsType {
                    self
                }
            }

            impl<'a> SpvRustEq<$name<'a>> for $rust {
                fn as_ty<'b>(&'b self) -> &'b dyn AsType {
                    self
                }
            }

            impl AsScalarTypeConst for $rust {
                const SCALAR_TY: crate::ScalarType = crate::ScalarType::$t$(($l))*;
            }

            impl AsScalarType for $rust {
                fn scalar_ty(&self) -> crate::ScalarType {
                    <Self as AsScalarTypeConst>::SCALAR_TY
                }

                fn scalar_id(&self, s: &mut dyn crate::Scope) -> usize {
                    let val = crate::ScalarVal::$name(*self);
                    let new_id = s.get_new_id();
                    s.push_instruction(crate::Instruction::SetConst(crate::OpSetConst {
                        val: crate::Val::Scalar(val),
                        store: new_id,
                    }));
                    new_id
                }

                fn as_scalar_ty_ref<'b>(&'b self) -> &'b dyn AsScalarType {
                    self
                }
            }

            impl AsTypeConst for $rust {
                const TY: crate::Type = crate::Type::Scalar(Self::SCALAR_TY);
            }
            
            impl AsType for $rust {
                fn ty(&self) -> crate::Type {
                    <Self as AsTypeConst>::TY
                }
            
                fn id(&self, s: &mut dyn crate::Scope) -> usize {
                    self.scalar_id(s)
                }

                fn as_ty_ref<'b>(&'b self) -> &'b dyn AsType {
                    self
                }
            }
        )*
    };
}

#[rustfmt::skip]
impl_scalar_ty!(
    Bool, bool, Bool,
    Int, i32, Signed(32),
    UInt, u32, Unsigned(32),
    Float, f32, Float(32),
    Double, f64, Float(64),
);

// impl vector types
// ================================================================================
// ================================================================================
// ================================================================================

macro_rules! impl_vector_ty {
    ($($name:ident, $rust:ident, $t:ident($d1:literal), $d2:literal,)*) => {
        $(

            #[derive(Clone, Copy)]
            pub struct $name<'a> {
                pub(crate) id: usize,
                pub(crate) b: &'a Rc<RefCell<crate::BuilderInner>>,
            }

            impl<'a> AsVectorTypeConst for $name<'a> {
                const VECTOR_TY: crate::VectorType = crate::VectorType {
                    scalar_ty: crate::ScalarType::$t($d1),
                    n_scalar: $d2,
                };
            }

            impl<'a> IsVectorTypeConst for $name<'a> { }

            impl<'a> AsVectorType for $name<'a> {
                fn vector_ty(&self) -> crate::VectorType {
                    <Self as AsVectorTypeConst>::VECTOR_TY
                }

                fn vector_id(&self, _: &mut dyn crate::Scope) -> usize {
                    self.id
                }

                fn as_vector_ty_ref<'b>(&'b self) -> &'b dyn AsVectorType {
                    self
                }
            }

            impl<'a> IsVectorType for $name<'a> { }

            impl<'a> AsTypeConst for $name<'a> {
                const TY: crate::Type = crate::Type::Vector(Self::VECTOR_TY);
            }

            impl<'a> IsTypeConst for $name<'a> { 
                type T<'b> = $name<'b>;
            }

            impl<'a> FromId<'a> for $name<'a> {
                fn from_id(id: usize, b: &'a Rc<RefCell<crate::BuilderInner>>) -> Self {
                    Self {
                        id,
                        b,
                    }
                }
            }

            impl<'a> AsType for $name<'a> {
                fn ty(&self) -> crate::Type {
                    <Self as AsTypeConst>::TY
                }

                fn id(&self, s: &mut dyn crate::Scope) -> usize {
                    self.vector_id(s)
                }

                fn as_ty_ref<'b>(&'b self) -> &'b dyn AsType {
                    self
                }
            }

            impl<'a> IsType for $name<'a> { }

            impl<'a, 'b> SpvRustEq<$name<'b>> for $name<'a> { 
                fn as_ty<'c>(&'c self) -> &'c dyn AsType {
                    self
                }
            }

            impl<'a> SpvRustEq<$name<'a>> for $rust { 
                fn as_ty<'b>(&'b self) -> &'b dyn AsType {
                    self
                }
            }

            impl AsVectorTypeConst for $rust {
                const VECTOR_TY: crate::VectorType = crate::VectorType {
                    scalar_ty: crate::ScalarType::$t($d1),
                    n_scalar: $d2,
                };
            }

            impl AsVectorType for $rust {
                fn vector_ty(&self) -> crate::VectorType {
                    <Self as AsVectorTypeConst>::VECTOR_TY
                }

                fn vector_id(&self, s: &mut dyn crate::Scope) -> usize {
                    let val = crate::VectorVal::$name(*self);
                    let new_id = s.get_new_id();
                    s.push_instruction(crate::Instruction::SetConst(crate::OpSetConst {
                        val: crate::Val::Vector(val),
                        store: new_id,
                    }));
                    new_id
                }

                fn as_vector_ty_ref<'b>(&'b self) -> &'b dyn AsVectorType {
                    self
                }
            }

            impl AsTypeConst for $rust {
                const TY: crate::Type = crate::Type::Vector(Self::VECTOR_TY);
            }

            impl AsType for $rust {
                fn ty(&self) -> crate::Type {
                    <Self as AsTypeConst>::TY
                }

                fn id(&self, s: &mut dyn crate::Scope) -> usize {
                    self.vector_id(s)
                }

                fn as_ty_ref<'b>(&'b self) -> &'b dyn AsType {
                    self
                }
            }
        )*
    };
}

#[rustfmt::skip]
impl_vector_ty!(
    IVec2, GlamIVec2, Signed(32), 2,
    IVec3, GlamIVec3, Signed(32), 3,
    IVec4, GlamIVec4, Signed(32), 4,
    UVec2, GlamUVec2, Unsigned(32), 2,
    UVec3, GlamUVec3, Unsigned(32), 3,
    UVec4, GlamUVec4, Unsigned(32), 4,
    Vec2, GlamVec2, Float(32), 2,
    Vec3, GlamVec3, Float(32), 3,
    Vec4, GlamVec4, Float(32), 4,
    DVec2, GlamDVec2, Float(64), 2,
    DVec3, GlamDVec3, Float(64), 3,
    DVec4, GlamDVec4, Float(64), 4,
);

// impl matrix types
// ================================================================================
// ================================================================================
// ================================================================================

macro_rules! impl_matrix_ty {
    ($($name:ident, $rust:ident, $t:ident($d1:literal), $d2:literal, $d3:literal,)*) => {
        $(

            #[derive(Clone, Copy)]
            pub struct $name<'a> {
                pub(crate) id: usize,
                pub(crate) b: &'a Rc<RefCell<crate::BuilderInner>>,
            }

            impl<'a> AsMatrixTypeConst for $name<'a> {
                const MATRIX_TY: crate::MatrixType = crate::MatrixType {
                    vec_ty: crate::VectorType {
                        scalar_ty: crate::ScalarType::$t($d1),
                        n_scalar: $d2,
                    },
                    n_vec: $d3,
                };
            }

            impl<'a> IsMatrixTypeConst for $name<'a> { }

            impl<'a> AsMatrixType for $name<'a> {
                fn matrix_ty(&self) -> crate::MatrixType {
                    <Self as AsMatrixTypeConst>::MATRIX_TY
                }

                fn matrix_id(&self, _: &mut dyn crate::Scope) -> usize {
                    self.id
                }

                fn as_matrix_ty_ref<'b>(&'b self) -> &'b dyn AsMatrixType {
                    self
                }
            }

            impl<'a> IsMatrixType for $name<'a> { }

            impl<'a> AsTypeConst for $name<'a> {
                const TY: crate::Type = crate::Type::Matrix(Self::MATRIX_TY);
            }
            
            impl<'a> IsTypeConst for $name<'a> { 
                type T<'b> = $name<'b>;
            }

            impl<'a> FromId<'a> for $name<'a> {
                fn from_id(id: usize, b: &'a Rc<RefCell<crate::BuilderInner>>) -> Self {
                    Self {
                        id,
                        b,
                    }
                }
            }

            impl<'a> AsType for $name<'a> {
                fn ty(&self) -> crate::Type {
                    <Self as AsTypeConst>::TY
                }
            
                fn id(&self, s: &mut dyn crate::Scope) -> usize {
                    self.matrix_id(s)
                }

                fn as_ty_ref<'b>(&'b self) -> &'b dyn AsType {
                    self
                }
            }

            impl<'a> IsType for $name<'a> { }

            impl<'a, 'b> SpvRustEq<$name<'b>> for $name<'a> { 
                fn as_ty<'c>(&'c self) -> &'c dyn AsType {
                    self
                }
            }

            impl<'a> SpvRustEq<$name<'a>> for $rust { 
                fn as_ty<'b>(&'b self) -> &'b dyn AsType {
                    self
                }
            }

            impl AsMatrixTypeConst for $rust {
                const MATRIX_TY: crate::MatrixType = crate::MatrixType {
                    vec_ty: crate::VectorType {
                        scalar_ty: crate::ScalarType::$t($d1),
                        n_scalar: $d2,
                    },
                    n_vec: $d3,
                };
            }

            impl AsMatrixType for $rust {
                fn matrix_ty(&self) -> crate::MatrixType {
                    <Self as AsMatrixTypeConst>::MATRIX_TY
                }

                fn matrix_id(&self, s: &mut dyn crate::Scope) -> usize {
                    let val = crate::MatrixVal::$name(*self);
                    let new_id = s.get_new_id();
                    s.push_instruction(crate::Instruction::SetConst(crate::OpSetConst {
                        val: crate::Val::Matrix(val),
                        store: new_id,
                    }));
                    new_id
                }

                fn as_matrix_ty_ref<'b>(&'b self) -> &'b dyn AsMatrixType {
                    self
                }
            }

            impl AsTypeConst for $rust {
                const TY: crate::Type = crate::Type::Matrix(Self::MATRIX_TY);
            }
            
            impl AsType for $rust {
                fn ty(&self) -> crate::Type {
                    <Self as AsTypeConst>::TY
                }
            
                fn id(&self, s: &mut dyn crate::Scope) -> usize {
                    self.matrix_id(s)
                }

                fn as_ty_ref<'b>(&'b self) -> &'b dyn AsType {
                    self
                }
            }
        )*
    };
}

#[rustfmt::skip]
impl_matrix_ty!(
    Mat2, GlamMat2, Float(32), 2, 2,
    Mat3, GlamMat3, Float(32), 3, 3,
    Mat4, GlamMat4, Float(32), 4, 4,
    DMat2, GlamDMat2, Float(64), 2, 2,
    DMat3, GlamDMat3, Float(64), 3, 3,
    DMat4, GlamDMat4, Float(64), 4, 4,
);

// impl convert
// ================================================================================
// ================================================================================
// ================================================================================

macro_rules! impl_convert {
    ($($dst:ident, $src:ident,)*) => {
        $(
            impl<'a> std::convert::From::<$src<'a>> for $dst<'a> {
                fn from(v: $src<'a>) -> Self {
                    let mut b = v.b.borrow_mut();
                    if let Some(scope) = &mut b.scope {
                        let new_id = scope.get_new_id();

                        scope.push_instruction(crate::Instruction::Convert(crate::OpConvert {
                            src: (v.id, v.ty()),
                            dst: (new_id, <Int as AsTypeConst>::TY),
                        }));

                        drop(scope);
                        drop(b);

                        $dst {
                            id: new_id,
                            b: v.b,
                        }
                    } else {
                        panic!("Cannot convert types when not in function")
                    }
                }
            }
        )*
    };
}

#[rustfmt::skip]
impl_convert!(
    Int, UInt,
    Int, Float,
    Int, Double,
    UInt, Int,
    UInt, Float,
    UInt, Double,
    Float, Int,
    Float, UInt,
    Float, Double,
    Double, Int,
    Double, UInt,
    Double, Float,
);

// store
// ================================================================================
// ================================================================================
// ================================================================================

macro_rules! impl_store {
    ($($name:ident,)*) => {
        $(
            impl<'a> $name<'a> {
                pub fn store<'b>(&mut self, v: $name<'b>) {
                    let mut inner = self.b.borrow_mut();
                    if let Some(scope) = &mut inner.scope {
                        scope.push_instruction(crate::Instruction::LoadStore(crate::OpLoadStore {
                            ty: <Self as AsTypeConst>::TY,
                            src: crate::OpLoadStoreData::Variable { id: v.id },
                            dst: crate::OpLoadStoreData::Variable { id: self.id }
                        }));
                    } else {
                        panic!("cannot store variable when not in function")
                    }
                }
            }
        )*
    };
}

impl_store!(
    Bool,
    Int,
    Float,
    Double,
    IVec2,
    IVec3,
    IVec4,
    UVec2,
    UVec3,
    UVec4,
    Vec2,
    Vec3,
    Vec4,
    DVec2,
    DVec3,
    DVec4,
    Mat2,
    Mat3,
    Mat4,
    DMat2,
    DMat3,
    DMat4,
);  

// bool ops
// ================================================================================
// ================================================================================
// ================================================================================

impl<'a, 'b> std::ops::BitAnd<Bool<'b>> for Bool<'a> {
    type Output = Bool<'a>;

    fn bitand(self, rhs: Bool<'b>) -> Self::Output {
        let mut b = self.b.borrow_mut();
        let id = basic_op(&mut b, &self, &rhs, crate::Type::BOOL, crate::OpLhsRhsType::LogicalAnd);
        Bool {
            id,
            b: self.b
        }
    }
}

impl<'a> std::ops::BitAnd<bool> for Bool<'a> {
    type Output = Bool<'a>;

    fn bitand(self, rhs: bool) -> Self::Output {
        let mut b = self.b.borrow_mut();
        let id = basic_op(&mut b, &self, &rhs, crate::Type::BOOL, crate::OpLhsRhsType::LogicalAnd);
        Bool {
            id,
            b: self.b
        }
    }
}

impl<'a> std::ops::BitAnd<Bool<'a>> for bool {
    type Output = Bool<'a>;

    fn bitand(self, rhs: Bool<'a>) -> Self::Output {
        let mut b = rhs.b.borrow_mut();
        let id = basic_op(&mut b, &self, &rhs, crate::Type::BOOL, crate::OpLhsRhsType::LogicalAnd);
        Bool {
            id,
            b: rhs.b
        }
    }
}

impl<'a, 'b> std::ops::BitOr<Bool<'b>> for Bool<'a> {
    type Output = Bool<'a>;

    fn bitor(self, rhs: Bool<'b>) -> Self::Output {
        let mut b = self.b.borrow_mut();
        let id = basic_op(&mut b, &self, &rhs, crate::Type::BOOL, crate::OpLhsRhsType::LogicalOr);
        Bool {
            id,
            b: self.b
        }
    }
}

impl<'a> std::ops::BitOr<bool> for Bool<'a> {
    type Output = Bool<'a>;

    fn bitor(self, rhs: bool) -> Self::Output {
        let mut b = self.b.borrow_mut();
        let id = basic_op(&mut b, &self, &rhs, crate::Type::BOOL, crate::OpLhsRhsType::LogicalOr);
        Bool {
            id,
            b: self.b
        }
    }
}

impl<'a> std::ops::BitOr<Bool<'a>> for bool {
    type Output = Bool<'a>;

    fn bitor(self, rhs: Bool<'a>) -> Self::Output {
        let mut b = rhs.b.borrow_mut();
        let id = basic_op(&mut b, &self, &rhs, crate::Type::BOOL, crate::OpLhsRhsType::LogicalOr);
        Bool {
            id,
            b: rhs.b
        }
    }
}

impl<'a> Bool<'a> {
    fn cmp(&self, rhs: impl SpvRustEq<Bool<'a>>, ty: crate::OpLhsRhsType) -> Bool<'a> {
        let mut inner = self.b.borrow_mut();
        if let Some(scope) = &mut inner.scope {
            let new_id = scope.get_new_id();

            let rhs_id = rhs.id(&mut **scope);
            let rhs_ty = rhs.ty();

            scope.push_instruction(crate::Instruction::LhsRhs(crate::OpLhsRhs {
                ty,
                lhs: (self.id, <Self as AsTypeConst>::TY),
                rhs: (rhs_id, rhs_ty),
                store: (new_id, crate::Type::BOOL),
            }));

            drop(scope);
            drop(inner);

            Bool {
                id: new_id,
                b: self.b,
            }
        } else {
            panic!("Cannot perform logical comparison when not in function");
        }
    }

    pub fn eq(&self, rhs: impl SpvRustEq<Bool<'a>>) -> Bool<'a> {
        self.cmp(rhs, crate::OpLhsRhsType::LogicalEqual)
    }

    pub fn neq(&self, rhs: impl SpvRustEq<Bool<'a>>) -> Bool<'a> {
        self.cmp(rhs, crate::OpLhsRhsType::LogicalNotEqual)
    }
}

// impl ops Add, Sub, Mul, Div
// ================================================================================
// ================================================================================
// ================================================================================

fn basic_op(b: &mut crate::BuilderInner, lhs: &dyn AsType, rhs: &dyn AsType, store: crate::Type, op: crate::OpLhsRhsType) -> usize {
    let id = if let Some(scope) = &mut b.scope {
        let lhs_id = lhs.id(&mut **scope);
        let lhs_ty = lhs.ty();
        let rhs_id = rhs.id(&mut **scope);
        let rhs_ty = rhs.ty();
        let store_id = scope.get_new_id();
        // never add two things and get different type
        scope.push_instruction(crate::Instruction::LhsRhs(crate::OpLhsRhs {
            ty: op,
            lhs: (lhs_id, lhs_ty),
            rhs: (rhs_id, rhs_ty),
            store: (store_id, store),
        }));
        store_id
    } else {
        panic!("Cannot perform op {:?} from builder no in scope", op);
    };
    id
}

fn add(b: &mut crate::BuilderInner, lhs: &dyn AsType, rhs: &dyn AsType, store: crate::Type) -> usize {
    basic_op(b, lhs, rhs, store, crate::OpLhsRhsType::Add)
}

fn sub(b: &mut crate::BuilderInner, lhs: &dyn AsType, rhs: &dyn AsType, store: crate::Type) -> usize {
    basic_op(b, lhs, rhs, store, crate::OpLhsRhsType::Sub)
}

fn mul(b: &mut crate::BuilderInner, lhs: &dyn AsType, rhs: &dyn AsType, store: crate::Type) -> usize {
    basic_op(b, lhs, rhs, store, crate::OpLhsRhsType::Mul)
}

fn div(b: &mut crate::BuilderInner, lhs: &dyn AsType, rhs: &dyn AsType, store: crate::Type) -> usize {
    basic_op(b, lhs, rhs, store, crate::OpLhsRhsType::Div)
}

fn assign_op(b: &mut crate::BuilderInner, lhs: &dyn AsType, rhs: &dyn AsType, store: crate::Type, op: crate::OpLhsRhsType) {
    if let Some(scope) = &mut b.scope {
        let lhs_id = lhs.id(&mut **scope);
        let lhs_ty = lhs.ty();
        let rhs_id = rhs.id(&mut **scope);
        let rhs_ty = rhs.ty();
        let store_id = lhs_id;
        scope.push_instruction(crate::Instruction::LhsRhs(crate::OpLhsRhs {
            ty: op,
            lhs: (lhs_id, lhs_ty),
            rhs: (rhs_id, rhs_ty),
            store: (store_id, store),
        }));
    } else {
        panic!("Cannot perform op {:?} from builder no in scope", op);
    };
}

fn add_assign(b: &mut crate::BuilderInner, lhs: &dyn AsType, rhs: &dyn AsType, store: crate::Type) {
    assign_op(b, lhs, rhs, store, crate::OpLhsRhsType::Add)
}

fn sub_assign(b: &mut crate::BuilderInner, lhs: &dyn AsType, rhs: &dyn AsType, store: crate::Type) {
    assign_op(b, lhs, rhs, store, crate::OpLhsRhsType::Sub)
}

fn mul_assign(b: &mut crate::BuilderInner, lhs: &dyn AsType, rhs: &dyn AsType, store: crate::Type) {
    assign_op(b, lhs, rhs, store, crate::OpLhsRhsType::Mul)
}

fn div_assign(b: &mut crate::BuilderInner, lhs: &dyn AsType, rhs: &dyn AsType, store: crate::Type) {
    assign_op(b, lhs, rhs, store, crate::OpLhsRhsType::Div)
}

macro_rules! impl_op {
    ($name:ident, $rust:ident, $op:ident, $f:ident, $store:ident) => {
        impl<'a, 'b> std::ops::$op<$name<'b>> for $name<'a> {
            type Output = $name<'a>;

            fn $f(self, rhs: $name<'b>) -> Self::Output {
                let mut b = self.b.borrow_mut();
                let id = $f(&mut b, &self, &rhs, crate::Type::$store);
                $name {
                    id,
                    b: self.b
                }
            }
        }

        impl<'a> std::ops::$op<$rust> for $name<'a> {
            type Output = $name<'a>;

            fn $f(self, rhs: $rust) -> Self::Output {
                let mut b = self.b.borrow_mut();
                let id = $f(&mut b, &self, &rhs, crate::Type::$store);
                $name {
                    id,
                    b: self.b
                }
            }
        }

        impl<'a> std::ops::$op<$name<'a>> for $rust {
            type Output = $name<'a>;

            fn $f(self, rhs: $name<'a>) -> Self::Output {
                let mut b = rhs.b.borrow_mut();
                let id = $f(&mut b, &self, &rhs, crate::Type::$store);
                $name {
                    id,
                    b: rhs.b
                }
            }
        }
    };
}

macro_rules! impl_basic_ops {
    ($name:ident, $rust:ident, $store:ident) => {
        impl_op!($name, $rust, Add, add, $store);
        impl_op!($name, $rust, Sub, sub, $store);
        impl_op!($name, $rust, Mul, mul, $store);
        impl_op!($name, $rust, Div, div, $store);            
    };
}

macro_rules! impl_assign_op {
    ($name:ident, $rust:ident, $op:ident, $f:ident, $store:ident) => {
        impl<'a, 'b> std::ops::$op<$name<'b>> for $name<'a> {
            fn $f(&mut self, rhs: $name<'b>) {
                let mut b = self.b.borrow_mut();
                $f(&mut b, &*self, &rhs, crate::Type::$store)
            }
        }
    };
}

macro_rules! impl_assign_ops {
    ($name:ident, $rust:ident, $store:ident) => {
        impl_assign_op!($name, $rust, AddAssign, add_assign, $store);
        impl_assign_op!($name, $rust, SubAssign, sub_assign, $store);
        impl_assign_op!($name, $rust, MulAssign, mul_assign, $store);
        impl_assign_op!($name, $rust, DivAssign, div_assign, $store);
    };
}

macro_rules! impl_ops {
    ($($name:ident, $rust:ident, $store:ident,)*) => {
        $(
            impl_basic_ops!($name, $rust, $store);
            impl_assign_ops!($name, $rust, $store);
        )*
    };
}

#[rustfmt::skip]
impl_ops!(
    Int, i32, INT,
    UInt, u32, UINT,
    Float, f32, FLOAT,
    Double, f64, DOUBLE,
    IVec2, GlamIVec2, IVEC2,
    IVec3, GlamIVec3, IVEC3,
    IVec4, GlamIVec4, IVEC4,
    UVec2, GlamUVec2, UVEC2,
    UVec3, GlamUVec3, UVEC3,
    UVec4, GlamUVec4, UVEC4,
    Vec2, GlamVec2, VEC2,
    Vec3, GlamVec3, VEC3,
    Vec4, GlamVec4, VEC4,
    DVec2, GlamDVec2, DVEC2, 
    DVec3, GlamDVec3, DVEC3,
    DVec4, GlamDVec4, DVEC4,
    Mat2, GlamMat2, MAT2,
    Mat3, GlamMat3, MAT3,
    Mat4, GlamMat4, MAT4,
    DMat2, GlamDMat2, DMAT2,
    DMat3, GlamDMat3, DMAT3,
    DMat4, GlamDMat4, DMAT4,
);

macro_rules! impl_scalar_vec_op {
    ($scalar:ident, $rust_scalar:ident, $vec:ident, $rust_vec:ident, $op:ident, $f:ident, $store:ident) => {
        impl<'a, 'b> std::ops::$op<$scalar<'b>> for $vec<'a> {
            type Output = $vec<'a>;

            fn $f(self, rhs: $scalar<'b>) -> Self::Output {
                let mut b = self.b.borrow_mut();
                let id = $f(&mut b, &self, &rhs, crate::Type::$store);
                $vec {
                    id,
                    b: self.b
                }
            }
        }

        impl<'a, 'b> std::ops::$op<$vec<'b>> for $scalar<'a> {
            type Output = $vec<'a>;

            fn $f(self, rhs: $vec<'b>) -> Self::Output {
                let mut b = self.b.borrow_mut();
                let id = $f(&mut b, &self, &rhs, crate::Type::$store);
                $vec {
                    id,
                    b: self.b
                }
            }
        }

        impl<'a> std::ops::$op<$rust_scalar> for $vec<'a> {
            type Output = $vec<'a>;

            fn $f(self, rhs: $rust_scalar) -> Self::Output {
                let mut b = self.b.borrow_mut();
                let id = $f(&mut b, &self, &rhs, crate::Type::$store);
                $vec {
                    id,
                    b: self.b
                }
            }
        }

        impl<'a> std::ops::$op<$vec<'a>> for $rust_scalar {
            type Output = $vec<'a>;

            fn $f(self, rhs: $vec<'a>) -> Self::Output {
                let mut b = rhs.b.borrow_mut();
                let id = $f(&mut b, &self, &rhs, crate::Type::$store);
                $vec {
                    id,
                    b: rhs.b
                }
            }
        }

        impl<'a> std::ops::$op<$scalar<'a>> for $rust_vec {
            type Output = $vec<'a>;

            fn $f(self, rhs: $scalar<'a>) -> Self::Output {
                let mut b = rhs.b.borrow_mut();
                let id = $f(&mut b, &self, &rhs, crate::Type::$store);
                $vec {
                    id,
                    b: rhs.b
                }
            }
        }

        impl<'a> std::ops::$op<$rust_vec> for $scalar<'a> {
            type Output = $vec<'a>;

            fn $f(self, rhs: $rust_vec) -> Self::Output {
                let mut b = self.b.borrow_mut();
                let id = $f(&mut b, &self, &rhs, crate::Type::$store);
                $vec {
                    id,
                    b: self.b
                }
            }
        }
    };
}

macro_rules! impl_scalar_vec_ops {
    ($($scalar:ident, $rust_scalar:ident, $vec:ident, $rust_vec:ident, $store:ident,)*) => {
        $(
            impl_scalar_vec_op!($scalar, $rust_scalar, $vec, $rust_vec, Mul, mul, $store);
            impl_scalar_vec_op!($scalar, $rust_scalar, $vec, $rust_vec, Div, div, $store);
        )*
    };
}

#[rustfmt::skip]
impl_scalar_vec_ops!(
    Int, i32, IVec2, GlamIVec2, IVEC2,
    Int, i32, IVec3, GlamIVec3, IVEC3,
    Int, i32, IVec4, GlamIVec4, IVEC4,
    UInt, u32, UVec2, GlamUVec2, UVEC2,
    UInt, u32, UVec3, GlamUVec3, UVEC3,
    UInt, u32, UVec4, GlamUVec4, UVEC4,
    Float, f32, Vec2, GlamVec2, VEC2,
    Float, f32, Vec3, GlamVec3, VEC3, 
    Float, f32, Vec4, GlamVec4, VEC4,
    Double, f64, DVec2, GlamDVec2, DVEC2,
    Double, f64, DVec3, GlamDVec3, DVEC3, 
    Double, f64, DVec4, GlamDVec4, DVEC4,
);

macro_rules! impl_scalar_vec_assign_op {
    ($scalar:ident, $rust_scalar:ident, $vec:ident, $op:ident, $f:ident, $store:ident) => {
        impl<'a, 'b> std::ops::$op<$scalar<'b>> for $vec<'a> {
            fn $f(&mut self, rhs: $scalar<'b>) {
                let mut b = self.b.borrow_mut();
                $f(&mut b, self, &rhs, crate::Type::$store);
            }
        }

        impl<'a> std::ops::$op<$rust_scalar> for $vec<'a> {
            fn $f(&mut self, rhs: $rust_scalar) {
                let mut b = self.b.borrow_mut();
                $f(&mut b, self, &rhs, crate::Type::$store);
            }
        }
    };
}

macro_rules! impl_scalar_vec_assign_ops {
    ($($scalar:ident, $rust_scalar:ident, $vec:ident, $store:ident,)*) => {
        $(
            impl_scalar_vec_assign_op!($scalar, $rust_scalar, $vec, MulAssign, mul_assign, $store);
            impl_scalar_vec_assign_op!($scalar, $rust_scalar, $vec, DivAssign, div_assign, $store);
        )*
    };
}

#[rustfmt::skip]
impl_scalar_vec_assign_ops!(
    Int, i32, IVec2, IVEC2,
    Int, i32, IVec3, IVEC3,
    Int, i32, IVec4, IVEC4,
    UInt, u32, UVec2, UVEC2,
    UInt, u32, UVec3, UVEC3,
    UInt, u32, UVec4, UVEC4,
    Float, f32, Vec2, VEC2,
    Float, f32, Vec3, VEC3, 
    Float, f32, Vec4, VEC4,
    Double, f64, DVec2, DVEC2,
    Double, f64, DVec3, DVEC3, 
    Double, f64, DVec4, DVEC4,
);

macro_rules! impl_scalar_mat_op {
    ($scalar:ident, $rust_scalar:ident, $mat:ident, $rust_mat:ident, $op:ident, $f:ident, $store:ident) => {
        impl<'a, 'b> std::ops::$op<$scalar<'b>> for $mat<'a> {
            type Output = $mat<'a>;

            fn $f(self, rhs: $scalar<'b>) -> Self::Output {
                let mut b = self.b.borrow_mut();
                let id = $f(&mut b, &self, &rhs, crate::Type::$store);
                $mat {
                    id,
                    b: self.b
                }
            }
        }

        impl<'a, 'b> std::ops::$op<$mat<'b>> for $scalar<'a> {
            type Output = $mat<'a>;

            fn $f(self, rhs: $mat<'b>) -> Self::Output {
                let mut b = self.b.borrow_mut();
                let id = $f(&mut b, &self, &rhs, crate::Type::$store);
                $mat {
                    id,
                    b: self.b
                }
            }
        }

        impl<'a> std::ops::$op<$rust_scalar> for $mat<'a> {
            type Output = $mat<'a>;

            fn $f(self, rhs: $rust_scalar) -> Self::Output {
                let mut b = self.b.borrow_mut();
                let id = $f(&mut b, &self, &rhs, crate::Type::$store);
                $mat {
                    id,
                    b: self.b
                }
            }
        }

        impl<'a> std::ops::$op<$mat<'a>> for $rust_scalar {
            type Output = $mat<'a>;

            fn $f(self, rhs: $mat<'a>) -> Self::Output {
                let mut b = rhs.b.borrow_mut();
                let id = $f(&mut b, &self, &rhs, crate::Type::$store);
                $mat {
                    id,
                    b: rhs.b
                }
            }
        }

        impl<'a> std::ops::$op<$scalar<'a>> for $rust_mat {
            type Output = $mat<'a>;

            fn $f(self, rhs: $scalar<'a>) -> Self::Output {
                let mut b = rhs.b.borrow_mut();
                let id = $f(&mut b, &self, &rhs, crate::Type::$store);
                $mat {
                    id,
                    b: rhs.b
                }
            }
        }

        impl<'a> std::ops::$op<$rust_mat> for $scalar<'a> {
            type Output = $mat<'a>;

            fn $f(self, rhs: $rust_mat) -> Self::Output {
                let mut b = self.b.borrow_mut();
                let id = $f(&mut b, &self, &rhs, crate::Type::$store);
                $mat {
                    id,
                    b: self.b
                }
            }
        }
    };
}

macro_rules! impl_vec_mat_op {
    ($vec:ident, $rust_vec:ident, $mat:ident, $rust_mat:ident, $op:ident, $f:ident, $store:ident) => {
        impl<'a, 'b> std::ops::$op<$vec<'b>> for $mat<'a> {
            type Output = $vec<'a>;

            fn $f(self, rhs: $vec<'b>) -> Self::Output {
                let mut b = self.b.borrow_mut();
                let id = mul(&mut b, &self, &rhs, crate::Type::$store);
                $vec {
                    id,
                    b: self.b
                }
            }
        }

        impl<'a> std::ops::$op<$rust_vec> for $mat<'a> {
            type Output = $vec<'a>;

            fn $f(self, rhs: $rust_vec) -> Self::Output {
                let mut b = self.b.borrow_mut();
                let id = mul(&mut b, &self, &rhs, crate::Type::$store);
                $vec {
                    id,
                    b: self.b
                }
            }
        }

        impl<'a> std::ops::$op<$vec<'a>> for $rust_mat {
            type Output = $vec<'a>;

            fn $f(self, rhs: $vec<'a>) -> Self::Output {
                let mut b = rhs.b.borrow_mut();
                let id = mul(&mut b, &self, &rhs, crate::Type::$store);
                $vec {
                    id,
                    b: rhs.b
                }
            }
        }
    };
}

macro_rules! impl_scalar_vec_mat_ops {
    ($($scalar:ident, $rust_scalar:ident, $vec:ident, $rust_vec:ident, $mat:ident, $rust_mat:ident, $mat_store:ident, $vec_store:ident,)*) => {
        $(
            impl_scalar_mat_op!($scalar, $rust_scalar, $mat, $rust_mat, Mul, mul, $mat_store);
            impl_scalar_mat_op!($scalar, $rust_scalar, $mat, $rust_mat, Div, div, $mat_store);
            impl_vec_mat_op!($vec, $rust_vec, $mat, $rust_mat, Mul, mul, $vec_store);
        )*
    };
}

#[rustfmt::skip]
impl_scalar_vec_mat_ops!(
    Float, f32, Vec2, GlamVec2, Mat2, GlamMat2, MAT2, VEC2,
    Float, f32, Vec3, GlamVec3, Mat3, GlamMat3, MAT3, VEC3, 
    Float, f32, Vec4, GlamVec4, Mat4, GlamMat4, MAT4, VEC4,
    Double, f64, DVec2, GlamDVec2, DMat2, GlamDMat2, DMAT2, DVEC2,
    Double, f64, DVec3, GlamDVec3, DMat3, GlamDMat3, DMAT3, DVEC3, 
    Double, f64, DVec4, GlamDVec4, DMat4, GlamDMat4, DMAT4, DVEC4,
);

// comparisons
// ================================================================================
// ================================================================================
// ================================================================================

macro_rules! impl_cmp {
    ($($name:ident,)*) => {
        $(
            impl<'a> $name<'a> {
                fn cmp(&self, other: impl SpvRustEq<$name<'a>>, cmp_ty: crate::CmpType) -> Bool<'a> {
                    let mut inner = self.b.borrow_mut();
                    if let Some(scope) = &mut inner.scope {
                        let new_id = scope.get_new_id();

                        let other_id = other.id(&mut **scope);
                        let other_ty = other.ty();

                        scope.push_instruction(crate::Instruction::Cmp(crate::OpCmp {
                            cmp: cmp_ty,
                            lhs: (self.id, <Self as AsTypeConst>::TY),
                            rhs: (other_id, other_ty),
                            store: new_id,
                        }));

                        Bool {
                            id: new_id,
                            b: self.b
                        }

                    } else {
                        panic!("Cannot compare values when not in function")
                    }
                }

                pub fn eq(&self, other: impl SpvRustEq<$name<'a>>) -> Bool<'a> {
                    self.cmp(other, crate::CmpType::Eq)
                }

                pub fn neq(&self, other: impl SpvRustEq<$name<'a>>) -> Bool<'a> {
                    self.cmp(other, crate::CmpType::NEq)
                }

                pub fn lt(&self, other: impl SpvRustEq<$name<'a>>) -> Bool<'a> {
                    self.cmp(other, crate::CmpType::Lt)
                }

                pub fn gt(&self, other: impl SpvRustEq<$name<'a>>) -> Bool<'a> {
                    self.cmp(other, crate::CmpType::Gt)
                }

                pub fn le(&self, other: impl SpvRustEq<$name<'a>>) -> Bool<'a> {
                    self.cmp(other, crate::CmpType::Le)
                }

                pub fn ge(&self, other: impl SpvRustEq<$name<'a>>) -> Bool<'a> {
                    self.cmp(other, crate::CmpType::Ge)
                }
            }
        )*
    };
}

impl_cmp!(
    Int, UInt, Float, Double,
);

// math functions
// ================================================================================
// ================================================================================
// ================================================================================

macro_rules! impl_math_func_lhs {
    ($($name:ident, $ret:ident, $f:ident, $op:ident,)*) => {
        $(
            impl<'a> $name<'a> {
                pub fn $f(&self) -> $ret<'a> {
                    let mut inner = self.b.borrow_mut();
                    if let Some(scope) = &mut inner.scope {
                        let new_id = scope.get_new_id();
    
                        scope.push_instruction(crate::Instruction::Lhs(crate::OpLhs {
                            ty: crate::OpLhsType::$op,
                            lhs: (self.id, <$name as AsTypeConst>::TY),
                            store: (new_id, <$ret as AsTypeConst>::TY),
                        }));
    
                        drop(scope);
                        drop(inner);
                        $ret {
                            id: new_id,
                            b: self.b,
                        }
                    } else {
                        panic!("Cannot call op on data when not in function")
                    }
                }
            }
        )*
    };
}

macro_rules! impl_math_func_lhs_assign {
    ($($name:ident, $f:ident, $op:ident,)*) => {
        $(
            impl<'a> $name<'a> {
                pub fn $f(&self) {
                    let mut inner = self.b.borrow_mut();
                    if let Some(scope) = &mut inner.scope {    
                        scope.push_instruction(crate::Instruction::Lhs(crate::OpLhs {
                            ty: crate::OpLhsType::$op,
                            lhs: (self.id, <$name as AsTypeConst>::TY),
                            store: (self.id, <$name as AsTypeConst>::TY),
                        }));
    
                        drop(scope);
                        drop(inner);
                    } else {
                        panic!("Cannot call op on data when not in function")
                    }
                }
            }
        )*
    };
}

#[rustfmt::skip]
impl_math_func_lhs!(
    Bool, Bool, not, LogicalNot,

    Vec2, Float, length, Length,
    Vec3, Float, length, Length,
    Vec4, Float, length, Length,
    DVec2, Double, length, Length,
    DVec3, Double, length, Length,
    DVec4, Double, length, Length,

    Vec2, Vec2, normalized, Normalize,
    Vec3, Vec3, normalized, Normalize,
    Vec4, Vec4, normalized, Normalize,
    DVec2, DVec2, normalized, Normalize,
    DVec3, DVec3, normalized, Normalize,
    DVec4, DVec4, normalized, Normalize,

    Float, Float, exp, Exp,
    Double, Double, exp, Exp,
    Vec2, Vec2, exp, Exp,
    Vec3, Vec3, exp, Exp,
    Vec4, Vec4, exp, Exp,
    DVec2, DVec2, exp, Exp,
    DVec3, DVec3, exp, Exp,
    DVec4, DVec4, exp, Exp,

    Float, Float, exp2, Exp2,
    Double, Double, exp2, Exp2,
    Vec2, Vec2, exp2, Exp2,
    Vec3, Vec3, exp2, Exp2,
    Vec4, Vec4, exp2, Exp2,
    DVec2, DVec2, exp2, Exp2,
    DVec3, DVec3, exp2, Exp2,
    DVec4, DVec4, exp2, Exp2,

    Float, Float, sin, Sin,
    Double, Double, sin, Sin,
    Vec2, Vec2, sin, Sin,
    Vec3, Vec3, sin, Sin,
    Vec4, Vec4, sin, Sin,
    DVec2, DVec2, sin, Sin,
    DVec3, DVec3, sin, Sin,
    DVec4, DVec4, sin, Sin,

    Float, Float, cos, Cos,
    Double, Double, cos, Cos,
    Vec2, Vec2, cos, Cos,
    Vec3, Vec3, cos, Cos,
    Vec4, Vec4, cos, Cos,
    DVec2, DVec2, cos, Cos,
    DVec3, DVec3, cos, Cos,
    DVec4, DVec4, cos, Cos,

    Float, Float, tan, Tan,
    Double, Double, tan, Tan,
    Vec2, Vec2, tan, Tan,
    Vec3, Vec3, tan, Tan,
    Vec4, Vec4, tan, Tan,
    DVec2, DVec2, tan, Tan,
    DVec3, DVec3, tan, Tan,
    DVec4, DVec4, tan, Tan,

    Float, Float, asin, ASin,
    Double, Double, asin, ASin,
    Vec2, Vec2, asin, ASin,
    Vec3, Vec3, asin, ASin,
    Vec4, Vec4, asin, ASin,
    DVec2, DVec2, asin, ASin,
    DVec3, DVec3, asin, ASin,
    DVec4, DVec4, asin, ASin,

    Float, Float, acos, ACos,
    Double, Double, acos, ACos,
    Vec2, Vec2, acos, ACos,
    Vec3, Vec3, acos, ACos,
    Vec4, Vec4, acos, ACos,
    DVec2, DVec2, acos, ACos,
    DVec3, DVec3, acos, ACos,
    DVec4, DVec4, acos, ACos,

    Float, Float, atan, ATan,
    Double, Double, atan, ATan,
    Vec2, Vec2, atan, ATan,
    Vec3, Vec3, atan, ATan,
    Vec4, Vec4, atan, ATan,
    DVec2, DVec2, atan, ATan,
    DVec3, DVec3, atan, ATan,
    DVec4, DVec4, atan, ATan,
);

#[rustfmt::skip]
impl_math_func_lhs_assign!(
    Vec2, normalize, Normalize,
    Vec3, normalize, Normalize,
    Vec4, normalize, Normalize,
    DVec2, normalize, Normalize,
    DVec3, normalize, Normalize,
    DVec4, normalize, Normalize,
);

macro_rules! impl_math_func_lhs_rhs {
    ($($name:ident, $rhs:ident, $ret:ident, $f:ident, $op:ident,)*) => {
        $(
            impl<'a> $name<'a> {
                pub fn $f(&self, rhs: impl SpvRustEq<$rhs<'a>>) -> $ret<'a> {
                    let mut inner = self.b.borrow_mut();
                    if let Some(scope) = &mut inner.scope {
                        let new_id = scope.get_new_id();
                        let rhs_id = rhs.id(&mut **scope);

                        scope.push_instruction(crate::Instruction::LhsRhs(crate::OpLhsRhs {
                            ty: crate::OpLhsRhsType::$op,
                            lhs: (self.id, <$name as AsTypeConst>::TY),
                            rhs: (rhs_id, rhs.ty()),
                            store: (new_id, <$ret as AsTypeConst>::TY),
                        }));
    
                        drop(scope);
                        drop(inner);
                        $ret {
                            id: new_id,
                            b: self.b,
                        }
                    } else {
                        panic!("Cannot call op on data when not in function")
                    }
                }
            }
        )*
    };
}

#[rustfmt::skip]
impl_math_func_lhs_rhs!(
    Vec2, Vec2, Float, dot, Dot,
    Vec3, Vec3, Float, dot, Dot,
    Vec4, Vec4, Float, dot, Dot,
    DVec2, DVec2, Double, dot, Dot,
    DVec3, DVec3, Double, dot, Dot,
    DVec4, DVec4, Double, dot, Dot,

    Vec3, Vec3, Vec3, cross, Cross,
    DVec3, DVec3, DVec3, cross, Cross,
);

// vec swizzels
// ================================================================================
// ================================================================================
// ================================================================================

macro_rules! unit {
    ($elem:ident) => {
        fn unit(&self, idx: u32) -> $elem<'a> {
            let mut inner = self.b.borrow_mut();
            if let Some(scope) = &mut inner.scope {
                let new_id = scope.get_new_id();

                scope.push_instruction(crate::Instruction::Extract(crate::OpExtract {
                    src_id: self.id,
                    src_ty: <Self as crate::AsTypeConst>::TY,
                    element_ty: <$elem as crate::AsTypeConst>::TY,
                    element_idx: idx,
                    store_id: new_id,
                }));

                $elem {
                    id: new_id,
                    b: self.b
                }
            } else {
                panic!("Cannot swizzle vector when not in function")
            }
        }
    };
}

macro_rules! vec2 {
    ($vec2:ident) => {
        fn vec2(&self, x: u32, y: u32) -> $vec2<'a> {
            let mut inner = self.b.borrow_mut();
            if let Some(scope) = &mut inner.scope {
                let new_id = scope.get_new_id();

                scope.push_instruction(crate::Instruction::VectorShuffle(crate::OpVectorShuffle {
                    src: (self.id, <Self as crate::AsVectorTypeConst>::VECTOR_TY),
                    dst: (new_id, <$vec2 as crate::AsVectorTypeConst>::VECTOR_TY),
                    components: [x, y, 0, 0],
                }));

                $vec2 {
                    id: new_id,
                    b: self.b
                }
            } else {
                panic!("Cannot swizzle vector when not in function")
            }
        }
    };
}

macro_rules! vec3 {
    ($vec2:ident) => {
        fn vec3(&self, x: u32, y: u32, z: u32) -> $vec2<'a> {
            let mut inner = self.b.borrow_mut();
            if let Some(scope) = &mut inner.scope {
                let new_id = scope.get_new_id();

                scope.push_instruction(crate::Instruction::VectorShuffle(crate::OpVectorShuffle {
                    src: (self.id, <Self as crate::AsVectorTypeConst>::VECTOR_TY),
                    dst: (new_id, <$vec2 as crate::AsVectorTypeConst>::VECTOR_TY),
                    components: [x, y, z, 0],
                }));

                $vec2 {
                    id: new_id,
                    b: self.b
                }
            } else {
                panic!("Cannot swizzle vector when not in function")
            }
        }
    };
}

macro_rules! vec4 {
    ($vec2:ident) => {
        fn vec4(&self, x: u32, y: u32, z: u32, w: u32) -> $vec2<'a> {
            let mut inner = self.b.borrow_mut();
            if let Some(scope) = &mut inner.scope {
                let new_id = scope.get_new_id();

                scope.push_instruction(crate::Instruction::VectorShuffle(crate::OpVectorShuffle {
                    src: (self.id, <Self as crate::AsVectorTypeConst>::VECTOR_TY),
                    dst: (new_id, <$vec2 as crate::AsVectorTypeConst>::VECTOR_TY),
                    components: [x, y, z, w],
                }));

                $vec2 {
                    id: new_id,
                    b: self.b
                }
            } else {
                panic!("Cannot swizzle vector when not in function")
            }
        }
    };
}

// yes impl_*_swizzles can all be combined into one macro no I can't be bothered
macro_rules! impl_unit_swizzles {
    ($elem:ident, $($f:ident, $i:literal,)*) => {
        $(
            pub fn $f(&self) -> $elem<'a> {
                self.unit($i)
            }
        )*
    };
}

macro_rules! impl_vec2_swizzles {
    ($vec:ident, $($f:ident, $x:literal, $y:literal,)*) => {
        $(
            pub fn $f(&self) -> $vec<'a> {
                self.vec2($x, $y)
            }
        )*
    };
}

macro_rules! impl_vec3_swizzles {
    ($vec:ident, $($f:ident, $x:literal, $y:literal, $z:literal,)*) => {
        $(
            pub fn $f(&self) -> $vec<'a> {
                self.vec3($x, $y, $z)
            }
        )*
    };
}

macro_rules! impl_vec4_swizzles {
    ($vec:ident, $($f:ident, $x:literal, $y:literal, $z:literal, $w:literal,)*) => {
        $(
            pub fn $f(&self) -> $vec<'a> {
                self.vec4($x, $y, $z, $w)
            }
        )*
    };
}

// swizzle combinations generated in swizzle.py and pasted into rust code
// there are definitly better ways of doing this but for me this was fastest
macro_rules! impl_swizzles {
    ($($elem:ident, $vec2:ident, $vec3:ident, $vec4:ident,)*) => {
        $(
            impl<'a> $vec2<'a> {
                unit!($elem);
                vec2!($vec2);
                vec3!($vec3);
                vec4!($vec4);

                #[rustfmt::skip]
                impl_unit_swizzles!(
                    $elem, 
                    x, 0, 
                    y, 1,
                );

                #[rustfmt::skip]
                impl_vec2_swizzles!(
                    $vec2,
                    xx, 0, 0,
                    xy, 0, 1,
                    yy, 1, 1,
                );

                #[rustfmt::skip]
                impl_vec3_swizzles!(
                    $vec3,
                    xxx, 0, 0, 0,
                    xxy, 0, 0, 1,
                    xyx, 0, 1, 0,
                    xyy, 0, 1, 1,
                    yxx, 1, 0, 0,
                    yxy, 1, 0, 1,
                    yyx, 1, 1, 0,
                    yyy, 1, 1, 1,
                );

                #[rustfmt::skip]
                impl_vec4_swizzles!(
                    $vec4,
                    xxxx, 0, 0, 0, 0,
                    xxxy, 0, 0, 0, 1,
                    xxyx, 0, 0, 1, 0,
                    xxyy, 0, 0, 1, 1,
                    xyxx, 0, 1, 0, 0,
                    xyxy, 0, 1, 0, 1,
                    xyyx, 0, 1, 1, 0,
                    xyyy, 0, 1, 1, 1,
                    yxxx, 1, 0, 0, 0,
                    yxxy, 1, 0, 0, 1,
                    yxyx, 1, 0, 1, 0,
                    yxyy, 1, 0, 1, 1,
                    yyxx, 1, 1, 0, 0,
                    yyxy, 1, 1, 0, 1,
                    yyyx, 1, 1, 1, 0,
                    yyyy, 1, 1, 1, 1,
                );
            }

            impl<'a> $vec3<'a> {
                unit!($elem);
                vec2!($vec2);
                vec3!($vec3);
                vec4!($vec4);

                #[rustfmt::skip]
                impl_unit_swizzles!(
                    $elem,
                    x, 0,
                    y, 1,
                    z, 2,
                );

                #[rustfmt::skip]
                impl_vec2_swizzles!(
                    $vec2,
                    xx, 0, 0,
                    xy, 0, 1,
                    xz, 0, 2,
                    yx, 1, 0,
                    yy, 1, 1,
                    yz, 1, 2,
                    zx, 2, 0,
                    zy, 2, 1,
                    zz, 2, 2,
                );

                #[rustfmt::skip]
                impl_vec3_swizzles!(
                    $vec3,
                    xxx, 0, 0, 0,
                    xxy, 0, 0, 1,
                    xxz, 0, 0, 2,
                    xyx, 0, 1, 0,
                    xyy, 0, 1, 1,
                    xyz, 0, 1, 2,
                    xzx, 0, 2, 0,
                    xzy, 0, 2, 1,
                    xzz, 0, 2, 2,
                    yxx, 1, 0, 0,
                    yxy, 1, 0, 1,
                    yxz, 1, 0, 2,
                    yyx, 1, 1, 0,
                    yyy, 1, 1, 1,
                    yyz, 1, 1, 2,
                    yzx, 1, 2, 0,
                    yzy, 1, 2, 1,
                    yzz, 1, 2, 2,
                    zxx, 2, 0, 0,
                    zxy, 2, 0, 1,
                    zxz, 2, 0, 2,
                    zyx, 2, 1, 0,
                    zyy, 2, 1, 1,
                    zyz, 2, 1, 2,
                    zzx, 2, 2, 0,
                    zzy, 2, 2, 1,
                    zzz, 2, 2, 2,
                );

                #[rustfmt::skip]
                impl_vec4_swizzles!(
                    $vec4,
                    xxxx, 0, 0, 0, 0,
                    xxxy, 0, 0, 0, 1,
                    xxxz, 0, 0, 0, 2,
                    xxyx, 0, 0, 1, 0,
                    xxyy, 0, 0, 1, 1,
                    xxyz, 0, 0, 1, 2,
                    xxzx, 0, 0, 2, 0,
                    xxzy, 0, 0, 2, 1,
                    xxzz, 0, 0, 2, 2,
                    xyxx, 0, 1, 0, 0,
                    xyxy, 0, 1, 0, 1,
                    xyxz, 0, 1, 0, 2,
                    xyyx, 0, 1, 1, 0,
                    xyyy, 0, 1, 1, 1,
                    xyyz, 0, 1, 1, 2,
                    xyzx, 0, 1, 2, 0,
                    xyzy, 0, 1, 2, 1,
                    xyzz, 0, 1, 2, 2,
                    xzxx, 0, 2, 0, 0,
                    xzxy, 0, 2, 0, 1,
                    xzxz, 0, 2, 0, 2,
                    xzyx, 0, 2, 1, 0,
                    xzyy, 0, 2, 1, 1,
                    xzyz, 0, 2, 1, 2,
                    xzzx, 0, 2, 2, 0,
                    xzzy, 0, 2, 2, 1,
                    xzzz, 0, 2, 2, 2,
                    yxxx, 1, 0, 0, 0,
                    yxxy, 1, 0, 0, 1,
                    yxxz, 1, 0, 0, 2,
                    yxyx, 1, 0, 1, 0,
                    yxyy, 1, 0, 1, 1,
                    yxyz, 1, 0, 1, 2,
                    yxzx, 1, 0, 2, 0,
                    yxzy, 1, 0, 2, 1,
                    yxzz, 1, 0, 2, 2,
                    yyxx, 1, 1, 0, 0,
                    yyxy, 1, 1, 0, 1,
                    yyxz, 1, 1, 0, 2,
                    yyyx, 1, 1, 1, 0,
                    yyyy, 1, 1, 1, 1,
                    yyyz, 1, 1, 1, 2,
                    yyzx, 1, 1, 2, 0,
                    yyzy, 1, 1, 2, 1,
                    yyzz, 1, 1, 2, 2,
                    yzxx, 1, 2, 0, 0,
                    yzxy, 1, 2, 0, 1,
                    yzxz, 1, 2, 0, 2,
                    yzyx, 1, 2, 1, 0,
                    yzyy, 1, 2, 1, 1,
                    yzyz, 1, 2, 1, 2,
                    yzzx, 1, 2, 2, 0,
                    yzzy, 1, 2, 2, 1,
                    yzzz, 1, 2, 2, 2,
                    zxxx, 2, 0, 0, 0,
                    zxxy, 2, 0, 0, 1,
                    zxxz, 2, 0, 0, 2,
                    zxyx, 2, 0, 1, 0,
                    zxyy, 2, 0, 1, 1,
                    zxyz, 2, 0, 1, 2,
                    zxzx, 2, 0, 2, 0,
                    zxzy, 2, 0, 2, 1,
                    zxzz, 2, 0, 2, 2,
                    zyxx, 2, 1, 0, 0,
                    zyxy, 2, 1, 0, 1,
                    zyxz, 2, 1, 0, 2,
                    zyyx, 2, 1, 1, 0,
                    zyyy, 2, 1, 1, 1,
                    zyyz, 2, 1, 1, 2,
                    zyzx, 2, 1, 2, 0,
                    zyzy, 2, 1, 2, 1,
                    zyzz, 2, 1, 2, 2,
                    zzxx, 2, 2, 0, 0,
                    zzxy, 2, 2, 0, 1,
                    zzxz, 2, 2, 0, 2,
                    zzyx, 2, 2, 1, 0,
                    zzyy, 2, 2, 1, 1,
                    zzyz, 2, 2, 1, 2,
                    zzzx, 2, 2, 2, 0,
                    zzzy, 2, 2, 2, 1,
                    zzzz, 2, 2, 2, 2,
                );
            }

            impl<'a> $vec4<'a> {
                unit!($elem);
                vec2!($vec2);
                vec3!($vec3);
                vec4!($vec4);

                #[rustfmt::skip]
                impl_unit_swizzles!(
                    $elem,
                    x, 0,
                    y, 1,
                    z, 2,
                    w, 3,
                );

                #[rustfmt::skip]
                impl_vec2_swizzles!(
                    $vec2,
                    xx, 0, 0,
                    xy, 0, 1,
                    xz, 0, 2,
                    xw, 0, 3,
                    yx, 1, 0,
                    yy, 1, 1,
                    yz, 1, 2,
                    yw, 1, 3,
                    zx, 2, 0,
                    zy, 2, 1,
                    zz, 2, 2,
                    zw, 2, 3,
                    wx, 3, 0,
                    wy, 3, 1,
                    wz, 3, 2,
                    ww, 3, 3,
                );

                #[rustfmt::skip]
                impl_vec3_swizzles!(
                    $vec3,
                    xxx, 0, 0, 0,
                    xxy, 0, 0, 1,
                    xxz, 0, 0, 2,
                    xxw, 0, 0, 3,
                    xyx, 0, 1, 0,
                    xyy, 0, 1, 1,
                    xyz, 0, 1, 2,
                    xyw, 0, 1, 3,
                    xzx, 0, 2, 0,
                    xzy, 0, 2, 1,
                    xzz, 0, 2, 2,
                    xzw, 0, 2, 3,
                    xwx, 0, 3, 0,
                    xwy, 0, 3, 1,
                    xwz, 0, 3, 2,
                    xww, 0, 3, 3,
                    yxx, 1, 0, 0,
                    yxy, 1, 0, 1,
                    yxz, 1, 0, 2,
                    yxw, 1, 0, 3,
                    yyx, 1, 1, 0,
                    yyy, 1, 1, 1,
                    yyz, 1, 1, 2,
                    yyw, 1, 1, 3,
                    yzx, 1, 2, 0,
                    yzy, 1, 2, 1,
                    yzz, 1, 2, 2,
                    yzw, 1, 2, 3,
                    ywx, 1, 3, 0,
                    ywy, 1, 3, 1,
                    ywz, 1, 3, 2,
                    yww, 1, 3, 3,
                    zxx, 2, 0, 0,
                    zxy, 2, 0, 1,
                    zxz, 2, 0, 2,
                    zxw, 2, 0, 3,
                    zyx, 2, 1, 0,
                    zyy, 2, 1, 1,
                    zyz, 2, 1, 2,
                    zyw, 2, 1, 3,
                    zzx, 2, 2, 0,
                    zzy, 2, 2, 1,
                    zzz, 2, 2, 2,
                    zzw, 2, 2, 3,
                    zwx, 2, 3, 0,
                    zwy, 2, 3, 1,
                    zwz, 2, 3, 2,
                    zww, 2, 3, 3,
                    wxx, 3, 0, 0,
                    wxy, 3, 0, 1,
                    wxz, 3, 0, 2,
                    wxw, 3, 0, 3,
                    wyx, 3, 1, 0,
                    wyy, 3, 1, 1,
                    wyz, 3, 1, 2,
                    wyw, 3, 1, 3,
                    wzx, 3, 2, 0,
                    wzy, 3, 2, 1,
                    wzz, 3, 2, 2,
                    wzw, 3, 2, 3,
                    wwx, 3, 3, 0,
                    wwy, 3, 3, 1,
                    wwz, 3, 3, 2,
                    www, 3, 3, 3,
                );

                #[rustfmt::skip]
                impl_vec4_swizzles!(
                    $vec4,
                    xxxx, 0, 0, 0, 0,
                    xxxy, 0, 0, 0, 1,
                    xxxz, 0, 0, 0, 2,
                    xxxw, 0, 0, 0, 3,
                    xxyx, 0, 0, 1, 0,
                    xxyy, 0, 0, 1, 1,
                    xxyz, 0, 0, 1, 2,
                    xxyw, 0, 0, 1, 3,
                    xxzx, 0, 0, 2, 0,
                    xxzy, 0, 0, 2, 1,
                    xxzz, 0, 0, 2, 2,
                    xxzw, 0, 0, 2, 3,
                    xxwx, 0, 0, 3, 0,
                    xxwy, 0, 0, 3, 1,
                    xxwz, 0, 0, 3, 2,
                    xxww, 0, 0, 3, 3,
                    xyxx, 0, 1, 0, 0,
                    xyxy, 0, 1, 0, 1,
                    xyxz, 0, 1, 0, 2,
                    xyxw, 0, 1, 0, 3,
                    xyyx, 0, 1, 1, 0,
                    xyyy, 0, 1, 1, 1,
                    xyyz, 0, 1, 1, 2,
                    xyyw, 0, 1, 1, 3,
                    xyzx, 0, 1, 2, 0,
                    xyzy, 0, 1, 2, 1,
                    xyzz, 0, 1, 2, 2,
                    xyzw, 0, 1, 2, 3,
                    xywx, 0, 1, 3, 0,
                    xywy, 0, 1, 3, 1,
                    xywz, 0, 1, 3, 2,
                    xyww, 0, 1, 3, 3,
                    xzxx, 0, 2, 0, 0,
                    xzxy, 0, 2, 0, 1,
                    xzxz, 0, 2, 0, 2,
                    xzxw, 0, 2, 0, 3,
                    xzyx, 0, 2, 1, 0,
                    xzyy, 0, 2, 1, 1,
                    xzyz, 0, 2, 1, 2,
                    xzyw, 0, 2, 1, 3,
                    xzzx, 0, 2, 2, 0,
                    xzzy, 0, 2, 2, 1,
                    xzzz, 0, 2, 2, 2,
                    xzzw, 0, 2, 2, 3,
                    xzwx, 0, 2, 3, 0,
                    xzwy, 0, 2, 3, 1,
                    xzwz, 0, 2, 3, 2,
                    xzww, 0, 2, 3, 3,
                    xwxx, 0, 3, 0, 0,
                    xwxy, 0, 3, 0, 1,
                    xwxz, 0, 3, 0, 2,
                    xwxw, 0, 3, 0, 3,
                    xwyx, 0, 3, 1, 0,
                    xwyy, 0, 3, 1, 1,
                    xwyz, 0, 3, 1, 2,
                    xwyw, 0, 3, 1, 3,
                    xwzx, 0, 3, 2, 0,
                    xwzy, 0, 3, 2, 1,
                    xwzz, 0, 3, 2, 2,
                    xwzw, 0, 3, 2, 3,
                    xwwx, 0, 3, 3, 0,
                    xwwy, 0, 3, 3, 1,
                    xwwz, 0, 3, 3, 2,
                    xwww, 0, 3, 3, 3,
                    yxxx, 1, 0, 0, 0,
                    yxxy, 1, 0, 0, 1,
                    yxxz, 1, 0, 0, 2,
                    yxxw, 1, 0, 0, 3,
                    yxyx, 1, 0, 1, 0,
                    yxyy, 1, 0, 1, 1,
                    yxyz, 1, 0, 1, 2,
                    yxyw, 1, 0, 1, 3,
                    yxzx, 1, 0, 2, 0,
                    yxzy, 1, 0, 2, 1,
                    yxzz, 1, 0, 2, 2,
                    yxzw, 1, 0, 2, 3,
                    yxwx, 1, 0, 3, 0,
                    yxwy, 1, 0, 3, 1,
                    yxwz, 1, 0, 3, 2,
                    yxww, 1, 0, 3, 3,
                    yyxx, 1, 1, 0, 0,
                    yyxy, 1, 1, 0, 1,
                    yyxz, 1, 1, 0, 2,
                    yyxw, 1, 1, 0, 3,
                    yyyx, 1, 1, 1, 0,
                    yyyy, 1, 1, 1, 1,
                    yyyz, 1, 1, 1, 2,
                    yyyw, 1, 1, 1, 3,
                    yyzx, 1, 1, 2, 0,
                    yyzy, 1, 1, 2, 1,
                    yyzz, 1, 1, 2, 2,
                    yyzw, 1, 1, 2, 3,
                    yywx, 1, 1, 3, 0,
                    yywy, 1, 1, 3, 1,
                    yywz, 1, 1, 3, 2,
                    yyww, 1, 1, 3, 3,
                    yzxx, 1, 2, 0, 0,
                    yzxy, 1, 2, 0, 1,
                    yzxz, 1, 2, 0, 2,
                    yzxw, 1, 2, 0, 3,
                    yzyx, 1, 2, 1, 0,
                    yzyy, 1, 2, 1, 1,
                    yzyz, 1, 2, 1, 2,
                    yzyw, 1, 2, 1, 3,
                    yzzx, 1, 2, 2, 0,
                    yzzy, 1, 2, 2, 1,
                    yzzz, 1, 2, 2, 2,
                    yzzw, 1, 2, 2, 3,
                    yzwx, 1, 2, 3, 0,
                    yzwy, 1, 2, 3, 1,
                    yzwz, 1, 2, 3, 2,
                    yzww, 1, 2, 3, 3,
                    ywxx, 1, 3, 0, 0,
                    ywxy, 1, 3, 0, 1,
                    ywxz, 1, 3, 0, 2,
                    ywxw, 1, 3, 0, 3,
                    ywyx, 1, 3, 1, 0,
                    ywyy, 1, 3, 1, 1,
                    ywyz, 1, 3, 1, 2,
                    ywyw, 1, 3, 1, 3,
                    ywzx, 1, 3, 2, 0,
                    ywzy, 1, 3, 2, 1,
                    ywzz, 1, 3, 2, 2,
                    ywzw, 1, 3, 2, 3,
                    ywwx, 1, 3, 3, 0,
                    ywwy, 1, 3, 3, 1,
                    ywwz, 1, 3, 3, 2,
                    ywww, 1, 3, 3, 3,
                    zxxx, 2, 0, 0, 0,
                    zxxy, 2, 0, 0, 1,
                    zxxz, 2, 0, 0, 2,
                    zxxw, 2, 0, 0, 3,
                    zxyx, 2, 0, 1, 0,
                    zxyy, 2, 0, 1, 1,
                    zxyz, 2, 0, 1, 2,
                    zxyw, 2, 0, 1, 3,
                    zxzx, 2, 0, 2, 0,
                    zxzy, 2, 0, 2, 1,
                    zxzz, 2, 0, 2, 2,
                    zxzw, 2, 0, 2, 3,
                    zxwx, 2, 0, 3, 0,
                    zxwy, 2, 0, 3, 1,
                    zxwz, 2, 0, 3, 2,
                    zxww, 2, 0, 3, 3,
                    zyxx, 2, 1, 0, 0,
                    zyxy, 2, 1, 0, 1,
                    zyxz, 2, 1, 0, 2,
                    zyxw, 2, 1, 0, 3,
                    zyyx, 2, 1, 1, 0,
                    zyyy, 2, 1, 1, 1,
                    zyyz, 2, 1, 1, 2,
                    zyyw, 2, 1, 1, 3,
                    zyzx, 2, 1, 2, 0,
                    zyzy, 2, 1, 2, 1,
                    zyzz, 2, 1, 2, 2,
                    zyzw, 2, 1, 2, 3,
                    zywx, 2, 1, 3, 0,
                    zywy, 2, 1, 3, 1,
                    zywz, 2, 1, 3, 2,
                    zyww, 2, 1, 3, 3,
                    zzxx, 2, 2, 0, 0,
                    zzxy, 2, 2, 0, 1,
                    zzxz, 2, 2, 0, 2,
                    zzxw, 2, 2, 0, 3,
                    zzyx, 2, 2, 1, 0,
                    zzyy, 2, 2, 1, 1,
                    zzyz, 2, 2, 1, 2,
                    zzyw, 2, 2, 1, 3,
                    zzzx, 2, 2, 2, 0,
                    zzzy, 2, 2, 2, 1,
                    zzzz, 2, 2, 2, 2,
                    zzzw, 2, 2, 2, 3,
                    zzwx, 2, 2, 3, 0,
                    zzwy, 2, 2, 3, 1,
                    zzwz, 2, 2, 3, 2,
                    zzww, 2, 2, 3, 3,
                    zwxx, 2, 3, 0, 0,
                    zwxy, 2, 3, 0, 1,
                    zwxz, 2, 3, 0, 2,
                    zwxw, 2, 3, 0, 3,
                    zwyx, 2, 3, 1, 0,
                    zwyy, 2, 3, 1, 1,
                    zwyz, 2, 3, 1, 2,
                    zwyw, 2, 3, 1, 3,
                    zwzx, 2, 3, 2, 0,
                    zwzy, 2, 3, 2, 1,
                    zwzz, 2, 3, 2, 2,
                    zwzw, 2, 3, 2, 3,
                    zwwx, 2, 3, 3, 0,
                    zwwy, 2, 3, 3, 1,
                    zwwz, 2, 3, 3, 2,
                    zwww, 2, 3, 3, 3,
                    wxxx, 3, 0, 0, 0,
                    wxxy, 3, 0, 0, 1,
                    wxxz, 3, 0, 0, 2,
                    wxxw, 3, 0, 0, 3,
                    wxyx, 3, 0, 1, 0,
                    wxyy, 3, 0, 1, 1,
                    wxyz, 3, 0, 1, 2,
                    wxyw, 3, 0, 1, 3,
                    wxzx, 3, 0, 2, 0,
                    wxzy, 3, 0, 2, 1,
                    wxzz, 3, 0, 2, 2,
                    wxzw, 3, 0, 2, 3,
                    wxwx, 3, 0, 3, 0,
                    wxwy, 3, 0, 3, 1,
                    wxwz, 3, 0, 3, 2,
                    wxww, 3, 0, 3, 3,
                    wyxx, 3, 1, 0, 0,
                    wyxy, 3, 1, 0, 1,
                    wyxz, 3, 1, 0, 2,
                    wyxw, 3, 1, 0, 3,
                    wyyx, 3, 1, 1, 0,
                    wyyy, 3, 1, 1, 1,
                    wyyz, 3, 1, 1, 2,
                    wyyw, 3, 1, 1, 3,
                    wyzx, 3, 1, 2, 0,
                    wyzy, 3, 1, 2, 1,
                    wyzz, 3, 1, 2, 2,
                    wyzw, 3, 1, 2, 3,
                    wywx, 3, 1, 3, 0,
                    wywy, 3, 1, 3, 1,
                    wywz, 3, 1, 3, 2,
                    wyww, 3, 1, 3, 3,
                    wzxx, 3, 2, 0, 0,
                    wzxy, 3, 2, 0, 1,
                    wzxz, 3, 2, 0, 2,
                    wzxw, 3, 2, 0, 3,
                    wzyx, 3, 2, 1, 0,
                    wzyy, 3, 2, 1, 1,
                    wzyz, 3, 2, 1, 2,
                    wzyw, 3, 2, 1, 3,
                    wzzx, 3, 2, 2, 0,
                    wzzy, 3, 2, 2, 1,
                    wzzz, 3, 2, 2, 2,
                    wzzw, 3, 2, 2, 3,
                    wzwx, 3, 2, 3, 0,
                    wzwy, 3, 2, 3, 1,
                    wzwz, 3, 2, 3, 2,
                    wzww, 3, 2, 3, 3,
                    wwxx, 3, 3, 0, 0,
                    wwxy, 3, 3, 0, 1,
                    wwxz, 3, 3, 0, 2,
                    wwxw, 3, 3, 0, 3,
                    wwyx, 3, 3, 1, 0,
                    wwyy, 3, 3, 1, 1,
                    wwyz, 3, 3, 1, 2,
                    wwyw, 3, 3, 1, 3,
                    wwzx, 3, 3, 2, 0,
                    wwzy, 3, 3, 2, 1,
                    wwzz, 3, 3, 2, 2,
                    wwzw, 3, 3, 2, 3,
                    wwwx, 3, 3, 3, 0,
                    wwwy, 3, 3, 3, 1,
                    wwwz, 3, 3, 3, 2,
                    wwww, 3, 3, 3, 3,
                );
            }
        )*
    };
}

#[rustfmt::skip]
impl_swizzles!(
    Int, IVec2, IVec3, IVec4,
    UInt, UVec2, UVec3, UVec4,
    Float, Vec2, Vec3, Vec4,
    Double, DVec2, DVec3, DVec4,    
);

// matrix extract columens
// ================================================================================
// ================================================================================
// ================================================================================

macro_rules! impl_mat_col {
    ($($mat:ident, $vec:ident,)*) => {
        $(
            impl<'a> $mat<'a> {
                pub fn col(&self, idx: u32) -> $vec<'a> {
                    let mut inner = self.b.borrow_mut();
                    if let Some(scope) = &mut inner.scope {
                        let new_id = scope.get_new_id();
            
                        scope.push_instruction(crate::Instruction::Extract(crate::OpExtract {
                            src_id: self.id,
                            src_ty: <Self as crate::AsTypeConst>::TY,
                            element_ty: <$vec as crate::AsTypeConst>::TY,
                            element_idx: idx,
                            store_id: new_id,
                        }));
            
                        $vec {
                            id: new_id,
                            b: self.b
                        }
                    } else {
                        panic!("Cannot extract col from matrix when not in function")
                    }
                }
            }
        )*
    };
}

#[rustfmt::skip]
impl_mat_col!(
    Mat2, Vec2,
    Mat3, Vec3, 
    Mat4, Vec4,
    DMat2, DVec2,
    DMat3, DVec3, 
    DMat4, DVec4,
);

// spv struct
// ================================================================================
// ================================================================================
// ================================================================================

pub trait AsStructTypeConst {
    const STRUCT_TY: crate::StructType;
}

pub trait IsStructTypeConst: AsStructTypeConst { }

pub trait AsStructType {
    fn struct_ty(&self) -> crate::StructType;

    fn struct_id(&self, s: &mut dyn crate::Scope) -> usize;

    fn as_struct_ty_ref<'a>(&'a self) -> &'a dyn AsStructType;
}

pub trait IsStructType: AsStructType { }

pub trait RustStructType: AsStructType {
    type Spv<'a>: FromId<'a>;

    fn fields<'a>(&'a self) -> Vec<&'a dyn crate::AsType>;
}

pub struct Struct<'a> {
    pub(crate) id: usize,
    pub(crate) ty: crate::StructType,
    pub(crate) b: &'a Rc<RefCell<crate::BuilderInner>>,
}

impl<'a> AsStructType for Struct<'a> {
    fn struct_ty(&self) -> crate::StructType {
        self.ty.clone()
    }

    fn struct_id(&self, _: &mut dyn crate::Scope) -> usize {
        self.id
    }

    fn as_struct_ty_ref<'b>(&'b self) -> &'b dyn AsStructType {
        self
    }
}

impl<'a> Struct<'a> {
    pub fn load_field_by_index<T: IsTypeConst>(&self, field: u32) -> T::T<'a> {
        let mut inner = self.b.borrow_mut();
        if let Some(scope) = &mut inner.scope {
            let new_id = scope.get_new_id();
            
            scope.push_instruction(crate::Instruction::LoadStore(crate::OpLoadStore {
                ty: T::TY,
                src: crate::OpLoadStoreData::Struct {
                    id: self.id,
                    struct_ty: self.struct_ty(),
                    field,
                },
                dst: crate::OpLoadStoreData::Variable { id: new_id },
            }));

            drop(scope);
            drop(inner);
            T::T::from_id(new_id, self.b)
        } else {
            panic!("Cannot load struct field when not in function")
        }
    }

    pub fn load_field<T: IsTypeConst>(&self, field: &str) -> T::T<'a> {
        let field = self
            .ty
            .members
            .iter()
            .enumerate()
            .filter_map(|(i, e)| e.name.as_ref().map(|n| (i, n)))
            .find(|(_, n)| {
                let n = match n {
                    Left(n) => *n,
                    Right(n) => &**n,
                };

                n == field
            })
            .expect(&format!("Error cannot find field by name {}", field)).0;
        self.load_field_by_index::<T>(field as u32)
    }
}

// spv array
// ================================================================================
// ================================================================================
// ================================================================================

pub trait AsArrayTypeConst {
    const ARRAY_TY: crate::ArrayType;
}

pub trait IsArrayTypeConst: AsArrayTypeConst {}

pub trait AsArrayType {
    fn array_ty(&self) -> crate::ArrayType;

    fn array_id(&self, s: &mut dyn crate::Scope) -> usize;

    fn as_array_ty_ref<'a>(&'a self) -> &'a dyn AsArrayType;
}

pub trait IsArrayType: AsArrayType { }
 
pub struct Array<'a, T: IsTypeConst, const N: usize> {
    pub(crate) id: usize,
    pub(crate) b: &'a Rc<RefCell<crate::BuilderInner>>,
    pub(crate) marker: PhantomData<T>,
}

impl<'a, T: IsTypeConst, const N: usize> Array<'a, T, N> {
    const ELEMENT_TY: &'static crate::Type = &T::TY;

    pub fn index(&self, index: impl SpvRustEq<Int<'a>>) -> T::T<'a> {
        let mut b = self.b.borrow_mut();
        if let Some(scope) = &mut b.scope {
            let new_id = scope.get_new_id();
            
            let index_id = index.id(&mut **scope);
            let index_ty = index.ty();

            scope.push_instruction(crate::Instruction::LoadStore(crate::OpLoadStore {
                ty: T::TY,
                src: crate::OpLoadStoreData::ArrayElement {
                    id: self.id,
                    array_ty: self.array_ty(),
                    index: (index_id, index_ty),
                },
                dst: crate::OpLoadStoreData::Variable { id: new_id },
            }));

            drop(scope);
            drop(b);
            T::T::from_id(new_id, self.b)
        } else {
            panic!("Cannot index array when not in function")
        }
    }
}

impl<'a, T: IsTypeConst, const N: usize> AsArrayTypeConst for Array<'a, T, N> {
    const ARRAY_TY: crate::ArrayType = crate::ArrayType {
        element_ty: Left(Self::ELEMENT_TY),
        length: Some(N),
    };
} 

impl<'a, T: IsTypeConst, const N: usize> AsArrayType for Array<'a, T, N> {
    fn array_ty(&self) -> crate::ArrayType {
        <Self as AsArrayTypeConst>::ARRAY_TY
    }

    fn array_id(&self, _: &mut dyn crate::Scope) -> usize {
        self.id
    }

    fn as_array_ty_ref<'b>(&'b self) -> &'b dyn AsArrayType {
        self
    }
}

impl<'a, T: IsTypeConst, const N: usize> IsArrayType for Array<'a, T, N> { }

struct Help<T: AsTypeConst> {
    marker: PhantomData<T>
}

impl<T: AsTypeConst> Help<T> {
    const ELEMENT_TY: &'static crate::Type = &T::TY;
}

impl<'a, T: AsTypeConst, const N: usize> AsArrayTypeConst for [T; N] {
    const ARRAY_TY: crate::ArrayType = crate::ArrayType {
        element_ty: Left(&Help::<T>::ELEMENT_TY),
        length: Some(N)
    };
}

impl<'a, T: AsTypeConst + AsType, const N: usize> AsArrayType for [T; N] {
    fn array_ty(&self) -> crate::ArrayType {
        <Self as AsArrayTypeConst>::ARRAY_TY
    }

    fn array_id(&self, s: &mut dyn crate::Scope) -> usize {
        let constituents = self.iter()
            .map(|c| {
                (c.id(s), c.ty())
            })
            .collect::<Vec<_>>();
        let new_id = s.get_new_id();
        s.push_instruction(crate::Instruction::Composite(crate::OpComposite {
            ty: crate::Type::Array(Self::ARRAY_TY),
            id: new_id,
            constituents,
        }));
        new_id
    }

    fn as_array_ty_ref<'b>(&'b self) -> &'b dyn AsArrayType {
        self
    }
}

impl<'a, T: AsTypeConst + AsType, const N: usize> AsTypeConst for [T; N] {
    const TY: crate::Type = crate::Type::Array(<Self as AsArrayTypeConst>::ARRAY_TY);
}

impl<'a, T: AsTypeConst + AsType, const N: usize> AsType for [T; N] {
    fn ty(&self) -> crate::Type {
        <Self as AsTypeConst>::TY
    }

    fn id(&self, s: &mut dyn crate::Scope) -> usize {
        self.array_id(s)
    }

    fn as_ty_ref<'b>(&'b self) -> &'b dyn AsType {
        self
    }
}

// dimension
// ================================================================================
// ================================================================================
// ================================================================================

#[derive(Copy, Clone, Debug)]
pub struct Sampler {
    pub(crate) id: usize,
}

pub trait AsDimension {
    const DIMENSION: crate::TextureDimension;

    type Coordinate<'a>: AsType + AsTypeConst;
}

macro_rules! impl_as_dimension {
    ($($name:ident, $coordinate:ident,)*) => {
        $(
            pub struct $name;

            impl AsDimension for $name {
                const DIMENSION: crate::TextureDimension = crate::TextureDimension::$name;

                type Coordinate<'a> = $coordinate<'a>;
            }
        )*
    };
}

#[rustfmt::skip]
impl_as_dimension!(
    D1, Float,
    D1Array, Vec2,
    D2, Vec2,
    D2Ms, Vec2,
    D2Array, Vec3,
    D2MsArray, Vec3,
    Cube, Vec3,
    CubeArray, Vec4,
    D3, Vec3,
);

// spv texture
// ================================================================================
// ================================================================================
// ================================================================================

pub trait GTexture<D: AsDimension> {
    const TEXTURE_TY: crate::TextureType;

    type Sampler: SampledGTexture<D>;

    fn new(id: usize, b: Rc<RefCell<crate::BuilderInner>>) -> Self;

    fn texture_id(&self) -> usize;

    fn b<'a>(&'a self) -> &'a Rc<RefCell<crate::BuilderInner>>;
}

macro_rules! impl_g_texture {
    ($($name:ident, $scalar_ty:ident, $sampler:ident,)*) => {
        $(
            pub struct $name<D: AsDimension> {
                pub(crate) id: usize,
                pub(crate) b: Rc<RefCell<crate::BuilderInner>>,
                pub(crate) marker: PhantomData<D>,
            }

            impl<D: AsDimension> GTexture<D> for $name<D> {
                const TEXTURE_TY: crate::TextureType = crate::TextureType {
                    scalar_ty: crate::ScalarType::$scalar_ty,
                    dimension: <D as AsDimension>::DIMENSION,
                    format: crate::TextureSpvFormat::Sampled,
                };

                type Sampler = $sampler<D>;

                fn new(id: usize, b: Rc<RefCell<crate::BuilderInner>>) -> Self {
                    Self {
                        id,
                        b,
                        marker: PhantomData,
                    }
                }

                fn texture_id(&self) -> usize {
                    self.id
                }

                fn b<'a>(&'a self) -> &'a Rc<RefCell<crate::BuilderInner>> {
                    &self.b
                }
            }
        )*
    };
}

#[rustfmt::skip]
impl_g_texture!(
    ITexture, INT, SampledITexture,
    UTexture, UINT, SampledUTexture,
    Texture, FLOAT, SampledTexture,
    DTexture, DOUBLE, SampledDTexture,
);

pub type ITexture1D         = ITexture<D1>;
pub type ITexture1DArray    = ITexture<D1Array>;
pub type ITexture2D         = ITexture<D2>;
pub type ITexture2DMs       = ITexture<D2Ms>;
pub type ITexture2DArray    = ITexture<D2Array>;
pub type ITexture2DMsArray  = ITexture<D2MsArray>;
pub type ITextureCube       = ITexture<Cube>;
pub type ITextureCubeArray  = ITexture<CubeArray>;

pub type UTexture1D         = UTexture<D1>;
pub type UTexture1DArray    = UTexture<D1Array>;
pub type UTexture2D         = UTexture<D2>;
pub type UTexture2DMs       = UTexture<D2Ms>;
pub type UTexture2DArray    = UTexture<D2Array>;
pub type UTexture2DMsArray  = UTexture<D2MsArray>;
pub type UTextureCube       = UTexture<Cube>;
pub type UTextureCubeArray  = UTexture<CubeArray>;

pub type Texture1D          = Texture<D1>;
pub type Texture1DArray     = Texture<D1Array>;
pub type Texture2D          = Texture<D2>;
pub type Texture2DMs        = Texture<D2Ms>;
pub type Texture2DArray     = Texture<D2Array>;
pub type Texture2DMsArray   = Texture<D2MsArray>;
pub type TextureCube        = Texture<Cube>;
pub type TextureCubeArray   = Texture<CubeArray>;

pub type DTexture1D         = DTexture<D1>;
pub type DTexture1DArray    = DTexture<D1Array>;
pub type DTexture2D         = DTexture<D2>;
pub type DTexture2DMs       = DTexture<D2Ms>;
pub type DTexture2DArray    = DTexture<D2Array>;
pub type DTexture2DMsArray  = DTexture<D2MsArray>;
pub type DTextureCube       = DTexture<Cube>;
pub type DTextureCubeArray  = DTexture<CubeArray>;

// spv sampled texture
// ================================================================================
// ================================================================================
// ================================================================================

pub trait SampledGTexture<D: AsDimension> {
    type Texture: GTexture<D>;

    type Sample<'a>: FromId<'a>;

    fn from_uniform(id: usize, b: Rc<RefCell<crate::BuilderInner>>) -> Self;

    fn from_combine(id: usize, b: Rc<RefCell<crate::BuilderInner>>) -> Self;

    /// Left(uniform) Right(combined)
    fn sampled_texture_id(&self) -> Either<usize, usize>;

    fn b<'a>(&'a self) -> &'a Rc<RefCell<crate::BuilderInner>>;
}

macro_rules! impl_g_sampler {
    ($($name:ident, $tex:ident, $sample:ident,)*) => {
        $(
            pub struct $name<D: AsDimension> {
                pub(crate) id: Either<usize, usize>,
                pub(crate) b: Rc<RefCell<crate::BuilderInner>>,
                pub(crate) marker: PhantomData<D>,
            }

            impl<D: AsDimension> SampledGTexture<D> for $name<D> {
                type Texture = $tex<D>;

                type Sample<'a> = $sample<'a>;

                fn from_uniform(id: usize, b: Rc<RefCell<crate::BuilderInner>>) -> Self {
                    Self {
                        id: Left(id),
                        b,
                        marker: PhantomData,
                    }
                }

                fn from_combine(id: usize, b: Rc<RefCell<crate::BuilderInner>>) -> Self {
                    Self {
                        id: Right(id),
                        b,
                        marker: PhantomData,
                    }
                }

                fn sampled_texture_id(&self) -> Either<usize, usize> {
                    self.id
                }

                fn b<'a>(&'a self) -> &'a Rc<RefCell<crate::BuilderInner>> {
                    &self.b
                }
            }
        )*
    };
}

#[rustfmt::skip]
impl_g_sampler!(
    SampledITexture, ITexture, IVec4,
    SampledUTexture, UTexture, UVec4,
    SampledTexture, Texture, Vec4,
    SampledDTexture, DTexture, DVec4,
);

pub type SampledITexture1D         = SampledITexture<D1>;
pub type SampledITexture1DArray    = SampledITexture<D1Array>;
pub type SampledITexture2D         = SampledITexture<D2>;
pub type SampledITexture2DMs       = SampledITexture<D2Ms>;
pub type SampledITexture2DArray    = SampledITexture<D2Array>;
pub type SampledITexture2DMsArray  = SampledITexture<D2MsArray>;
pub type SampledITextureCube       = SampledITexture<Cube>;
pub type SampledITextureCubeArray  = SampledITexture<CubeArray>;

pub type SampledUTexture1D         = SampledUTexture<D1>;
pub type SampledUTexture1DArray    = SampledUTexture<D1Array>;
pub type SampledUTexture2D         = SampledUTexture<D2>;
pub type SampledUTexture2DMs       = SampledUTexture<D2Ms>;
pub type SampledUTexture2DArray    = SampledUTexture<D2Array>;
pub type SampledUTexture2DMsArray  = SampledUTexture<D2MsArray>;
pub type SampledUTextureCube       = SampledUTexture<Cube>;
pub type SampledUTextureCubeArray  = SampledUTexture<CubeArray>;

pub type SampledTexture1D          = SampledTexture<D1>;
pub type SampledTexture1DArray     = SampledTexture<D1Array>;
pub type SampledTexture2D          = SampledTexture<D2>;
pub type SampledTexture2DMs        = SampledTexture<D2Ms>;
pub type SampledTexture2DArray     = SampledTexture<D2Array>;
pub type SampledTexture2DMsArray   = SampledTexture<D2MsArray>;
pub type SampledTextureCube        = SampledTexture<Cube>;
pub type SampledTextureCubeArray   = SampledTexture<CubeArray>;

pub type SampledDTexture1D         = SampledDTexture<D1>;
pub type SampledDTexture1DArray    = SampledDTexture<D1Array>;
pub type SampledDTexture2D         = SampledDTexture<D2>;
pub type SampledDTexture2DMs       = SampledDTexture<D2Ms>;
pub type SampledDTexture2DArray    = SampledDTexture<D2Array>;
pub type SampledDTexture2DMsArray  = SampledDTexture<D2MsArray>;
pub type SampledDTextureCube       = SampledDTexture<Cube>;
pub type SampledDTextureCubeArray  = SampledDTexture<CubeArray>;