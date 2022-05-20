use std::collections::HashMap;

use crate::data::{PrimitiveVal, PrimitiveType};

#[derive(Clone, PartialEq, Debug)]
pub enum Instruction {
    Store {
        /// Declare a constant with this value
        val: PrimitiveVal,
        /// Store into variable with this id
        store: usize,
    },
    Add { lhs: (usize, PrimitiveType), rhs: (usize, PrimitiveType), res: (usize, PrimitiveType) },
    Sub { lhs: (usize, PrimitiveType), rhs: (usize, PrimitiveType), res: (usize, PrimitiveType) },
    Mul { lhs: (usize, PrimitiveType), rhs: (usize, PrimitiveType), res: (usize, PrimitiveType) },
    Div { lhs: (usize, PrimitiveType), rhs: (usize, PrimitiveType), res: (usize, PrimitiveType) },
    AddAssign { lhs: (usize, PrimitiveType), rhs: (usize, PrimitiveType) },
    SubAssign { lhs: (usize, PrimitiveType), rhs: (usize, PrimitiveType) },
    MulAssign { lhs: (usize, PrimitiveType), rhs: (usize, PrimitiveType) },
    DivAssign { lhs: (usize, PrimitiveType), rhs: (usize, PrimitiveType) },
    BitAnd { lhs: (usize, PrimitiveType), rhs: (usize, PrimitiveType), res: (usize, PrimitiveType) },
    BitOr { lhs: (usize, PrimitiveType), rhs: (usize, PrimitiveType), res: (usize, PrimitiveType) },
    BitXor { lhs: (usize, PrimitiveType), rhs: (usize, PrimitiveType), res: (usize, PrimitiveType) },
    BitAndAssign { lhs: (usize, PrimitiveType), rhs: (usize, PrimitiveType) },
    BitOrAssign { lhs: (usize, PrimitiveType), rhs: (usize, PrimitiveType) },
    BitXorAssign { lhs: (usize, PrimitiveType), rhs: (usize, PrimitiveType) },
    VectorShuffle { src: (usize, PrimitiveType), dst: (usize, PrimitiveType), components: [u32; 4] },
    IfChain {
        conditions: Vec<usize>,
        instructions: Vec<Vec<Instruction>>,
    },
    Loop {
        condition: usize,
        body: Vec<Instruction>,
    },
    LoadIn { index: usize, ty: PrimitiveType, store: usize },
    StoreOut { index: usize, ty: PrimitiveType, read: usize },
    LoadUniform {

    },
    LoadStorage {

    },
    StoreStorage {

    },
    Break,
    Continue,
    FnCall {
        fn_id: usize,
        store_id: usize,
        arguments: Vec<(usize, PrimitiveType)>,
    },
    Return {
        id: usize,
    }
}

impl Instruction {
    pub fn process(
        &self, 
        builder: &mut rspirv::dr::Builder, 
        var_map: &mut HashMap<usize, u32>,
        function_map: &HashMap<usize, usize>,
        inputs: &[u32],
        outputs: &[u32],
    ) {
        match self {
            Instruction::Store { 
                val, 
                store 
            } => {
                let v = val.set(builder);
                var_map.insert(*store, v);
            },
            Instruction::Add { lhs, rhs, res } => process_add(var_map, lhs, builder, rhs, res),
            Instruction::Sub { lhs, rhs, res } => process_sub(var_map, lhs, builder, rhs, res),
            Instruction::Mul { lhs, rhs, res } => process_mul(var_map, lhs, builder, rhs, res),
            Instruction::Div { lhs, rhs, res } => process_div(var_map, lhs, builder, rhs, res),
            Instruction::AddAssign { lhs, rhs } => process_add_assign(var_map, lhs, builder, rhs),
            Instruction::SubAssign { lhs, rhs } => process_sub_assign(var_map, lhs, builder, rhs),
            Instruction::MulAssign { lhs, rhs } => process_mul_assign(var_map, lhs, builder, rhs),
            Instruction::DivAssign { lhs, rhs } => process_div_assign(var_map, lhs, builder, rhs),
            Instruction::BitAnd { lhs, rhs, res } => process_bit_op(var_map, lhs, builder, rhs, res, rspirv::dr::Builder::bitwise_and),
            Instruction::BitOr { lhs, rhs, res } => process_bit_op(var_map, lhs, builder, rhs, res, rspirv::dr::Builder::bitwise_or),
            Instruction::BitXor { lhs, rhs, res } => process_bit_op(var_map, lhs, builder, rhs, res, rspirv::dr::Builder::bitwise_xor),
            Instruction::BitAndAssign { lhs, rhs, } => process_bit_op_assign(var_map, lhs, builder, rhs, rspirv::dr::Builder::bitwise_and),
            Instruction::BitOrAssign { lhs, rhs, } => process_bit_op_assign(var_map, lhs, builder, rhs,  rspirv::dr::Builder::bitwise_or),
            Instruction::BitXorAssign { lhs, rhs, } => process_bit_op_assign(var_map, lhs, builder, rhs, rspirv::dr::Builder::bitwise_xor),
            Instruction::IfChain { 
                conditions, 
                instructions 
            } => {
                
            },
            Instruction::Loop { 
                condition, 
                body 
            } => {
                
            },
            Instruction::Break => {
                
            },
            Instruction::Continue => {
                
            },
            Instruction::FnCall { 
                fn_id, 
                store_id, 
                arguments 
            } => {
                
            },
            Instruction::Return { 
                id 
            } => {
                
            },
            Instruction::LoadIn { index, ty, store } => {
                let ty = ty.raw_ty(builder);
                let input_var = *inputs.get(*index).unwrap();
                let spv_obj = builder.load(ty, None, input_var, None, None).unwrap();
            
                let spv_var = if let Some(&spv_var) = var_map.get(store) {
                    spv_var
                } else {
                    let pointer_ty = builder.type_pointer(None, rspirv::spirv::StorageClass::Function, ty);
                    let variable = builder.variable(pointer_ty, None, rspirv::spirv::StorageClass::Function, None);
                    variable
                };
                
                builder.store(spv_var, spv_obj, None, None).unwrap();
                var_map.insert(*store, spv_var);
            },
            Instruction::StoreOut { index, ty, read } => {
                let ty = ty.raw_ty(builder);
                let spv_var = *var_map.get(read).unwrap();
                let spv_obj = builder.load(ty, None, spv_var, None, None).unwrap();

                let output_var = *outputs.get(*index).unwrap();

                builder.store(output_var, spv_obj, None, None).unwrap();
            },
            Instruction::LoadUniform {  } => todo!(),
            Instruction::LoadStorage {  } => todo!(),
            Instruction::StoreStorage {  } => todo!(),
            Instruction::VectorShuffle { src, dst, components } => process_vector_shuffle(var_map, builder, src, dst, *components),
        }
    }
}

fn process_vector_shuffle(var_map: &mut HashMap<usize, u32>, builder: &mut rspirv::dr::Builder, src: &(usize, PrimitiveType), dst: &(usize, PrimitiveType), components: [u32; 4]) {
    let src_var = *var_map.get(&src.0).unwrap();
    let src_obj_ty = src.1.raw_ty(builder);
    let dst_obj_ty = dst.1.raw_ty(builder);

    let dst_obj = if dst.1.is_vector() {

        let src_obj = builder.load(src_obj_ty, None, src_var, None, None).unwrap();

        let component_count = match dst.1 {
            PrimitiveType::IVec2 => 2,
            PrimitiveType::IVec3 => 3,
            PrimitiveType::IVec4 => 4,
            PrimitiveType::UVec2 => 2,
            PrimitiveType::UVec3 => 3,
            PrimitiveType::UVec4 => 4,
            PrimitiveType::Vec2 => 2,
            PrimitiveType::Vec3 => 3,
            PrimitiveType::Vec4 => 4,
            PrimitiveType::DVec2 => 2,
            PrimitiveType::DVec3 => 3,
            PrimitiveType::DVec4 => 4,
            _ => unreachable!(),
        };
        let components = &components[0..component_count];
        builder.vector_shuffle(dst_obj_ty, None, src_obj, src_obj, components.into_iter().map(|a| *a)).unwrap()
    } else {
        let pointer_ty = builder.type_pointer(None, rspirv::spirv::StorageClass::Function, dst_obj_ty);
        let pointer = builder.access_chain(pointer_ty, None, src_var, Some(components[0])).unwrap();
    
        builder.load(dst_obj_ty, None, pointer, None, None).unwrap()
    };
    
    let res_var = get_res_var(builder, dst_obj_ty);
    builder.store(res_var, dst_obj, None, None).unwrap();
    var_map.insert(dst.0, res_var);
}

fn get_objects(var_map: &mut HashMap<usize, u32>, lhs: &(usize, PrimitiveType), builder: &mut rspirv::dr::Builder, rhs: &(usize, PrimitiveType), res: &(usize, PrimitiveType)) -> (u32, u32, u32) {
    let spv_lhs_id = *var_map.get(&lhs.0).unwrap();
    let lhs_ty = lhs.1.raw_ty(builder);
    let spv_lhs = builder.load(lhs_ty, None, spv_lhs_id, None, None).unwrap();
    let spv_rhs_id = *var_map.get(&rhs.0).unwrap();
    let rhs_ty = rhs.1.raw_ty(builder);
    let spv_rhs = builder.load(rhs_ty, None, spv_rhs_id, None, None).unwrap();
    let res_ty = res.1.raw_ty(builder);
    (spv_lhs, spv_rhs, res_ty)
}

fn get_res_var(builder: &mut rspirv::dr::Builder, res_ty: u32) -> u32 {
    let p_ty = builder.type_pointer(
        None,
        rspirv::spirv::StorageClass::Function,
        res_ty,
    );
    let res_var = builder.variable(
        p_ty,
        None,
        rspirv::spirv::StorageClass::Function,
        None,
    );
    res_var
}

fn process_add(var_map: &mut HashMap<usize, u32>, lhs: &(usize, PrimitiveType), builder: &mut rspirv::dr::Builder, rhs: &(usize, PrimitiveType), res: &(usize, PrimitiveType)) {
    let (spv_lhs, spv_rhs, res_ty) = get_objects(var_map, lhs, builder, rhs, res);
    let f = if lhs.1.is_float() || lhs.1.is_double() {
        rspirv::dr::Builder::f_add
    } else if lhs.1.is_int() || lhs.1.is_uint() {
        rspirv::dr::Builder::i_add
    } else {
        unreachable!()
    };
    let spv_res = f(builder, res_ty, None, spv_lhs, spv_rhs).unwrap();
    let res_var = get_res_var(builder, res_ty);
    builder.store(res_var, spv_res, None, None).unwrap();
    var_map.insert(res.0, res_var);
}

fn process_sub(var_map: &mut HashMap<usize, u32>, lhs: &(usize, PrimitiveType), builder: &mut rspirv::dr::Builder, rhs: &(usize, PrimitiveType), res: &(usize, PrimitiveType)) {
    let (spv_lhs, spv_rhs, res_ty) = get_objects(var_map, lhs, builder, rhs, res);
    let f = if lhs.1.is_float() || lhs.1.is_double() {
        rspirv::dr::Builder::f_sub
    } else if lhs.1.is_int() || lhs.1.is_uint() {
        rspirv::dr::Builder::i_sub
    } else {
        unreachable!()
    };
    let spv_res = f(builder, res_ty, None, spv_lhs, spv_rhs).unwrap();
    let res_var = get_res_var(builder, res_ty);
    builder.store(res_var, spv_res, None, None).unwrap();
    var_map.insert(res.0, res_var);
}

fn get_mul_fn_p(lhs: &(usize, PrimitiveType), rhs: &(usize, PrimitiveType), spv_lhs: &mut u32, spv_rhs: &mut u32) -> fn(&mut rspirv::dr::Builder, u32, Option<u32>, u32, u32) -> Result<u32, rspirv::dr::Error> {
    let f = if lhs.1.is_matrix() {
        if rhs.1.is_matrix() {
            rspirv::dr::Builder::matrix_times_matrix
        } else if rhs.1.is_vector() {
            rspirv::dr::Builder::matrix_times_vector
        } else if rhs.1.is_scalar() {
            rspirv::dr::Builder::matrix_times_scalar
        } else {
            unreachable!();
        }
    } else if lhs.1.is_vector() {
        if rhs.1.is_vector() {
            rspirv::dr::Builder::f_mul
        } else if rhs.1.is_scalar() {
            rspirv::dr::Builder::vector_times_scalar
        } else {
            unreachable!()
        }
    } else if lhs.1.is_scalar() {
        if rhs.1.is_matrix() {
            std::mem::swap(spv_lhs, spv_rhs);
            rspirv::dr::Builder::matrix_times_scalar
        } else if rhs.1.is_vector() {
            std::mem::swap(spv_lhs, spv_rhs);
            rspirv::dr::Builder::vector_times_scalar
        } else if rhs.1.is_scalar() {
            if rhs.1.is_float() || rhs.1.is_double() {
                rspirv::dr::Builder::f_mul
            } else if rhs.1.is_int() || rhs.1.is_uint() {
                rspirv::dr::Builder::i_mul
            } else {
                unreachable!();
            }
        } else {
            unreachable!();
        }
    } else {
        unreachable!();
    };
    f
}

fn process_mul(var_map: &mut HashMap<usize, u32>, lhs: &(usize, PrimitiveType), builder: &mut rspirv::dr::Builder, rhs: &(usize, PrimitiveType), res: &(usize, PrimitiveType)) {
    let (mut spv_lhs, mut spv_rhs, res_ty) = get_objects(var_map, lhs, builder, rhs, res);
    let f = get_mul_fn_p(lhs, rhs, &mut spv_lhs, &mut spv_rhs);
    let spv_res = f(builder, res_ty, None, spv_lhs, spv_rhs).unwrap();
    let res_var = get_res_var(builder, res_ty);
    builder.store(res_var, spv_res, None, None).unwrap();
    var_map.insert(res.0, res_var);
}

fn process_div(var_map: &mut HashMap<usize, u32>, lhs: &(usize, PrimitiveType), builder: &mut rspirv::dr::Builder, rhs: &(usize, PrimitiveType), res: &(usize, PrimitiveType)) {
    let (spv_lhs, spv_rhs, res_ty) = get_objects(var_map, lhs, builder, rhs, res);
    let f = if lhs.1.is_float() || lhs.1.is_double() {
        rspirv::dr::Builder::f_div
    } else if lhs.1.is_int() {
        rspirv::dr::Builder::s_div
    } else if lhs.1.is_uint() {
        rspirv::dr::Builder::u_div
    } else {
        unreachable!();
    };

    let spv_res = f(builder, res_ty, None, spv_lhs, spv_rhs).unwrap();
    let res_var = get_res_var(builder, res_ty);
    builder.store(res_var, spv_res, None, None).unwrap();
    var_map.insert(res.0, res_var);
}

fn process_bit_op(var_map: &mut HashMap<usize, u32>, lhs: &(usize, PrimitiveType), builder: &mut rspirv::dr::Builder, rhs: &(usize, PrimitiveType), res: &(usize, PrimitiveType), f: fn(&mut rspirv::dr::Builder, u32, Option<u32>, u32, u32) -> Result<u32, rspirv::dr::Error>) {
    let (spv_lhs, spv_rhs, res_ty) = get_objects(var_map, lhs, builder, rhs, res);
    let spv_res = f(builder, res_ty, None, spv_lhs, spv_rhs).unwrap();

    let p_ty = builder.type_pointer(
        None,
        rspirv::spirv::StorageClass::Function,
        res_ty,
    );
    let res_var = builder.variable(
        p_ty,
        None,
        rspirv::spirv::StorageClass::Function,
        None,
    );
    builder.store(res_var, spv_res, None, None).unwrap();
    var_map.insert(res.0, res_var);
}

fn get_object_assign(var_map: &mut HashMap<usize, u32>, lhs: &(usize, PrimitiveType), builder: &mut rspirv::dr::Builder, rhs: &(usize, PrimitiveType)) -> (u32, u32, u32, u32) {
    let spv_lhs_id = *var_map.get(&lhs.0).unwrap();
    let lhs_ty = lhs.1.raw_ty(builder);
    let spv_lhs = builder.load(lhs_ty, None, spv_lhs_id, None, None).unwrap();
    let spv_rhs_id = *var_map.get(&rhs.0).unwrap();
    let rhs_ty = rhs.1.raw_ty(builder);
    let spv_rhs = builder.load(rhs_ty, None, spv_rhs_id, None, None).unwrap();
    (spv_lhs_id, lhs_ty, spv_lhs, spv_rhs)
}

fn process_add_assign(var_map: &mut HashMap<usize, u32>, lhs: &(usize, PrimitiveType), builder: &mut rspirv::dr::Builder, rhs: &(usize, PrimitiveType)) {
    let (spv_lhs_id, lhs_ty, spv_lhs, spv_rhs) = get_object_assign(var_map, lhs, builder, rhs);
    
    let f = if lhs.1.is_float() || lhs.1.is_double() {
        rspirv::dr::Builder::f_add
    } else if lhs.1.is_int() || lhs.1.is_uint() {
        rspirv::dr::Builder::i_add
    } else {
        unreachable!()
    };
    let spv_res = f(builder, lhs_ty, None, spv_lhs, spv_rhs).unwrap();
    builder.store(spv_lhs_id, spv_res, None, None).unwrap();
}

fn process_sub_assign(var_map: &mut HashMap<usize, u32>, lhs: &(usize, PrimitiveType), builder: &mut rspirv::dr::Builder, rhs: &(usize, PrimitiveType)) {
    let (spv_lhs_id, lhs_ty, spv_lhs, spv_rhs) = get_object_assign(var_map, lhs, builder, rhs);
    
    let f = if lhs.1.is_float() || lhs.1.is_double() {
        rspirv::dr::Builder::f_sub
    } else if lhs.1.is_int() || lhs.1.is_uint() {
        rspirv::dr::Builder::i_sub
    } else {
        unreachable!()
    };
    let spv_res = f(builder, lhs_ty, None, spv_lhs, spv_rhs).unwrap();
    builder.store(spv_lhs_id, spv_res, None, None).unwrap();
}

fn process_mul_assign(var_map: &mut HashMap<usize, u32>, lhs: &(usize, PrimitiveType), builder: &mut rspirv::dr::Builder, rhs: &(usize, PrimitiveType)) {
    let (spv_lhs_id, lhs_ty, mut spv_lhs, mut spv_rhs) = get_object_assign(var_map, lhs, builder, rhs);
    let f = get_mul_fn_p(lhs, rhs, &mut spv_lhs, &mut spv_rhs);
    let spv_res = f(builder, lhs_ty, None, spv_lhs, spv_rhs).unwrap();
    builder.store(spv_lhs_id, spv_res, None, None).unwrap();
}

fn process_div_assign(var_map: &mut HashMap<usize, u32>, lhs: &(usize, PrimitiveType), builder: &mut rspirv::dr::Builder, rhs: &(usize, PrimitiveType)) {
    let (spv_lhs_id, lhs_ty, spv_lhs, spv_rhs) = get_object_assign(var_map, lhs, builder, rhs);
    let f = if lhs.1.is_float() || lhs.1.is_double() {
        rspirv::dr::Builder::f_div
    } else if lhs.1.is_int() {
        rspirv::dr::Builder::s_div
    } else if lhs.1.is_uint() {
        rspirv::dr::Builder::u_div
    } else {
        unreachable!();
    };

    let spv_res = f(builder, lhs_ty, None, spv_lhs, spv_rhs).unwrap();
    builder.store(spv_lhs_id, spv_res, None, None).unwrap();
}

fn process_bit_op_assign(var_map: &mut HashMap<usize, u32>, lhs: &(usize, PrimitiveType), builder: &mut rspirv::dr::Builder, rhs: &(usize, PrimitiveType), f: fn(&mut rspirv::dr::Builder, u32, Option<u32>, u32, u32) -> Result<u32, rspirv::dr::Error>) {
    let (spv_lhs_id, lhs_ty, spv_lhs, spv_rhs) = get_object_assign(var_map, lhs, builder, rhs);
    let spv_res = f(builder, lhs_ty, None, spv_lhs, spv_rhs).unwrap();

    builder.store(spv_lhs_id, spv_res, None, None).unwrap();
}