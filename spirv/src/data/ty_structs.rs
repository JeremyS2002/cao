
use crate::builder::RawBuilder;

use glam::BVec2 as GlamBVec2;
use glam::BVec3 as GlamBVec3;
use glam::BVec4 as GlamBVec4;
use glam::IVec2 as GlamIVec2;
use glam::IVec3 as GlamIVec3;
use glam::IVec4 as GlamIVec4;
use glam::UVec2 as GlamUVec2;
use glam::UVec3 as GlamUVec3;
use glam::UVec4 as GlamUVec4;
use glam::Vec2 as GlamVec2;
use glam::Vec3 as GlamVec3;
use glam::Vec4 as GlamVec4;
use glam::DVec2 as GlamDVec2;
use glam::DVec3 as GlamDVec3;
use glam::DVec4 as GlamDVec4;
use glam::Mat2 as GlamMat2;
use glam::Mat3 as GlamMat3;
use glam::Mat4 as GlamMat4;
use glam::DMat2 as GlamDMat2;
use glam::DMat3 as GlamDMat3;
use glam::DMat4 as GlamDMat4;

macro_rules! gen_types {
    ($($name:ident,)*) => {
        $(
            #[derive(Copy, Clone)]
            pub struct $name {
                pub(crate) id: usize,
            }
        )*
    };
}

gen_types!(
    Bool,
    Int,
    UInt,
    Float,
    Double,
    BVec2,
    BVec3,
    BVec4,
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

pub trait AsDataType {
    const TY: crate::data::DataType;
}

pub trait AsData {
    fn id(&self, b: &dyn RawBuilder) -> usize;

    fn ty(&self) -> crate::data::DataType;
}

pub trait FromId {
    fn from_id(id: usize) -> Self;
}

pub trait SpvStore<Rhs: AsDataType>: AsDataType + AsData { 
    fn val(rhs: Rhs) -> crate::data::DataVal;
}

macro_rules! gen_as_data {
    ($($name:ident, $rust:ident,)*) => {
        $(
            impl SpvStore<$rust> for $name { 
                fn val(rhs: $rust) -> crate::data::DataVal {
                    crate::data::DataVal::$name(rhs)
                }
            }

            impl AsDataType for $name {
                const TY: crate::data::DataType = crate::data::DataType::$name;
            }

            impl AsData for $name {
                fn id(&self, _: &dyn RawBuilder) -> usize {
                    self.id
                }

                fn ty(&self) -> crate::data::DataType {
                    Self::TY
                }
            }

            impl AsDataType for $rust {
                const TY: crate::data::DataType = crate::data::DataType::$name;
            }

            impl AsData for $rust {
                fn id(&self, b: &dyn RawBuilder) -> usize {
                    let id = b.get_new_id(crate::data::DataType::$name);
                    b.push_instruction(crate::builder::Instruction::Store {
                        val: crate::data::DataVal::$name(*self),
                        store: (id, crate::data::DataType::$name),
                    });
                    id
                }

                fn ty(&self) -> crate::data::DataType {
                    Self::TY
                }
            }

            impl FromId for $name {
                fn from_id(id: usize) -> Self {
                    Self { id }
                }
            }
        )*
    };
}

gen_as_data!(
    Bool, bool,
    Int, i32,
    UInt, u32,
    Float, f32,
    Double, f64,
    BVec2, GlamBVec2,
    BVec3, GlamBVec3,
    BVec4, GlamBVec4,
    IVec2, GlamIVec2,
    IVec3, GlamIVec3,
    IVec4, GlamIVec4,
    UVec2, GlamUVec2,
    UVec3, GlamUVec3,
    UVec4, GlamUVec4,
    Vec2, GlamVec2,
    Vec3, GlamVec3,
    Vec4, GlamVec4,
    DVec2, GlamDVec2,
    DVec3, GlamDVec3,
    DVec4, GlamDVec4,
    Mat2, GlamMat2,
    Mat3, GlamMat3,
    Mat4, GlamMat4,
    DMat2, GlamDMat2,
    DMat3, GlamDMat3,
    DMat4, GlamDMat4,
);

// macro_rules! gen_op {
//     ($op:ident, $f:ident, $name:ident, $rust:ident) => {
//         // name name
//         impl std::ops::$op<$name> for $name {
//             type Output = $name;

//             fn $f(self, rhs: $name) -> Self::Output {
//                 let new_id = self.builder.get_new_id(crate::data::DataType::$name);
//                 self.builder.push_instruction(crate::builder::Instruction::$op {
//                     lhs: self.id,
//                     rhs: rhs.id,
//                     res: new_id,
//                 });
//                 $name {
//                     builder: Rc::clone(&self.builder),
//                     id: new_id,
//                 }
//             }
//         }

//         // name &name
//         impl std::ops::$op<$name> for &'_ $name {
//             type Output = $name;

//             fn $f(self, rhs: $name) -> Self::Output {
//                 let new_id = self.builder.get_new_id(crate::data::DataType::$name);
//                 self.builder.push_instruction(crate::builder::Instruction::$op {
//                     lhs: self.id,
//                     rhs: rhs.id,
//                     res: new_id,
//                 });
//                 $name {
//                     builder: Rc::clone(&self.builder),
//                     id: new_id,
//                 }
//             }
//         }

//         // &name name
//         impl std::ops::$op<&'_ $name> for $name {
//             type Output = $name;

//             fn $f(self, rhs: &'_ $name) -> Self::Output {
//                 let new_id = self.builder.get_new_id(crate::data::DataType::$name);
//                 self.builder.push_instruction(crate::builder::Instruction::$op {
//                     lhs: self.id,
//                     rhs: rhs.id,
//                     res: new_id,
//                 });
//                 $name {
//                     builder: Rc::clone(&self.builder),
//                     id: new_id,
//                 }
//             }
//         }

//         // &name &name
//         impl std::ops::$op<&'_ $name> for &'_ $name {
//             type Output = $name;

//             fn $f(self, rhs: &'_ $name) -> Self::Output {
//                 let new_id = self.builder.get_new_id(crate::data::DataType::$name);
//                 self.builder.push_instruction(crate::builder::Instruction::$op {
//                     lhs: self.id,
//                     rhs: rhs.id,
//                     res: new_id,
//                 });
//                 $name {
//                     builder: Rc::clone(&self.builder),
//                     id: new_id,
//                 }
//             }
//         }

//         // name rust
//         impl std::ops::$op<$rust> for $name {
//             type Output = $name;

//             fn $f(self, rhs: $rust) -> Self::Output {
//                 let rhs_id = self.builder.get_new_id(crate::data::DataType::$name);
//                 self.builder.push_instruction(crate::builder::Instruction::Store {
//                     val: crate::data::DataVal::$name(rhs),
//                     store: rhs_id,
//                 });
//                 let new_id = self.builder.get_new_id(crate::data::DataType::$name);
//                 self.builder.push_instruction(crate::builder::Instruction::$op {
//                     lhs: self.id,
//                     rhs: rhs_id,
//                     res: new_id,
//                 });
//                 $name {
//                     builder: Rc::clone(&self.builder),
//                     id: new_id,
//                 }
//             }
//         }
        
//         // name &rust
//         impl std::ops::$op<&'_ $rust> for $name {
//             type Output = $name;

//             fn $f(self, rhs: &'_ $rust) -> Self::Output {
//                 let rhs_id = self.builder.get_new_id(crate::data::DataType::$name);
//                 self.builder.push_instruction(crate::builder::Instruction::Store {
//                     val: crate::data::DataVal::$name(*rhs),
//                     store: rhs_id,
//                 });
//                 let new_id = self.builder.get_new_id(crate::data::DataType::$name);
//                 self.builder.push_instruction(crate::builder::Instruction::$op {
//                     lhs: self.id,
//                     rhs: rhs_id,
//                     res: new_id,
//                 });
//                 $name {
//                     builder: Rc::clone(&self.builder),
//                     id: new_id,
//                 }
//             }
//         }

//         // &name rust
//         impl std::ops::$op<$rust> for &'_ $name {
//             type Output = $name;

//             fn $f(self, rhs: $rust) -> Self::Output {
//                 let rhs_id = self.builder.get_new_id(crate::data::DataType::$name);
//                 self.builder.push_instruction(crate::builder::Instruction::Store {
//                     val: crate::data::DataVal::$name(rhs),
//                     store: rhs_id,
//                 });
//                 let new_id = self.builder.get_new_id(crate::data::DataType::$name);
//                 self.builder.push_instruction(crate::builder::Instruction::$op {
//                     lhs: self.id,
//                     rhs: rhs_id,
//                     res: new_id,
//                 });
//                 $name {
//                     builder: Rc::clone(&self.builder),
//                     id: new_id,
//                 }
//             }
//         }

//         // &name &rust
//         impl std::ops::$op<&'_ $rust> for &'_ $name {
//             type Output = $name;

//             fn $f(self, rhs: &'_ $rust) -> Self::Output {
//                 let rhs_id = self.builder.get_new_id(crate::data::DataType::$name);
//                 self.builder.push_instruction(crate::builder::Instruction::Store {
//                     val: crate::data::DataVal::$name(*rhs),
//                     store: rhs_id,
//                 });
//                 let new_id = self.builder.get_new_id(crate::data::DataType::$name);
//                 self.builder.push_instruction(crate::builder::Instruction::$op {
//                     lhs: self.id,
//                     rhs: rhs_id,
//                     res: new_id,
//                 });
//                 $name {
//                     builder: Rc::clone(&self.builder),
//                     id: new_id,
//                 }
//             }
//         }

//         // rust name
//         impl std::ops::$op<$name> for $rust {
//             type Output = $name;

//             fn $f(self, rhs: $name) -> Self::Output {
//                 let lhs_id = rhs.builder.get_new_id(crate::data::DataType::$name);
//                 rhs.builder.push_instruction(crate::builder::Instruction::Store {
//                     val: crate::data::DataVal::$name(self),
//                     store: lhs_id,
//                 });
//                 let new_id = rhs.builder.get_new_id(crate::data::DataType::$name);
//                 rhs.builder.push_instruction(crate::builder::Instruction::$op {
//                     lhs: lhs_id,
//                     rhs: rhs.id,
//                     res: new_id,
//                 });
//                 $name {
//                     builder: Rc::clone(&rhs.builder),
//                     id: new_id,
//                 }
//             }
//         }

//         // rust &name
//         impl std::ops::$op<&'_ $name> for $rust {
//             type Output = $name;

//             fn $f(self, rhs: &'_ $name) -> Self::Output {
//                 let lhs_id = rhs.builder.get_new_id(crate::data::DataType::$name);
//                 rhs.builder.push_instruction(crate::builder::Instruction::Store {
//                     val: crate::data::DataVal::$name(self),
//                     store: lhs_id,
//                 });
//                 let new_id = rhs.builder.get_new_id(crate::data::DataType::$name);
//                 rhs.builder.push_instruction(crate::builder::Instruction::$op {
//                     lhs: lhs_id,
//                     rhs: rhs.id,
//                     res: new_id,
//                 });
//                 $name {
//                     builder: Rc::clone(&rhs.builder),
//                     id: new_id,
//                 }
//             }
//         }

//         // &rust name
//         impl std::ops::$op<$name> for &'_ $rust {
//             type Output = $name;

//             fn $f(self, rhs: $name) -> Self::Output {
//                 let lhs_id = rhs.builder.get_new_id(crate::data::DataType::$name);
//                 rhs.builder.push_instruction(crate::builder::Instruction::Store {
//                     val: crate::data::DataVal::$name(*self),
//                     store: lhs_id,
//                 });
//                 let new_id = rhs.builder.get_new_id(crate::data::DataType::$name);
//                 rhs.builder.push_instruction(crate::builder::Instruction::$op {
//                     lhs: lhs_id,
//                     rhs: rhs.id,
//                     res: new_id,
//                 });
//                 $name {
//                     builder: Rc::clone(&rhs.builder),
//                     id: new_id,
//                 }
//             }
//         }

//         // &rust &name
//         impl std::ops::$op<&'_ $name> for &'_ $rust {
//             type Output = $name;

//             fn $f(self, rhs: &'_ $name) -> Self::Output {
//                 let lhs_id = rhs.builder.get_new_id(crate::data::DataType::$name);
//                 rhs.builder.push_instruction(crate::builder::Instruction::Store {
//                     val: crate::data::DataVal::$name(*self),
//                     store: lhs_id,
//                 });
//                 let new_id = rhs.builder.get_new_id(crate::data::DataType::$name);
//                 rhs.builder.push_instruction(crate::builder::Instruction::$op {
//                     lhs: lhs_id,
//                     rhs: rhs.id,
//                     res: new_id,
//                 });
//                 $name {
//                     builder: Rc::clone(&rhs.builder),
//                     id: new_id,
//                 }
//             }
//         }
//     };
// }

// macro_rules! gen_bool_ops {
//     ($($name:ident, $rust:ident,)*) => {
//         $(
//             gen_op!(BitAnd, bitand, $name, $rust);
//             gen_op!(BitOr, bitor, $name, $rust);
//             gen_op!(BitXor, bitxor, $name, $rust);
//         )*
//     };
// }

// gen_bool_ops!(
//     Bool, bool,
//     BVec2, GlamBVec2,
//     BVec3, GlamBVec3,
//     BVec4, GlamBVec4,
// );

// macro_rules! gen_ops {
//     ($($name:ident, $rust:ident,)*) => {
//         $(
//             gen_op!(Add, add, $name, $rust);
//             gen_op!(Sub, sub, $name, $rust);
//             gen_op!(Mul, mul, $name, $rust);
//             gen_op!(Div, div, $name, $rust);
//         )*
//     };
// }

// gen_ops!(
//     Int, i32,
//     UInt, u32,
//     Float, f32,
//     Double, f64,
//     IVec2, GlamIVec2,
//     IVec3, GlamIVec3,
//     IVec4, GlamIVec4,
//     UVec2, GlamUVec2,
//     UVec3, GlamUVec3,
//     UVec4, GlamUVec4,
//     Vec2, GlamVec2,
//     Vec3, GlamVec3,
//     Vec4, GlamVec4,
//     DVec2, GlamDVec2,
//     DVec3, GlamDVec3,
//     DVec4, GlamDVec4,
//     Mat2, GlamMat2,
//     Mat3, GlamMat3,
//     Mat4, GlamMat4,
//     DMat2, GlamDMat2,
//     DMat3, GlamDMat3,
//     DMat4, GlamDMat4,
// );


// macro_rules! gen_vec_mat_ops {
//     ($($mat:ident, $rust_mat:ident, $vec:ident, $rust_vec:ident,)*) => {
//         $(
//             // mat vec
//             impl std::ops::Mul<$vec> for $mat {
//                 type Output = $vec;

//                 fn mul(self, rhs: $vec) -> Self::Output {
//                     let new_id = self.builder.get_new_id(crate::data::DataType::$vec);
//                     self.builder.push_instruction(crate::builder::Instruction::Mul {
//                         lhs: self.id,
//                         rhs: rhs.id,
//                         res: new_id,
//                     });
//                     $vec {
//                         builder: Rc::clone(&self.builder),
//                         id: new_id,
//                     }
//                 }
//             }

//             // mat &vec
//             impl std::ops::Mul<&'_ $vec> for $mat {
//                 type Output = $vec;

//                 fn mul(self, rhs: &'_ $vec) -> Self::Output {
//                     let new_id = self.builder.get_new_id(crate::data::DataType::$vec);
//                     self.builder.push_instruction(crate::builder::Instruction::Mul {
//                         lhs: self.id,
//                         rhs: rhs.id,
//                         res: new_id,
//                     });
//                     $vec {
//                         builder: Rc::clone(&self.builder),
//                         id: new_id,
//                     }
//                 }
//             }

//             // &mat vec
//             impl std::ops::Mul<$vec> for &'_ $mat {
//                 type Output = $vec;

//                 fn mul(self, rhs: $vec) -> Self::Output {
//                     let new_id = self.builder.get_new_id(crate::data::DataType::$vec);
//                     self.builder.push_instruction(crate::builder::Instruction::Mul {
//                         lhs: self.id,
//                         rhs: rhs.id,
//                         res: new_id,
//                     });
//                     $vec {
//                         builder: Rc::clone(&self.builder),
//                         id: new_id,
//                     }
//                 }
//             }

//             // &mat &vec
//             impl std::ops::Mul<&'_ $vec> for &'_ $mat {
//                 type Output = $vec;

//                 fn mul(self, rhs: &'_ $vec) -> Self::Output {
//                     let new_id = self.builder.get_new_id(crate::data::DataType::$vec);
//                     self.builder.push_instruction(crate::builder::Instruction::Mul {
//                         lhs: self.id,
//                         rhs: rhs.id,
//                         res: new_id,
//                     });
//                     $vec {
//                         builder: Rc::clone(&self.builder),
//                         id: new_id,
//                     }
//                 }
//             }



//             // mat rust_vec
//             impl std::ops::Mul<$rust_vec> for $mat {
//                 type Output = $vec;

//                 fn mul(self, rhs: $rust_vec) -> Self::Output {
//                     let rhs_id = self.builder.get_new_id(crate::data::DataType::$vec);
//                     self.builder.push_instruction(crate::builder::Instruction::Store {
//                         val: crate::data::DataVal::$vec(rhs),
//                         store: rhs_id,
//                     });
//                     let new_id = self.builder.get_new_id(crate::data::DataType::$vec);
//                     self.builder.push_instruction(crate::builder::Instruction::Mul {
//                         lhs: self.id,
//                         rhs: rhs_id,
//                         res: new_id,
//                     });
//                     $vec {
//                         builder: Rc::clone(&self.builder),
//                         id: new_id,
//                     }
//                 }
//             }

//             // mat &rust_vec
//             impl std::ops::Mul<&'_ $rust_vec> for $mat {
//                 type Output = $vec;

//                 fn mul(self, rhs: &'_ $rust_vec) -> Self::Output {
//                     let rhs_id = self.builder.get_new_id(crate::data::DataType::$vec);
//                     self.builder.push_instruction(crate::builder::Instruction::Store {
//                         val: crate::data::DataVal::$vec(*rhs),
//                         store: rhs_id,
//                     });
//                     let new_id = self.builder.get_new_id(crate::data::DataType::$vec);
//                     self.builder.push_instruction(crate::builder::Instruction::Mul {
//                         lhs: self.id,
//                         rhs: rhs_id,
//                         res: new_id,
//                     });
//                     $vec {
//                         builder: Rc::clone(&self.builder),
//                         id: new_id,
//                     }
//                 }
//             }

//             // &mat rust_vec
//             impl std::ops::Mul<$rust_vec> for &'_ $mat {
//                 type Output = $vec;

//                 fn mul(self, rhs: $rust_vec) -> Self::Output {
//                     let rhs_id = self.builder.get_new_id(crate::data::DataType::$vec);
//                     self.builder.push_instruction(crate::builder::Instruction::Store {
//                         val: crate::data::DataVal::$vec(rhs),
//                         store: rhs_id,
//                     });
//                     let new_id = self.builder.get_new_id(crate::data::DataType::$vec);
//                     self.builder.push_instruction(crate::builder::Instruction::Mul {
//                         lhs: self.id,
//                         rhs: rhs_id,
//                         res: new_id,
//                     });
//                     $vec {
//                         builder: Rc::clone(&self.builder),
//                         id: new_id,
//                     }
//                 }
//             }

//             // &mat &rust_vec
//             impl std::ops::Mul<&'_ $rust_vec> for &'_ $mat {
//                 type Output = $vec;

//                 fn mul(self, rhs: &'_ $rust_vec) -> Self::Output {
//                     let rhs_id = self.builder.get_new_id(crate::data::DataType::$vec);
//                     self.builder.push_instruction(crate::builder::Instruction::Store {
//                         val: crate::data::DataVal::$vec(*rhs),
//                         store: rhs_id,
//                     });
//                     let new_id = self.builder.get_new_id(crate::data::DataType::$vec);
//                     self.builder.push_instruction(crate::builder::Instruction::Mul {
//                         lhs: self.id,
//                         rhs: rhs_id,
//                         res: new_id,
//                     });
//                     $vec {
//                         builder: Rc::clone(&self.builder),
//                         id: new_id,
//                     }
//                 }
//             }



//             // rust_mat vec
//             impl std::ops::Mul<$vec> for $rust_mat {
//                 type Output = $vec;

//                 fn mul(self, rhs: $vec) -> Self::Output {
//                     let lhs_id = rhs.builder.get_new_id(crate::data::DataType::$vec);
//                     rhs.builder.push_instruction(crate::builder::Instruction::Store {
//                         val: crate::data::DataVal::$mat(self),
//                         store: lhs_id,
//                     });
//                     let new_id = rhs.builder.get_new_id(crate::data::DataType::$vec);
//                     rhs.builder.push_instruction(crate::builder::Instruction::Mul {
//                         lhs: lhs_id,
//                         rhs: rhs.id,
//                         res: new_id,
//                     });
//                     $vec {
//                         builder: Rc::clone(&rhs.builder),
//                         id: new_id,
//                     }
//                 }
//             }

//             // rust_mat &vec
//             impl std::ops::Mul<&'_ $vec> for $rust_mat {
//                 type Output = $vec;

//                 fn mul(self, rhs: &'_ $vec) -> Self::Output {
//                     let lhs_id = rhs.builder.get_new_id(crate::data::DataType::$vec);
//                     rhs.builder.push_instruction(crate::builder::Instruction::Store {
//                         val: crate::data::DataVal::$mat(self),
//                         store: lhs_id,
//                     });
//                     let new_id = rhs.builder.get_new_id(crate::data::DataType::$vec);
//                     rhs.builder.push_instruction(crate::builder::Instruction::Mul {
//                         lhs: lhs_id,
//                         rhs: rhs.id,
//                         res: new_id,
//                     });
//                     $vec {
//                         builder: Rc::clone(&rhs.builder),
//                         id: new_id,
//                     }
//                 }
//             }

//             // &rust_mat vec
//             impl std::ops::Mul<$vec> for &'_ $rust_mat {
//                 type Output = $vec;

//                 fn mul(self, rhs: $vec) -> Self::Output {
//                     let lhs_id = rhs.builder.get_new_id(crate::data::DataType::$vec);
//                     rhs.builder.push_instruction(crate::builder::Instruction::Store {
//                         val: crate::data::DataVal::$mat(*self),
//                         store: lhs_id,
//                     });
//                     let new_id = rhs.builder.get_new_id(crate::data::DataType::$vec);
//                     rhs.builder.push_instruction(crate::builder::Instruction::Mul {
//                         lhs: lhs_id,
//                         rhs: rhs.id,
//                         res: new_id,
//                     });
//                     $vec {
//                         builder: Rc::clone(&rhs.builder),
//                         id: new_id,
//                     }
//                 }
//             }

//             // &rust_mat &vec
//             impl std::ops::Mul<&'_ $vec> for &'_ $rust_mat {
//                 type Output = $vec;

//                 fn mul(self, rhs: &'_ $vec) -> Self::Output {
//                     let lhs_id = rhs.builder.get_new_id(crate::data::DataType::$vec);
//                     rhs.builder.push_instruction(crate::builder::Instruction::Store {
//                         val: crate::data::DataVal::$mat(*self),
//                         store: lhs_id,
//                     });
//                     let new_id = rhs.builder.get_new_id(crate::data::DataType::$vec);
//                     rhs.builder.push_instruction(crate::builder::Instruction::Mul {
//                         lhs: lhs_id,
//                         rhs: rhs.id,
//                         res: new_id,
//                     });
//                     $vec {
//                         builder: Rc::clone(&rhs.builder),
//                         id: new_id,
//                     }
//                 }
//             }
//         )*
        
//     };
// }

// gen_vec_mat_ops!(
//     Mat2, GlamMat2, Vec2, GlamVec2,
//     Mat3, GlamMat3, Vec3, GlamVec3,
//     Mat4, GlamMat4, Vec4, GlamVec4,
//     DMat2, GlamDMat2, DVec2, GlamDVec2,
//     DMat3, GlamDMat3, DVec3, GlamDVec3,
//     DMat4, GlamDMat4, DVec4, GlamDVec4,
// );