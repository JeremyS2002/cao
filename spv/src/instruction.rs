
use std::rc::Rc;
use std::cell::RefCell;

use either::*;
use rspirv::dr::Builder;

use crate::ScalarType;

// lhs rhs instruction
// ================================================================================
// ================================================================================
// ================================================================================

/// Note assign ops are implemented by setting the store id to the same as lhs id
#[derive(Clone, Copy, Debug)]
pub enum OpLhsRhsType {
    Add,
    Sub,
    Mul,
    Div,
    BitAnd,
    BitOr,
    BitXor,
    LogicalAnd,
    LogicalOr,
    LogicalEqual,
    LogicalNotEqual,
    Cross,
    Dot,
}

pub struct OpLhsRhs {
    pub ty: OpLhsRhsType,
    pub lhs: (usize, crate::Type),
    pub rhs: (usize, crate::Type),
    pub store: (usize, crate::Type),
}

impl OpLhsRhs {
    fn compile(&self, b: &mut crate::RSpirvBuilder, _: &crate::ShaderMapInfo, func_info: &mut crate::FuncMapInfo) -> bool {
        let spv_res_ty = self.store.1.rspirv(b);

        let spv_lhs_var = func_info.var(b, self.lhs.0, &self.lhs.1);
        let spv_lhs_ty = self.lhs.1.rspirv(b);
        let mut spv_lhs_obj = b.load(spv_lhs_ty, None, spv_lhs_var, None, None).unwrap();

        let spv_rhs_var = func_info.var(b, self.rhs.0, &self.rhs.1);
        let spv_rhs_ty = self.rhs.1.rspirv(b);
        let mut spv_rhs_obj = b.load(spv_rhs_ty, None, spv_rhs_var, None, None).unwrap();

        let f = self.get_fn_pointer(&mut spv_lhs_obj, &mut spv_rhs_obj, b);
        
        let spv_res_obj = f(b, spv_res_ty, None, spv_lhs_obj, spv_rhs_obj).unwrap();
        let spv_res_var = func_info.var(b, self.store.0, &self.store.1);

        b.store(spv_res_var, spv_res_obj, None, None).unwrap();

        false
    }

    fn get_fn_pointer(&self, spv_lhs_obj: &mut u32, spv_rhs_obj: &mut u32, b: &mut crate::RSpirvBuilder) -> Box<dyn FnOnce(&mut rspirv::dr::Builder, u32, Option<u32>, u32, u32) -> Result<u32, rspirv::dr::Error>> {
        let f = match self.ty {
            OpLhsRhsType::Add => Box::new(self.get_add_fn_pointer()),
            OpLhsRhsType::Sub => Box::new(self.get_sub_fn_pointer()),
            OpLhsRhsType::Mul => Box::new(self.get_mul_fn_pointer(spv_lhs_obj, spv_rhs_obj)),
            OpLhsRhsType::Div => Box::new(self.get_div_fn_pointer(b, spv_lhs_obj, spv_rhs_obj)),
            OpLhsRhsType::BitOr => todo!(),
            OpLhsRhsType::BitXor => todo!(),
            OpLhsRhsType::BitAnd => todo!(),
            OpLhsRhsType::LogicalAnd => Box::new(Builder::logical_and as _),
            OpLhsRhsType::LogicalOr => Box::new(Builder::logical_or as _),
            OpLhsRhsType::LogicalEqual => Box::new(Builder::logical_equal as _),
            OpLhsRhsType::LogicalNotEqual => Box::new(Builder::logical_not_equal as _),
            OpLhsRhsType::Cross => self.get_cross_fn_pointer(b.ext),
            OpLhsRhsType::Dot => Box::new(self.get_dot_fn_pointer()),
        };
        f
    }

    fn get_add_fn_pointer(&self) -> fn(&mut rspirv::dr::Builder, u32, Option<u32>, u32, u32) -> Result<u32, rspirv::dr::Error> {
        match self.lhs.1 {
            crate::Type::Scalar(s0) => match self.rhs.1 {
                crate::Type::Scalar(_) => {
                    // add lhs (scalar) rhs (scalar)
                    if s0.is_int() || s0.is_uint() {
                        Builder::i_add
                    } else if s0.is_float() {
                        Builder::f_add
                    } else {
                        unreachable!()
                    }
                },
                _ => unreachable!()
            },
            crate::Type::Vector(v0) => match self.rhs.1 {
                crate::Type::Vector(_) => {
                    // add lhs (vector) rhs (vector)
                    if v0.is_int() || v0.is_uint() {
                        Builder::i_add
                    } else if v0.is_float() {
                        Builder::f_add
                    } else {
                        unreachable!()
                    }
                },
                _ => unreachable!("{:?} {:?}", self.lhs.1, self.rhs.1)
            },
            crate::Type::Matrix(m0) => match self.rhs.1 {
                crate::Type::Matrix(_) => {
                    // add lhs (matrix) rhs (matrix)
                    if m0.is_int() || m0.is_uint() {
                        Builder::i_add
                    } else if m0.is_float() {
                        Builder::f_add
                    } else {
                        unreachable!()
                    }
                },
                _ => unreachable!()
            },
            _ => unreachable!()
        }
    }

    fn get_sub_fn_pointer(&self) -> fn(&mut Builder, u32, Option<u32>, u32, u32) -> Result<u32, rspirv::dr::Error> {
        match self.lhs.1 {
            crate::Type::Scalar(s0) => match self.rhs.1 {
                crate::Type::Scalar(_) => {
                    // sub lhs (scalar) rhs (scalar)
                    if s0.is_int() || s0.is_uint() {
                        Builder::i_sub
                    } else if s0.is_float() {
                        Builder::f_sub
                    } else {
                        unreachable!()
                    }
                },
                _ => unreachable!()
            },
            crate::Type::Vector(v0) => match self.rhs.1 {
                crate::Type::Vector(_) => {
                    // sub lhs (vector) rhs (vector)
                    if v0.is_int() || v0.is_uint() {
                        Builder::i_sub
                    } else if v0.is_float() {
                        Builder::f_sub
                    } else {
                        unreachable!()
                    }
                },
                _ => unreachable!()
            },
            crate::Type::Matrix(m0) => match self.rhs.1 {
                crate::Type::Matrix(_) => {
                    // sub lhs (matrix) rhs (matrix)
                    if m0.is_int() || m0.is_uint() {
                        Builder::i_sub
                    } else if m0.is_float() {
                        Builder::f_sub
                    } else {
                        unreachable!()
                    }
                },
                _ => unreachable!()
            },
            _ => unreachable!()
        }
    }

    fn get_mul_fn_pointer(&self, spv_lhs_obj: &mut u32, spv_rhs_obj: &mut u32) -> fn(&mut Builder, u32, Option<u32>, u32, u32) -> Result<u32, rspirv::dr::Error> {
        match self.lhs.1 {
            crate::Type::Scalar(s0) => match self.rhs.1 {
                crate::Type::Scalar(_) => {
                    // mul lhs (scalar) rhs (scalar)
                    if s0.is_int() || s0.is_uint() {
                        Builder::i_mul
                    } else if s0.is_float() {
                        Builder::f_mul
                    } else {
                        unreachable!()
                    }
                },
                crate::Type::Vector(_) => {
                    // mul lhs (scalar) rhs (vector)
                    std::mem::swap(spv_lhs_obj, spv_rhs_obj);
                    Builder::vector_times_scalar
                },
                crate::Type::Matrix(_) => {
                    // mul lhs (scalar) rhs (matrix)
                    std::mem::swap(spv_lhs_obj, spv_rhs_obj);
                    Builder::matrix_times_scalar
                },
                _ => unreachable!()
            },
            crate::Type::Vector(v0) => match self.rhs.1 {
                crate::Type::Scalar(_) => {
                    // mul lhs (vector) rhs (scalar)
                    Builder::vector_times_scalar
                },
                crate::Type::Vector(_) => {
                    // mul lhs (vector) rhs (vector)
                    if v0.is_int() || v0.is_uint() {
                        Builder::i_mul
                    } else if v0.is_float() {
                        Builder::f_mul
                    } else {
                        unreachable!()
                    }
                },
                _ => unreachable!()
            },
            crate::Type::Matrix(_) => match self.rhs.1 {
                crate::Type::Scalar(_) => {
                    // mul lhs (matrix) rhs (scalar)
                    Builder::matrix_times_scalar
                },
                crate::Type::Vector(_) => {
                    // mul lhs (matrix) rhs (vector)
                    Builder::matrix_times_vector
                },
                crate::Type::Matrix(_) => {
                    // mul lhs (matrix) rhs (matrix)
                    Builder::matrix_times_matrix
                },
                _ => unreachable!()
            },
            _ => unreachable!()
        }
    }

    fn get_div_fn_pointer(&self, b: &mut crate::RSpirvBuilder, spv_lhs_obj: &mut u32, spv_rhs_obj: &mut u32) -> fn(&mut Builder, u32, Option<u32>, u32, u32) -> Result<u32, rspirv::dr::Error> {
        match self.lhs.1 {
            crate::Type::Scalar(s0) => match self.rhs.1 {
                crate::Type::Scalar(_) => {
                    // div lhs (scalar) rhs (scalar)
                    if s0.is_int() {
                        Builder::s_div
                    } else if s0.is_uint() {
                        Builder::u_div
                    } else if s0.is_float() {
                        Builder::f_div
                    } else {
                        unreachable!()
                    }
                },
                crate::Type::Vector(v1) => {
                    // div lhs (scalar) rhs (vector)
                    let spv_vec_ty = v1.rspirv(b);
                    let component_count = v1.n_scalar;
                    let spv_lhs_vec_obj = b.composite_construct(spv_vec_ty, None, (0..component_count).map(|_| *spv_lhs_obj)).unwrap();
                    let _ = std::mem::replace(spv_lhs_obj, spv_lhs_vec_obj);

                    if s0.is_int() {
                        Builder::s_div
                    } else if s0.is_uint() {
                        Builder::u_div
                    } else if s0.is_float() {
                        Builder::f_div
                    } else {
                        unreachable!()
                    }
                },
                crate::Type::Matrix(m1) => {
                    // div lhs (scalar) rhs (matrix)
                    let spv_vec_ty = m1.vec_ty.rspirv(b);
                    let spv_mat_ty = m1.rspirv(b);
                    let vec_component_count = m1.vec_ty.n_scalar;
                    let mat_component_count = m1.n_vec;
                    let spv_lhs_vec_obj = b.composite_construct(spv_vec_ty, None, (0..vec_component_count).map(|_| *spv_lhs_obj)).unwrap();
                    let spv_lhs_mat_obj = b.composite_construct(spv_mat_ty, None, (0..mat_component_count).map(|_| spv_lhs_vec_obj)).unwrap();
                    let _ = std::mem::replace(spv_lhs_obj, spv_lhs_mat_obj);

                    if s0.is_int() {
                        Builder::s_div
                    } else if s0.is_uint() {
                        Builder::u_div
                    } else if s0.is_float() {
                        Builder::f_div
                    } else {
                        unreachable!()
                    }
                }
                _ => unreachable!()
            },
            crate::Type::Vector(v0) => match self.rhs.1 {
                crate::Type::Scalar(s1) => {
                    // div lhs (vector) rhs (scalar)
                    let spv_vec_ty = v0.rspirv(b);
                    let component_count = v0.n_scalar;
                    let spv_rhs_vec_obj = b.composite_construct(spv_vec_ty, None, (0..component_count).map(|_| *spv_rhs_obj)).unwrap();
                    let _ = std::mem::replace(spv_rhs_obj, spv_rhs_vec_obj);
    
                    if s1.is_int() {
                        Builder::s_div
                    } else if s1.is_uint() {
                        Builder::u_div
                    } else if s1.is_float() {
                        Builder::f_div
                    } else {
                        unreachable!()
                    }
                },
                crate::Type::Vector(_) => {
                    // div lhs (vector) rhs (vector)
                    if v0.scalar_ty.is_int() {
                        Builder::s_div
                    } else if v0.scalar_ty.is_uint() {
                        Builder::u_div
                    } else if v0.scalar_ty.is_float() {
                        Builder::f_div
                    } else {
                        unreachable!()
                    }
                },
                _ => unreachable!()
            },
            crate::Type::Matrix(m0) => match self.rhs.1 {
                crate::Type::Scalar(s1) => {
                    // div lhs (matrix) rhs (scalar)
                    let spv_vec_ty = m0.vec_ty.rspirv(b);
                    let spv_mat_ty = m0.rspirv(b);
                    let vec_component_count = m0.vec_ty.n_scalar;
                    let mat_component_count = m0.n_vec;
                    let spv_rhs_vec_obj = b.composite_construct(spv_vec_ty, None, (0..vec_component_count).map(|_| *spv_rhs_obj)).unwrap();
                    let spv_rhs_mat_obj = b.composite_construct(spv_mat_ty, None, (0..mat_component_count).map(|_| spv_rhs_vec_obj)).unwrap();
                    let _ = std::mem::replace(spv_rhs_obj, spv_rhs_mat_obj);

                    if s1.is_int() {
                        Builder::s_div
                    } else if s1.is_uint() {
                        Builder::u_div
                    } else if s1.is_float() {
                        Builder::f_div
                    } else {
                        unreachable!()
                    }
                },
                _ => unreachable!()
            }
            _ => unreachable!()
        }
    }

    fn get_dot_fn_pointer(&self) -> fn(&mut Builder, u32, Option<u32>, u32, u32) -> Result<u32, rspirv::dr::Error> {
        Builder::dot
    }

    fn get_cross_fn_pointer(&self, ext: u32) -> Box<dyn FnOnce(&mut Builder, u32, Option<u32>, u32, u32) -> Result<u32, rspirv::dr::Error>> {
        Box::new(move |builder: &mut rspirv::dr::Builder, result_type: u32, result_id: Option<u32>, lhs: u32, rhs: u32| {
            Builder::ext_inst(builder, result_type, result_id, ext, rspirv::spirv::GLOp::Cross as _, [rspirv::dr::Operand::IdRef(lhs), rspirv::dr::Operand::IdRef(rhs)])
        })
    }
}

// op lhs
// ================================================================================
// ================================================================================
// ================================================================================

pub enum OpLhsType {
    LogicalNot,
    Normalize,
    Length,
    Exp,
    Exp2,
    Sin,
    Cos,
    Tan,
    ASin,
    ACos,
    ATan,
    
}

pub struct OpLhs {
    pub ty: OpLhsType,
    pub lhs: (usize, crate::Type),
    pub store: (usize, crate::Type),
}

impl OpLhs {
    fn compile(&self, b: &mut crate::RSpirvBuilder, _: &crate::ShaderMapInfo, func_info: &mut crate::FuncMapInfo) -> bool {
        let spv_lhs_ty = self.lhs.1.rspirv(b);
        let spv_lhs_var = func_info.var(b, self.lhs.0, &self.lhs.1);
        let spv_lhs_obj = b.load(spv_lhs_ty, None, spv_lhs_var, None, None).unwrap();

        let spv_res_ty = self.store.1.rspirv(b);

        let ext = b.ext;

        let f: Box<dyn FnOnce(&mut Builder, u32, Option<u32>, u32) -> Result<u32, rspirv::dr::Error>> = match self.ty {
            OpLhsType::LogicalNot => Box::new(Builder::logical_not),
            OpLhsType::Normalize => Box::new(move |builder: &mut rspirv::dr::Builder, result_type: u32, result_id: Option<u32>, operand: u32| {
                Builder::ext_inst(builder, result_type, result_id, ext, rspirv::spirv::GLOp::Normalize as _, Some(rspirv::dr::Operand::IdRef(operand)))
            }),
            OpLhsType::Length => Box::new(move |builder: &mut rspirv::dr::Builder, result_type: u32, result_id: Option<u32>, operand: u32| {
                Builder::ext_inst(builder, result_type, result_id, ext, rspirv::spirv::GLOp::Length as _, Some(rspirv::dr::Operand::IdRef(operand)))
            }),
            OpLhsType::Exp => Box::new(move |builder: &mut rspirv::dr::Builder, result_type: u32, result_id: Option<u32>, operand: u32| {
                Builder::ext_inst(builder, result_type, result_id, ext, rspirv::spirv::GLOp::Exp as _, Some(rspirv::dr::Operand::IdRef(operand)))
            }),
            OpLhsType::Exp2 => Box::new(move |builder: &mut rspirv::dr::Builder, result_type: u32, result_id: Option<u32>, operand: u32| {
                Builder::ext_inst(builder, result_type, result_id, ext, rspirv::spirv::GLOp::Exp2 as _, Some(rspirv::dr::Operand::IdRef(operand)))
            }),
            OpLhsType::Sin => Box::new(move |builder: &mut rspirv::dr::Builder, result_type: u32, result_id: Option<u32>, operand: u32| {
                Builder::ext_inst(builder, result_type, result_id, ext, rspirv::spirv::GLOp::Sin as _, Some(rspirv::dr::Operand::IdRef(operand)))
            }),
            OpLhsType::Cos => Box::new(move |builder: &mut rspirv::dr::Builder, result_type: u32, result_id: Option<u32>, operand: u32| {
                Builder::ext_inst(builder, result_type, result_id, ext, rspirv::spirv::GLOp::Cos as _, Some(rspirv::dr::Operand::IdRef(operand)))
            }),
            OpLhsType::Tan => Box::new(move |builder: &mut rspirv::dr::Builder, result_type: u32, result_id: Option<u32>, operand: u32| {
                Builder::ext_inst(builder, result_type, result_id, ext, rspirv::spirv::GLOp::Tan as _, Some(rspirv::dr::Operand::IdRef(operand)))
            }),
            OpLhsType::ASin => Box::new(move |builder: &mut rspirv::dr::Builder, result_type: u32, result_id: Option<u32>, operand: u32| {
                Builder::ext_inst(builder, result_type, result_id, ext, rspirv::spirv::GLOp::Asin as _, Some(rspirv::dr::Operand::IdRef(operand)))
            }),
            OpLhsType::ACos => Box::new(move |builder: &mut rspirv::dr::Builder, result_type: u32, result_id: Option<u32>, operand: u32| {
                Builder::ext_inst(builder, result_type, result_id, ext, rspirv::spirv::GLOp::Acos as _, Some(rspirv::dr::Operand::IdRef(operand)))
            }),
            OpLhsType::ATan => Box::new(move |builder: &mut rspirv::dr::Builder, result_type: u32, result_id: Option<u32>, operand: u32| {
                Builder::ext_inst(builder, result_type, result_id, ext, rspirv::spirv::GLOp::Atan as _, Some(rspirv::dr::Operand::IdRef(operand)))
            }),
        };

        let spv_res_obj = f(b, spv_res_ty, None, spv_lhs_obj).unwrap();

        let spv_res_var = func_info.var(b, self.store.0, &self.store.1);
        b.store(spv_res_var, spv_res_obj, None, None).unwrap();

        false
    }
}

// vector swizzle
// ================================================================================
// ================================================================================
// ================================================================================

pub struct OpVectorShuffle {
    pub src: (usize, crate::VectorType),
    pub dst: (usize, crate::VectorType),
    pub components: [u32; 4],
}

impl OpVectorShuffle {
    fn compile(&self, b: &mut crate::RSpirvBuilder, _: &crate::ShaderMapInfo, func_info: &mut crate::FuncMapInfo) -> bool {
        let src_spv_var = func_info.var(b, self.src.0, &crate::Type::Vector(self.src.1));
        let src_obj_ty = self.src.1.rspirv(b);
        let dst_obj_ty = self.dst.1.rspirv(b);

        let src_spv_obj = b.load(src_obj_ty, None, src_spv_var, None, None).unwrap();

        let component_count = self.dst.1.n_scalar;
        let components = self.components
            .iter()
            .take(component_count as _)
            .cloned()
            .collect::<Vec<_>>();
        let dst_spv_obj = b.vector_shuffle(dst_obj_ty, None, src_spv_obj, src_spv_obj, components).unwrap();
        
        let dst_spv_var = func_info.var(b, self.dst.0, &crate::Type::Vector(self.dst.1));
        b.store(dst_spv_var, dst_spv_obj, None, None).unwrap();

        false
    }
}

// op load store
// ================================================================================
// ================================================================================
// ================================================================================

pub enum OpLoadStoreData {
    Input { 
        location: usize,
    },
    Output {
        location: usize,
    },
    UniformField {
        field: u32,
        id: usize,
    },
    Uniform {
        id: usize,
    },
    Storage {
        id: usize,
    },
    StorageElement {
        id: usize,
        element: (usize, crate::Type),
    },
    StorageElementField {
        id: usize,
        element: (usize, crate::Type),
        field: u32,
    },
    Variable {
        id: usize,
    },
    Struct {
        id: usize,
        struct_ty: crate::StructType,
        field: u32,
    },
    ArrayElement {
        id: usize,
        array_ty: crate::ArrayType,
        index: (usize, crate::Type),
    },
    PushConstant,
    PushConstantField {
        field: u32,
    }
}

impl OpLoadStoreData {
    fn get_spv_var(&self, b: &mut crate::RSpirvBuilder, shader_info: &crate::ShaderMapInfo, func_info: &mut crate::FuncMapInfo, ty: &crate::Type) -> u32 {
        let spv_obj_ty = ty.rspirv(b);
        match self {
            OpLoadStoreData::Input { location } => shader_info.inputs[*location],
            OpLoadStoreData::Output { location } => shader_info.outputs[*location],
            OpLoadStoreData::UniformField { field, id } => {
                let spv_var = shader_info.uniforms[*id];
                let spv_p_ty = b.type_pointer(None, rspirv::spirv::StorageClass::Uniform, spv_obj_ty);
                let idx1 = crate::ScalarVal::UInt(0).set_rspirv(b);
                let idx2 = crate::ScalarVal::UInt(*field).set_rspirv(b);
                b.access_chain(spv_p_ty, None, spv_var, [idx1, idx2]).unwrap()
            },
            OpLoadStoreData::Uniform { id } => {
                let outer_spv_var = shader_info.uniforms[*id];
                let spv_p_ty = b.type_pointer(None, rspirv::spirv::StorageClass::Uniform, spv_obj_ty);
                let idx = crate::ScalarVal::UInt(0).set_rspirv(b);
                b.access_chain(spv_p_ty, None, outer_spv_var, Some(idx)).unwrap()
            },
            OpLoadStoreData::Storage { id } => {
                let outer_spv_var = shader_info.storages[*id];
                let spv_array_ty = b.type_runtime_array(spv_obj_ty);
                let spv_p_ty = b.type_pointer(None, rspirv::spirv::StorageClass::Uniform, spv_array_ty);
                let idx = crate::ScalarVal::UInt(0).set_rspirv(b);
                b.access_chain(spv_p_ty, None, outer_spv_var, Some(idx)).unwrap()
            },
            OpLoadStoreData::StorageElement { id, element } => {
                let spv_var = shader_info.storages[*id];
                let spv_p_ty = b.type_pointer(None, rspirv::spirv::StorageClass::Uniform, spv_obj_ty);
                let idx1 = crate::ScalarVal::UInt(0).set_rspirv(b);

                // let idx2 = crate::ScalarVal::UInt(*element).set_rspirv(b);
                let spv_idx2_ty = element.1.rspirv(b);
                let spv_idx2_var = func_info.var(b, element.0, &element.1);
                let idx2 = b.load(spv_idx2_ty, None, spv_idx2_var, None, None).unwrap();
                b.access_chain(spv_p_ty, None, spv_var, [idx1, idx2]).unwrap()
            },
            OpLoadStoreData::StorageElementField { id, element, field } => {
                let spv_var = shader_info.storages[*id];
                let spv_p_ty = b.type_pointer(None, rspirv::spirv::StorageClass::Uniform, spv_obj_ty);
                let idx1 = crate::ScalarVal::UInt(0).set_rspirv(b);

                let spv_idx2_ty = element.1.rspirv(b);
                let spv_idx2_var = func_info.var(b, element.0, &element.1);
                let idx2 = b.load(spv_idx2_ty, None, spv_idx2_var, None, None).unwrap();

                let idx3 = crate::ScalarVal::UInt(*field).set_rspirv(b);
                b.access_chain(spv_p_ty, None, spv_var, [idx1, idx2, idx3]).unwrap()
            },
            OpLoadStoreData::Variable { id } =>  func_info.var(b, *id, ty),
            OpLoadStoreData::PushConstant => {
                let spv_var = shader_info.push_constants.unwrap();
                let spv_p_ty = b.type_pointer(None, rspirv::spirv::StorageClass::PushConstant, spv_obj_ty);
                let idx = crate::ScalarVal::UInt(0).set_rspirv(b);
                b.access_chain(spv_p_ty, None, spv_var, Some(idx)).unwrap()
            },
            OpLoadStoreData::PushConstantField { field } => {
                let spv_var = shader_info.push_constants.unwrap();
                let spv_p_ty = b.type_pointer(None, rspirv::spirv::StorageClass::PushConstant, spv_obj_ty);
                let idx1 = crate::ScalarVal::UInt(0).set_rspirv(b);
                let idx2 = crate::ScalarVal::UInt(*field).set_rspirv(b);
                b.access_chain(spv_p_ty, None, spv_var, [idx1, idx2]).unwrap()
            },
            OpLoadStoreData::Struct { id, field, struct_ty } => {
                let spv_var = func_info.var(b, *id, &crate::Type::Struct(struct_ty.clone()));
                let spv_p_ty = b.type_pointer(None, rspirv::spirv::StorageClass::Function, spv_obj_ty);
                let idx = crate::ScalarVal::UInt(*field).set_rspirv(b);
                b.access_chain(spv_p_ty, None, spv_var, Some(idx)).unwrap()
            },
            OpLoadStoreData::ArrayElement { id, index, array_ty } => {
                let spv_var = func_info.var(b, *id, &crate::Type::Array(array_ty.clone()));
                let spv_p_ty = b.type_pointer(None, rspirv::spirv::StorageClass::Function, spv_obj_ty);
                // let idx = crate::ScalarVal::UInt(*index).set_rspirv(b);

                let spv_idx_ty = index.1.rspirv(b);
                let spv_idx_var = func_info.var(b, index.0, &index.1);
                let idx = b.load(spv_idx_ty, None, spv_idx_var, None, None).unwrap();

                b.access_chain(spv_p_ty, None, spv_var, Some(idx)).unwrap()
            },
            
        }
    }
}

pub struct OpLoadStore {
    pub ty: crate::Type,
    pub src: OpLoadStoreData,
    pub dst: OpLoadStoreData,
}

impl OpLoadStore {
    fn compile(&self, b: &mut crate::RSpirvBuilder, shader_info: &crate::ShaderMapInfo, func_info: &mut crate::FuncMapInfo) -> bool {
        let spv_obj_ty = self.ty.rspirv(b);
        let spv_src_var = self.src.get_spv_var(b, shader_info, func_info, &self.ty);
        let spv_obj = b.load(spv_obj_ty, None, spv_src_var, None, None).unwrap();
        let spv_dst_var = self.dst.get_spv_var(b, shader_info, func_info, &self.ty);
        b.store(spv_dst_var, spv_obj, None, None).unwrap();
        false
    }
}

// op fn call
// ================================================================================
// ================================================================================
// ================================================================================

pub struct OpFuncCall {
    pub func: usize,
    pub store_ty: crate::Type,
    pub store: usize,
    pub args: Vec<(usize, crate::Type)>,
}

impl OpFuncCall {
    fn compile(&self, _: &mut crate::RSpirvBuilder, _: &crate::ShaderMapInfo, _: &mut crate::FuncMapInfo) -> bool {
        todo!()
    }
}

// op set const
// ================================================================================
// ================================================================================
// ================================================================================

pub struct OpSetConst {
    pub val: crate::Val,
    pub store: usize,
}

impl OpSetConst {
    fn compile(&self, b: &mut crate::RSpirvBuilder, _: &crate::ShaderMapInfo, func_info: &mut crate::FuncMapInfo) -> bool {
        let spv_obj = self.val.set_rspirv(b);
        let spv_var = func_info.var(b, self.store, &self.val.ty());

        b.store(spv_var, spv_obj, None, None).unwrap();
        false
    }
}

// op cmp
// ================================================================================
// ================================================================================
// ================================================================================

pub enum CmpType {
    Eq,
    NEq,
    Lt,
    Gt,
    Le,
    Ge,
}

pub struct OpCmp {
    pub cmp: CmpType,
    pub lhs: (usize, crate::Type),
    pub rhs: (usize, crate::Type),
    pub store: usize,
}

impl OpCmp {
    fn compile(&self, b: &mut crate::RSpirvBuilder, _: &crate::ShaderMapInfo, func_info: &mut crate::FuncMapInfo) -> bool {        
        let res_spv_ty = crate::ScalarType::Bool.rspirv(b);

        let lhs_spv_var = func_info.var(b, self.lhs.0, &self.lhs.1);
        let lhs_spv_ty = self.lhs.1.rspirv(b);
        let lhs_spv_obj = b.load(lhs_spv_ty, None, lhs_spv_var, None, None).unwrap();

        let rhs_spv_var = func_info.var(b, self.rhs.0, &self.rhs.1);
        let rhs_spv_ty = self.rhs.1.rspirv(b);
        let rhs_spv_obj = b.load(rhs_spv_ty, None, rhs_spv_var, None, None).unwrap();

        let f = match self.cmp {
            CmpType::Eq => self.get_eq_fn_pointer(),
            CmpType::NEq => self.get_neq_fn_pointer(),
            CmpType::Lt => self.get_lt_fn_pointer(),
            CmpType::Gt => self.get_gt_fn_pointer(),
            CmpType::Le => self.get_le_fn_pointer(),
            CmpType::Ge => self.get_ge_fn_pointer(),
        };

        let res_spv_obj = f(b, res_spv_ty, None, lhs_spv_obj, rhs_spv_obj).unwrap();
        let res_spv_var = func_info.var(b, self.store, &crate::Type::Scalar(crate::ScalarType::Bool));
        b.store(res_spv_var, res_spv_obj, None, None).unwrap();
        false
    }

    fn get_ge_fn_pointer(&self) -> fn(&mut Builder, u32, Option<u32>, u32, u32) -> Result<u32, rspirv::dr::Error> {
        match self.lhs.1 {
            crate::Type::Scalar(s) => match s {
                crate::ScalarType::Signed(_) => Builder::s_greater_than_equal,
                crate::ScalarType::Unsigned(_) => Builder::u_greater_than_equal,
                crate::ScalarType::Float(_) => Builder::f_ord_greater_than_equal,
                _ => unreachable!()
            },
            crate::Type::Vector(v) => match v.scalar_ty {
                crate::ScalarType::Signed(_) => Builder::s_greater_than_equal,
                crate::ScalarType::Unsigned(_) => Builder::u_greater_than_equal,
                crate::ScalarType::Float(_) => Builder::f_ord_greater_than_equal,
                _ => unreachable!()
            },
            crate::Type::Matrix(m) => match m.vec_ty.scalar_ty {
                crate::ScalarType::Signed(_) => Builder::s_greater_than_equal,
                crate::ScalarType::Unsigned(_) => Builder::u_greater_than_equal,
                crate::ScalarType::Float(_) => Builder::f_ord_greater_than_equal,
                _ => unreachable!()
            },
            _ => unreachable!(),
        }
    }

    fn get_le_fn_pointer(&self) -> fn(&mut Builder, u32, Option<u32>, u32, u32) -> Result<u32, rspirv::dr::Error> {
        match self.lhs.1 {
            crate::Type::Scalar(s) => match s {
                crate::ScalarType::Signed(_) => Builder::s_less_than_equal,
                crate::ScalarType::Unsigned(_) => Builder::u_less_than_equal,
                crate::ScalarType::Float(_) => Builder::f_ord_less_than_equal,
                _ => unreachable!()
            },
            crate::Type::Vector(v) => match v.scalar_ty {
                crate::ScalarType::Signed(_) => Builder::s_less_than_equal,
                crate::ScalarType::Unsigned(_) => Builder::u_less_than_equal,
                crate::ScalarType::Float(_) => Builder::f_ord_less_than_equal,
                _ => unreachable!()
            },
            crate::Type::Matrix(m) => match m.vec_ty.scalar_ty {
                crate::ScalarType::Signed(_) => Builder::s_less_than_equal,
                crate::ScalarType::Unsigned(_) => Builder::u_less_than_equal,
                crate::ScalarType::Float(_) => Builder::f_ord_less_than_equal,
                _ => unreachable!()
            },
            _ => unreachable!(),
        }
    }

    fn get_gt_fn_pointer(&self) -> fn(&mut Builder, u32, Option<u32>, u32, u32) -> Result<u32, rspirv::dr::Error> {
        match self.lhs.1 {
            crate::Type::Scalar(s) => match s {
                crate::ScalarType::Signed(_) => Builder::s_greater_than,
                crate::ScalarType::Unsigned(_) => Builder::u_greater_than,
                crate::ScalarType::Float(_) => Builder::f_ord_greater_than,
                _ => unreachable!()
            },
            crate::Type::Vector(v) => match v.scalar_ty {
                crate::ScalarType::Signed(_) => Builder::s_greater_than,
                crate::ScalarType::Unsigned(_) => Builder::u_greater_than,
                crate::ScalarType::Float(_) => Builder::f_ord_greater_than,
                _ => unreachable!()
            },
            crate::Type::Matrix(m) => match m.vec_ty.scalar_ty {
                crate::ScalarType::Signed(_) => Builder::s_greater_than,
                crate::ScalarType::Unsigned(_) => Builder::u_greater_than,
                crate::ScalarType::Float(_) => Builder::f_ord_greater_than,
                _ => unreachable!()
            },
            _ => unreachable!(),
        }
    }

    fn get_lt_fn_pointer(&self) -> fn(&mut Builder, u32, Option<u32>, u32, u32) -> Result<u32, rspirv::dr::Error> {
        match self.lhs.1 {
            crate::Type::Scalar(s) => match s {
                crate::ScalarType::Signed(_) => Builder::s_less_than,
                crate::ScalarType::Unsigned(_) => Builder::u_less_than,
                crate::ScalarType::Float(_) => Builder::f_ord_less_than,
                _ => unreachable!()
            },
            crate::Type::Vector(v) => match v.scalar_ty {
                crate::ScalarType::Signed(_) => Builder::s_less_than,
                crate::ScalarType::Unsigned(_) => Builder::u_less_than,
                crate::ScalarType::Float(_) => Builder::f_ord_less_than,
                _ => unreachable!()
            },
            crate::Type::Matrix(m) => match m.vec_ty.scalar_ty {
                crate::ScalarType::Signed(_) => Builder::s_less_than,
                crate::ScalarType::Unsigned(_) => Builder::u_less_than,
                crate::ScalarType::Float(_) => Builder::f_ord_less_than,
                _ => unreachable!()
            },
            _ => unreachable!(),
        }
    }

    fn get_neq_fn_pointer(&self) -> fn(&mut Builder, u32, Option<u32>, u32, u32) -> Result<u32, rspirv::dr::Error> {
        match self.lhs.1 {
            crate::Type::Scalar(s) => match s {
                crate::ScalarType::Bool => Builder::logical_not_equal,
                crate::ScalarType::Signed(_) => Builder::i_not_equal,
                crate::ScalarType::Unsigned(_) => Builder::i_not_equal,
                crate::ScalarType::Float(_) => Builder::f_ord_not_equal,
            },
            crate::Type::Vector(v) => match v.scalar_ty {
                crate::ScalarType::Bool => Builder::logical_not_equal,
                crate::ScalarType::Signed(_) => Builder::i_not_equal,
                crate::ScalarType::Unsigned(_) => Builder::i_not_equal,
                crate::ScalarType::Float(_) => Builder::f_ord_not_equal,
            },
            crate::Type::Matrix(m) => match m.vec_ty.scalar_ty {
                crate::ScalarType::Bool => Builder::logical_not_equal,
                crate::ScalarType::Signed(_) => Builder::i_not_equal,
                crate::ScalarType::Unsigned(_) => Builder::i_not_equal,
                crate::ScalarType::Float(_) => Builder::f_ord_not_equal,
            },
            _ => unreachable!(),
        }
    }

    fn get_eq_fn_pointer(&self) -> fn(&mut Builder, u32, Option<u32>, u32, u32) -> Result<u32, rspirv::dr::Error> {
        match self.lhs.1 {
            crate::Type::Scalar(s) => match s {
                crate::ScalarType::Bool => Builder::logical_equal,
                crate::ScalarType::Signed(_) => Builder::i_equal,
                crate::ScalarType::Unsigned(_) => Builder::i_equal,
                crate::ScalarType::Float(_) => Builder::f_ord_equal,
            },
            crate::Type::Vector(v) => match v.scalar_ty {
                crate::ScalarType::Bool => Builder::logical_equal,
                crate::ScalarType::Signed(_) => Builder::i_equal,
                crate::ScalarType::Unsigned(_) => Builder::i_equal,
                crate::ScalarType::Float(_) => Builder::f_ord_equal,
            },
            crate::Type::Matrix(m) => match m.vec_ty.scalar_ty {
                crate::ScalarType::Bool => Builder::logical_equal,
                crate::ScalarType::Signed(_) => Builder::i_equal,
                crate::ScalarType::Unsigned(_) => Builder::i_equal,
                crate::ScalarType::Float(_) => Builder::f_ord_equal,
            },
            _ => unreachable!(),
        }
    }
}

// op composite
// ================================================================================
// ================================================================================
// ================================================================================

pub struct OpComposite {
    pub ty: crate::Type,
    pub id: usize,
    pub constituents: Vec<(usize, crate::Type)>,
}

impl OpComposite {
    fn compile(&self, b: &mut crate::RSpirvBuilder, _: &crate::ShaderMapInfo, func_info: &mut crate::FuncMapInfo) -> bool {
        let spv_res_ty = self.ty.rspirv(b);
        let spv_constituents = self.constituents
            .iter()
            .map(|(id, ty)| {
                let spv_comp_var = func_info.var(b, *id, ty);
                let spv_comp_ty = ty.rspirv(b);
                let spv_comp_obj = b.load(spv_comp_ty, None, spv_comp_var, None, None).unwrap();
                spv_comp_obj
                
            })
            .collect::<Vec<_>>();
        let spv_res_obj = b.composite_construct(spv_res_ty, None, spv_constituents).unwrap();

        let spv_res_var = func_info.var(b, self.id, &self.ty);
        b.store(spv_res_var, spv_res_obj, None, None).unwrap();
        false
    }
}

// op extract
// ================================================================================
// ================================================================================
// ================================================================================

pub struct OpExtract {
    pub src_id: usize,
    pub src_ty: crate::Type,
    pub element_ty: crate::Type,
    pub element_idx: u32,
    pub store_id: usize,
}

impl OpExtract {
    fn compile(&self, b: &mut crate::RSpirvBuilder, _: &crate::ShaderMapInfo, func_info: &mut crate::FuncMapInfo) -> bool {
        let src_spv_var = func_info.var(b, self.src_id, &self.element_ty);
        let p_ty = self.element_ty.pointer(b);
        let spv_idx = crate::ScalarVal::UInt(self.element_idx).set_rspirv(b);
        let spv_p = b.access_chain(p_ty, None, src_spv_var, Some(spv_idx)).unwrap();
        
        let dst_spv_ty = self.element_ty.rspirv(b);
        let spv_res_obj = b.load(dst_spv_ty, None, spv_p, None, None).unwrap();

        let spv_res_var = func_info.var(b, self.store_id, &self.element_ty);
        b.store(spv_res_var, spv_res_obj, None, None).unwrap();
        false
    }
}

// op combine
// ================================================================================
// ================================================================================
// ================================================================================

pub struct OpCombine {
    pub tex_ty: crate::TextureType,
    pub texture: usize,
    pub sampler: usize,
    pub store: usize,
}

impl OpCombine {
    fn compile(&self, b: &mut crate::RSpirvBuilder, shader_info: &crate::ShaderMapInfo, func_info: &mut crate::FuncMapInfo) -> bool {
        let spv_texture = shader_info.textures[self.texture];
        let spv_texture_ty = self.tex_ty.rspirv(b);
        let spv_texture_obj = b.load(spv_texture_ty, None, spv_texture, None, None).unwrap();
        
        let spv_sampler = shader_info.samplers[self.sampler];
        let spv_sampler_ty = b.type_sampler();
        let spv_sampler_obj = b.load(spv_sampler_ty, None, spv_sampler, None, None).unwrap();

        let spv_sampled_texture_ty = b.type_sampled_image(spv_texture_ty);

        let spv_sampled_texture = b.sampled_image(spv_sampled_texture_ty, None, spv_texture_obj, spv_sampler_obj).unwrap();

        func_info.vars.insert(self.store, spv_sampled_texture);
        false
    }
}

// op convert
// ================================================================================
// ================================================================================
// ================================================================================

pub struct OpConvert {
    pub src: (usize, crate::Type),
    pub dst: (usize, crate::Type),
}

impl OpConvert {
    fn compile(&self, b: &mut crate::RSpirvBuilder, _: &crate::ShaderMapInfo, func_info: &mut crate::FuncMapInfo) -> bool {
        let spv_src_ty = self.src.1.rspirv(b);
        let spv_src_var = func_info.var(b, self.src.0, &self.src.1);
        let spv_src_obj = b.load(spv_src_ty, None, spv_src_var, None, None).unwrap();

        let f = if let crate::Type::Scalar(s1) = &self.src.1 {
            if let crate::Type::Scalar(s2) = &self.dst.1 {
                match *s1 {
                    ScalarType::Signed(_) => match *s2 {
                        ScalarType::Signed(_) => Builder::s_convert,
                        ScalarType::Unsigned(_) => Builder::sat_convert_s_to_u,
                        ScalarType::Float(_) => Builder::convert_s_to_f,
                        _ => unimplemented!()
                    },
                    ScalarType::Unsigned(_) => match *s2 {
                        ScalarType::Signed(_) => Builder::sat_convert_u_to_s,
                        ScalarType::Unsigned(_) => Builder::u_convert,
                        ScalarType::Float(_) => Builder::convert_u_to_f,
                        _ => unimplemented!()
                    },
                    ScalarType::Float(_) => match *s2 {
                        ScalarType::Signed(_) => Builder::convert_f_to_s,
                        ScalarType::Unsigned(_) => Builder::convert_f_to_u,
                        ScalarType::Float(_) => Builder::f_convert,
                        _ => unimplemented!()
                    }
                    _ => unimplemented!()
                }
            } else {
                unimplemented!()
            }
        } else {
            unimplemented!()
        };

        let spv_res_ty = self.dst.1.rspirv(b);
        let spv_res_obj = f(b, spv_res_ty, None, spv_src_obj).unwrap();

        let spv_res_var = func_info.var(b, self.dst.0, &self.dst.1);
        b.store(spv_res_var, spv_res_obj, None, None).unwrap();

        false
    }
}

// op sample
// ================================================================================
// ================================================================================
// ================================================================================

pub struct OpSample {
    // Left(uniform) Right(combined)
    pub tex_ty: crate::TextureType,
    pub sampled_texture: Either<usize, usize>,
    pub coordinate: (usize, crate::Type),
    pub store: (usize, crate::Type),
    pub explict_lod: bool,
}

impl OpSample {
    fn compile(&self, b: &mut crate::RSpirvBuilder, shader_info: &crate::ShaderMapInfo, func_info: &mut crate::FuncMapInfo) -> bool {
        let spv_sampled_texture = match self.sampled_texture {
            Left(id) => shader_info.sampled_textures[id],
            Right(id) => func_info.var(b, id, &crate::Type::Texture(self.tex_ty)),
        };

        let spv_coord_ty = self.coordinate.1.rspirv(b);
        let spv_coord_var = func_info.var(b, self.coordinate.0, &self.coordinate.1);
        let spv_coord_obj = b.load(spv_coord_ty, None, spv_coord_var, None, None).unwrap();

        let spv_res_ty = self.store.1.rspirv(b);

        let spv_res_obj = if self.explict_lod {
            b.image_sample_explicit_lod(
                spv_res_ty, 
                None, 
                spv_sampled_texture, 
                spv_coord_obj,
                rspirv::spirv::ImageOperands::LOD, 
                None
            ).unwrap()
        } else {
            b.image_sample_implicit_lod(
                spv_res_ty, 
                None, 
                spv_sampled_texture, 
                spv_coord_obj,
                None, 
                None
            ).unwrap()
        };

        let spv_res_var = func_info.var(b, self.store.0, &self.store.1);
        b.store(spv_res_var, spv_res_obj, None, None).unwrap();
        false
    }
}

// op if
// ================================================================================
// ================================================================================
// ================================================================================

pub struct OpIf {
    pub condition: usize,
    pub instructions: Vec<Instruction>,
    pub then: Rc<RefCell<Option<Either<Box<OpIf>, OpElse>>>>,
}

impl OpIf {
    fn compile(&self, b: &mut crate::RSpirvBuilder, shader_info: &crate::ShaderMapInfo, func_info: &mut crate::FuncMapInfo) -> bool {
        let prev_block = func_info.block_info;

        let spv_condition_var = func_info.var(b, self.condition, &crate::Type::BOOL);
        let spv_condition_ty = crate::Type::BOOL.rspirv(b);
        let spv_condition_obj = b.load(spv_condition_ty, None, spv_condition_var, None, None).unwrap();
        
        let true_label = b.id();
        let false_label = b.id();
        let end_label = b.id();

        let block = b.selected_block().unwrap();
        b.selection_merge(end_label, rspirv::spirv::SelectionControl::NONE).unwrap();
        b.select_block(Some(block)).unwrap();
        b.branch_conditional(spv_condition_obj, true_label, false_label, None).unwrap();

        b.begin_block(Some(true_label)).unwrap();

        let mut bl = false;
        for instruction in &self.instructions {
            bl |= instruction.compile(b, shader_info, func_info);
            if bl {
                break;
            }
        }

        if !bl {
            b.branch(end_label).unwrap();
        }

        b.begin_block(Some(false_label)).unwrap();

        func_info.block_info = crate::BlockInfo::If { 
            end_label, 
        };

        let then = self.then.borrow_mut();
        let bl = if let Some(then) = &*then {
            match then {
                Left(t) => t.compile(b, shader_info, func_info),
                Right(t) => t.compile(b, shader_info, func_info),
            }
        } else {
            false
        };

        if !bl {
            b.branch(end_label).unwrap();
        }

        b.begin_block(Some(end_label)).unwrap();

        func_info.block_info = prev_block;

        false
    }
}

pub struct OpElse {
    pub instructions: Vec<Instruction>,
}

impl OpElse {
    fn compile(&self, b: &mut crate::RSpirvBuilder, shader_info: &crate::ShaderMapInfo, func_info: &mut crate::FuncMapInfo) -> bool {
        let mut bl = false;
        for instruction in &self.instructions {
            bl |= instruction.compile(b, shader_info, func_info);
            if bl {
                break;
            }
        }
        bl
    }
}

// instruction
// ================================================================================
// ================================================================================
// ================================================================================

pub enum Instruction {
    LhsRhs(OpLhsRhs),
    Lhs(OpLhs),
    VectorShuffle(OpVectorShuffle),
    LoadStore(OpLoadStore),
    FuncCall(OpFuncCall),
    SetConst(OpSetConst),
    Cmp(OpCmp),
    Composite(OpComposite),
    Extract(OpExtract),
    Sample(OpSample),
    Combine(OpCombine),
    Convert(OpConvert),
    If(OpIf),
    Return,
    Discard,
    Continue,
    Break,
}

impl Instruction {
    pub(crate) fn compile(&self, b: &mut crate::RSpirvBuilder, shader_info: &crate::ShaderMapInfo, func_info: &mut crate::FuncMapInfo) -> bool {
        match self {
            Instruction::LhsRhs(o) => o.compile(b, shader_info, func_info),
            Instruction::Lhs(o) => o.compile(b, shader_info, func_info),
            Instruction::VectorShuffle(o) => o.compile(b, shader_info, func_info),
            Instruction::LoadStore(o) => o.compile(b, shader_info, func_info),
            Instruction::FuncCall(o) => o.compile(b, shader_info, func_info),
            Instruction::SetConst(o) => o.compile(b, shader_info, func_info),
            Instruction::Cmp(o) => o.compile(b, shader_info, func_info),
            Instruction::Composite(o) => o.compile(b, shader_info, func_info),
            Instruction::Extract(o) => o.compile(b, shader_info, func_info),
            Instruction::Sample(o) => o.compile(b, shader_info, func_info),
            Instruction::Combine(o) => o.compile(b, shader_info, func_info),
            Instruction::Convert(o) => o.compile(b, shader_info, func_info),
            Instruction::If(o) => o.compile(b, shader_info, func_info),
            Instruction::Return => todo!(),
            Instruction::Discard => {
                b.kill().unwrap();
                true
            },
            Instruction::Continue => todo!(),
            Instruction::Break => todo!(),
            
        }
    }
}
