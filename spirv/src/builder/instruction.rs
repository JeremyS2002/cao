use std::any::TypeId;
use std::collections::HashMap;
use std::collections::VecDeque;

use crate::data::DataType;
use crate::data::{PrimitiveType, PrimitiveVal};

#[derive(Clone, Debug)]
pub enum Instruction {
    Store {
        /// Declare a constant with this value
        val: PrimitiveVal,
        /// Store into variable with this id
        store: usize,
    },
    Add {
        lhs: (usize, PrimitiveType),
        rhs: (usize, PrimitiveType),
        res: (usize, PrimitiveType),
    },
    Sub {
        lhs: (usize, PrimitiveType),
        rhs: (usize, PrimitiveType),
        res: (usize, PrimitiveType),
    },
    Mul {
        lhs: (usize, PrimitiveType),
        rhs: (usize, PrimitiveType),
        res: (usize, PrimitiveType),
    },
    Div {
        lhs: (usize, PrimitiveType),
        rhs: (usize, PrimitiveType),
        res: (usize, PrimitiveType),
    },
    AddAssign {
        lhs: (usize, PrimitiveType),
        rhs: (usize, PrimitiveType),
    },
    SubAssign {
        lhs: (usize, PrimitiveType),
        rhs: (usize, PrimitiveType),
    },
    MulAssign {
        lhs: (usize, PrimitiveType),
        rhs: (usize, PrimitiveType),
    },
    DivAssign {
        lhs: (usize, PrimitiveType),
        rhs: (usize, PrimitiveType),
    },
    BitAnd {
        lhs: (usize, PrimitiveType),
        rhs: (usize, PrimitiveType),
        res: (usize, PrimitiveType),
    },
    BitOr {
        lhs: (usize, PrimitiveType),
        rhs: (usize, PrimitiveType),
        res: (usize, PrimitiveType),
    },
    BitXor {
        lhs: (usize, PrimitiveType),
        rhs: (usize, PrimitiveType),
        res: (usize, PrimitiveType),
    },
    BitAndAssign {
        lhs: (usize, PrimitiveType),
        rhs: (usize, PrimitiveType),
    },
    BitOrAssign {
        lhs: (usize, PrimitiveType),
        rhs: (usize, PrimitiveType),
    },
    BitXorAssign {
        lhs: (usize, PrimitiveType),
        rhs: (usize, PrimitiveType),
    },
    LogicalAnd {
        lhs: usize,
        rhs: usize,
        res: usize,
    },
    LogicalOr {
        lhs: usize,
        rhs: usize,
        res: usize,
    },
    LogicalEqual {
        lhs: usize,
        rhs: usize,
        res: usize,
    },
    LogicalNot {
        lhs: usize,
        res: usize,
    },
    VectorShuffle {
        src: (usize, PrimitiveType),
        dst: (usize, PrimitiveType),
        components: [u32; 4],
    },
    VectorComposite {
        components: [usize; 4],
        ty: PrimitiveType,
        store: usize,
    },
    IfChain {
        conditions: VecDeque<usize>,
        instructions: VecDeque<Vec<Instruction>>,
        else_instructions: Option<Vec<Instruction>>,
    },
    Loop {
        condition: usize,
        body: Vec<Instruction>,
    },
    LoadIn {
        index: usize,
        ty: PrimitiveType,
        store: usize,
    },
    StoreOut {
        index: usize,
        ty: PrimitiveType,
        read: usize,
    },
    LoadUniform {
        index: usize,
        ty: DataType,
        store: usize,
    },
    LoadUniformField {
        u_index: usize,
        f_index: usize,
        store: usize,
        ty: DataType,
        f_ty: DataType,
    },
    LoadStorage {},
    StoreStorage {},
    Break,
    Continue,
    FnCall {
        fn_id: usize,
        store_id: usize,
        arguments: Vec<(usize, DataType)>,
    },
    Return {
        id: usize,
    },
    NewArray {
        store: usize,
        ty: PrimitiveType,
        data: Vec<usize>,
    },
    ArrayStore {
        array: usize,
        index: usize,
        data: usize,
        element_ty: PrimitiveType,
    },
    ArrayLoad {
        array: usize,
        index: usize,
        store: usize,
        element_ty: PrimitiveType,
    },
    NewStruct {
        store: usize,
        ty: DataType,
        data: Vec<usize>,
    },
    StructStore {
        struct_id: usize,
        field: usize,
        ty: DataType,
        data: usize,
    },
    StructLoad {
        struct_id: usize,
        field: usize,
        ty: DataType,
        store: usize,
    },
    SampleTexture {
        texture: usize,
        sampler: usize,
        coordinate: usize,
        coordinate_ty: PrimitiveType,
        res_ty: PrimitiveType,
        store: usize,
    }
}

impl Instruction {
    pub fn process(
        &mut self,
        builder: &mut rspirv::dr::Builder,
        var_map: &mut HashMap<usize, u32>,
        function_map: &HashMap<usize, usize>,
        struct_map: &mut HashMap<TypeId, u32>,
        uniforms: &[u32],
        storages: &[u32],
        inputs: &[u32],
        outputs: &[u32],
        textures: &[(u32, u32)],
        samplers: &[(u32, u32)],
        break_target: Option<u32>,
        continue_target: Option<u32>,
        var_block: usize,
    ) {
        match self {
            Instruction::Store { val, store } => {
                let v = val.set(builder, var_block);
                var_map.insert(*store, v);
            }
            Instruction::Add { lhs, rhs, res } => process_add(var_map, lhs, builder, rhs, res, var_block),
            Instruction::Sub { lhs, rhs, res } => process_sub(var_map, lhs, builder, rhs, res, var_block),
            Instruction::Mul { lhs, rhs, res } => process_mul(var_map, lhs, builder, rhs, res, var_block),
            Instruction::Div { lhs, rhs, res } => process_div(var_map, lhs, builder, rhs, res, var_block),
            Instruction::AddAssign { lhs, rhs } => process_add_assign(var_map, lhs, builder, rhs),
            Instruction::SubAssign { lhs, rhs } => process_sub_assign(var_map, lhs, builder, rhs),
            Instruction::MulAssign { lhs, rhs } => process_mul_assign(var_map, lhs, builder, rhs),
            Instruction::DivAssign { lhs, rhs } => process_div_assign(var_map, lhs, builder, rhs),
            Instruction::BitAnd { lhs, rhs, res } => process_bit_op(
                var_map,
                lhs,
                builder,
                rhs,
                res,
                rspirv::dr::Builder::bitwise_and,
                var_block,
            ),
            Instruction::BitOr { lhs, rhs, res } => process_bit_op(
                var_map,
                lhs,
                builder,
                rhs,
                res,
                rspirv::dr::Builder::bitwise_or,
                var_block,
            ),
            Instruction::BitXor { lhs, rhs, res } => process_bit_op(
                var_map,
                lhs,
                builder,
                rhs,
                res,
                rspirv::dr::Builder::bitwise_xor,
                var_block,
            ),
            Instruction::BitAndAssign { lhs, rhs } => {
                process_bit_op_assign(var_map, lhs, builder, rhs, rspirv::dr::Builder::bitwise_and)
            }
            Instruction::BitOrAssign { lhs, rhs } => {
                process_bit_op_assign(var_map, lhs, builder, rhs, rspirv::dr::Builder::bitwise_or)
            }
            Instruction::BitXorAssign { lhs, rhs } => {
                process_bit_op_assign(var_map, lhs, builder, rhs, rspirv::dr::Builder::bitwise_xor)
            }
            Instruction::IfChain {
                conditions,
                instructions,
                else_instructions,
            } => process_if_chain(
                conditions,
                instructions,
                else_instructions,
                var_map,
                builder,
                function_map,
                struct_map,
                uniforms,
                storages,
                inputs,
                outputs,
                textures,
                samplers,
                break_target,
                continue_target,
                var_block,
            ),
            Instruction::Loop { condition, body } => process_loop(
                builder,
                var_map,
                condition,
                body,
                function_map,
                struct_map,
                uniforms,
                storages,
                inputs,
                outputs,
                textures,
                samplers,
                var_block,
            ),
            Instruction::Break => builder.branch(break_target.unwrap()).unwrap(),
            Instruction::Continue => builder.branch(continue_target.unwrap()).unwrap(),
            Instruction::FnCall {
                ..
            } => {}
            Instruction::Return { .. } => {}
            Instruction::LoadIn { index, ty, store } => {
                let spv_ty = ty.base_type(builder);
                let input_var = *inputs.get(*index).unwrap();
                let spv_obj = builder.load(spv_ty, None, input_var, None, None).unwrap();

                let spv_var = if let Some(&spv_var) = var_map.get(store) {
                    spv_var
                } else {
                    ty.variable(builder, var_block)
                };

                builder.store(spv_var, spv_obj, None, None).unwrap();
                var_map.insert(*store, spv_var);
            }
            Instruction::StoreOut { index, ty, read } => {
                let ty = ty.base_type(builder);
                let spv_var = *var_map.get(read).unwrap();
                let spv_obj = builder.load(ty, None, spv_var, None, None).unwrap();

                let output_var = *outputs.get(*index).unwrap();

                builder.store(output_var, spv_obj, None, None).unwrap();
            }
            Instruction::LoadStorage {} => todo!(),
            Instruction::StoreStorage {} => todo!(),
            Instruction::VectorShuffle {
                src,
                dst,
                components,
            } => process_vector_shuffle(var_map, builder, src, dst, *components, var_block),
            Instruction::VectorComposite {
                components,
                ty,
                store,
            } => process_vector_composite(var_map, builder, components, ty, store, var_block),
            Instruction::NewArray { store, ty, data } => {                
                let spv_ty = DataType::Array(*ty, data.len()).base_type(builder, struct_map);
                let var = DataType::Array(*ty, data.len()).variable(builder, struct_map, var_block);

                let elements = data
                    .iter()
                    .map(|e| *var_map.get(e).unwrap())
                    .collect::<Vec<_>>();

                let spv_obj = builder.constant_composite(spv_ty, elements);
                builder.store(var, spv_obj, None, None).unwrap();
                var_map.insert(*store, var);
            }
            Instruction::ArrayStore {
                array,
                index,
                data,
                element_ty,
            } => {
                let index_obj = PrimitiveVal::UInt(*index as u32).set_constant(builder).0;
                let array_obj = *var_map.get(array).unwrap();
                let spv_element_ty = element_ty.base_type(builder);
                let index_p = builder
                    .access_chain(spv_element_ty, None, array_obj, [index_obj])
                    .unwrap();

                let data_var = *var_map.get(data).unwrap();
                let data_obj = builder
                    .load(spv_element_ty, None, data_var, None, None)
                    .unwrap();

                builder.store(index_p, data_obj, None, None).unwrap();
            }
            Instruction::ArrayLoad {
                array,
                index,
                store,
                element_ty,
            } => {
                let index_obj = PrimitiveVal::UInt(*index as u32).set_constant(builder).0;
                let array_obj = *var_map.get(array).unwrap();
                let spv_element_ty = element_ty.base_type(builder);
                let index_p = builder
                    .access_chain(spv_element_ty, None, array_obj, [index_obj])
                    .unwrap();

                let index_obj = builder
                    .load(spv_element_ty, None, index_p, None, None)
                    .unwrap();

                let store_var = element_ty.variable(builder, var_block);
                builder.store(store_var, index_obj, None, None).unwrap();
                var_map.insert(*store, store_var);
            }
            Instruction::LogicalAnd { lhs, rhs, res } => process_logical(
                builder,
                var_map,
                lhs,
                rhs,
                res,
                var_block,
                rspirv::dr::Builder::logical_and,
            ),
            Instruction::LogicalOr { lhs, rhs, res } => process_logical(
                builder,
                var_map,
                lhs,
                rhs,
                res,
                var_block,
                rspirv::dr::Builder::logical_or,
            ),
            Instruction::LogicalEqual { lhs, rhs, res } => process_logical(
                builder,
                var_map,
                lhs,
                rhs,
                res,
                var_block,
                rspirv::dr::Builder::logical_equal,
            ),
            Instruction::LogicalNot { lhs, res } => {
                let ty = PrimitiveType::Bool.base_type(builder);
                let lhs_pointer = *var_map.get(lhs).unwrap();
                let lhs_obj = builder.load(ty, None, lhs_pointer, None, None).unwrap();
                let res_obj = builder.logical_not(ty, None, lhs_obj).unwrap();
                let res_variable = PrimitiveType::Bool.variable(builder, var_block);
                builder.store(res_variable, res_obj, None, None).unwrap();
                var_map.insert(*res, res_variable);
            }
            Instruction::NewStruct { store, ty, data } => {
                let variable = ty.variable(builder, struct_map, var_block);

                let types = if let DataType::Struct(_, _, _, types) = ty {
                    types
                } else {
                    unreachable!();
                };

                for i in 0..data.len() {
                    let f_ty = types.get(i).unwrap();
                    let f_spv_ty = f_ty.base_type(builder, struct_map);
                    let f_spv_p_ty =
                        builder.type_pointer(None, rspirv::spirv::StorageClass::Function, f_spv_ty);
                    let f_obj = *var_map.get(data.get(i).unwrap()).unwrap();

                    let index = PrimitiveVal::UInt(i as u32).set_constant(builder).0;

                    let pointer = builder
                        .access_chain(f_spv_p_ty, None, variable, [index])
                        .unwrap();
                    builder.store(pointer, f_obj, None, None).unwrap();
                }

                var_map.insert(*store, variable);
            }
            Instruction::StructStore {
                struct_id,
                field,
                ty,
                data,
            } => {
                let base_pointer = *var_map.get(struct_id).unwrap();
                let f_spv_ty = ty.base_type(builder, struct_map);
                let f_spv_p_ty =
                    builder.type_pointer(None, rspirv::spirv::StorageClass::Function, f_spv_ty);

                let f_obj = *var_map.get(data).unwrap();

                let index = PrimitiveVal::UInt(*field as u32).set_constant(builder).0;

                let pointer = builder
                    .access_chain(f_spv_p_ty, None, base_pointer, [index])
                    .unwrap();
                builder.store(pointer, f_obj, None, None).unwrap();
            }
            Instruction::StructLoad {
                struct_id,
                field,
                ty,
                store,
            } => {
                let base_pointer = *var_map.get(struct_id).unwrap();
                let f_spv_ty = ty.base_type(builder, struct_map);
                let f_spv_p_ty =
                    builder.type_pointer(None, rspirv::spirv::StorageClass::Function, f_spv_ty);

                let index_obj = PrimitiveVal::UInt(*field as u32).set_constant(builder).0;

                let pointer = builder
                    .access_chain(f_spv_p_ty, None, base_pointer, Some(index_obj))
                    .unwrap();
                let res_spv_obj = builder.load(f_spv_ty, None, pointer, None, None).unwrap();

                let variable = ty.variable(builder, struct_map, var_block);
                builder.store(variable, res_spv_obj, None, None).unwrap();

                var_map.insert(*store, variable);
            }
            Instruction::LoadUniform { 
                index, 
                ty, 
                store 
            } => {
                // The following is based on how shaderc compiles glsl
                // TODO afaik there is no reason not to use copy_memory instead
                // let pointer_ty = builder.type_pointer(None, rspirv::spirv::StorageClass::Function, base_ty);
                // let res_var = builder.variable(pointer_ty, None, rspirv::spirv::StorageClass::Function, None, var_block);
                // builder.copy_memory(res_var, variable, None, None, None).unwrap();
                // var_map.insert(*store, res_var);
                let variable = *uniforms.get(*index).unwrap();
                let base_ty = ty.base_type(builder, struct_map);
                let base_p_ty = builder.type_pointer(None, rspirv::spirv::StorageClass::Uniform, base_ty);
                let index_obj = PrimitiveVal::UInt(0).set_constant(builder).0;
                let field_pointer = builder.access_chain(base_p_ty, None, variable, Some(index_obj)).unwrap();

                let obj = builder.load(base_ty, None, field_pointer, None, None).unwrap();

                let res_var = ty.variable(builder, struct_map, var_block);

                copy_composite(ty, builder, res_var, obj, struct_map);

                var_map.insert(*store, res_var);
            },
            Instruction::LoadUniformField { 
                u_index, 
                f_index, 
                store,
                ty, 
                f_ty,
            } => {
                let uniform_variable = *uniforms.get(*u_index).unwrap();
                let struct_index = PrimitiveVal::UInt(0).set_constant(builder).0;
                let struct_ty = ty.base_type(builder, struct_map);
                let struct_p_ty = builder.type_pointer(None, rspirv::spirv::StorageClass::Uniform, struct_ty);
                let struct_p = builder.access_chain(struct_p_ty, None, uniform_variable, Some(struct_index)).unwrap();

                let index_obj = PrimitiveVal::UInt(*f_index as u32).set_constant(builder).0;
                let field_ty = f_ty.base_type(builder, struct_map);
                let field_p_ty = builder.type_pointer(None, rspirv::spirv::StorageClass::Uniform, field_ty);
                let pointer = builder.access_chain(field_p_ty, None, struct_p, Some(index_obj)).unwrap();

                let res_var = f_ty.variable(builder, struct_map, var_block);

                let field_obj = builder.load(field_ty, None, pointer, None, None).unwrap();
                builder.store(res_var, field_obj, None, None).unwrap();
                var_map.insert(*store, res_var);
            },
            Instruction::SampleTexture { 
                texture, 
                sampler, 
                coordinate,
                coordinate_ty,
                res_ty,
                store,
            } => {
                let (spv_texture, spv_texture_ty) = *textures.get(*texture).unwrap();
                let (spv_sampler, spv_sampler_ty) = *samplers.get(*sampler).unwrap();

                let spv_tex_obj = builder.load(spv_texture_ty, None, spv_texture, None, None).unwrap();
                let spv_sam_obj = builder.load(spv_sampler_ty, None, spv_sampler, None, None).unwrap();

                let sampled_image_ty = builder.type_sampled_image(spv_texture_ty);

                let sampled_image = builder.sampled_image(sampled_image_ty, None, spv_tex_obj, spv_sam_obj).unwrap();
            
                let res_spv_ty = res_ty.base_type(builder);

                let spv_coord_ty = coordinate_ty.base_type(builder);
                let spv_coord_var = *var_map.get(coordinate).unwrap();
                let spv_coord_obj = builder.load(spv_coord_ty, None, spv_coord_var, None, None).unwrap();

                let res_obj = builder.image_sample_implicit_lod(
                    res_spv_ty, 
                    None, 
                    sampled_image, 
                    spv_coord_obj, 
                    None, 
                    None
                ).unwrap();

                let res_var = res_ty.variable(builder, var_block);

                builder.store(res_var, res_obj, None, None).unwrap();

                var_map.insert(*store, res_var);
            },
        }
    }
}

fn copy_composite(ty: &DataType, builder: &mut rspirv::dr::Builder, res_var: u32, obj: u32, struct_map: &mut HashMap<TypeId, u32>) {
    match ty {
        DataType::Primitive(_) => {
            builder.store(res_var, obj, None, None).unwrap();
        },
        DataType::Array(p, n) => {
            let index_type = p.base_type(builder);
            let index_p_type = p.pointer_type(builder);
            for index in 0..*n {
                let index_obj = PrimitiveVal::UInt(index as u32).set_constant(builder).0;
                let index_data_obj = builder.composite_extract(index_type, None, obj, Some(index_obj)).unwrap();
                let pointer = builder.access_chain(index_p_type, None, res_var, Some(index_obj)).unwrap();
                builder.store(pointer, index_data_obj, None, None).unwrap();
            }
        },
        DataType::Struct(_, _, _, types) => {
            let mut index = 0;
            for field_type in *types {
                let raw_field_type = field_type.base_type(builder, struct_map);
                let raw_field_p_type = field_type.pointer_type(builder, struct_map);//builder.type_pointer(None, rspirv::spirv::StorageClass::Uniform, raw_field_type);
                let field_obj = builder.composite_extract(raw_field_type, None, obj, Some(index)).unwrap();
                let index_obj = PrimitiveVal::UInt(index).set_constant(builder).0;

                let pointer = builder.access_chain(raw_field_p_type, None, res_var, Some(index_obj)).unwrap();
        
                // builder.store(pointer, field_obj, None, None).unwrap();

                copy_composite(field_type, builder, pointer, field_obj, struct_map);

                index += 1;
            }
        }
    }
}

fn process_vector_composite(var_map: &mut HashMap<usize, u32>, builder: &mut rspirv::dr::Builder, components: &[usize; 4], ty: &PrimitiveType, store: &usize, var_block: usize) {
    let n = ty.components();
    let spv_ty = ty.base_type(builder);
    let spv_components = components.iter().take(n as _).map(|i| {
        let comp_spv_ty = ty.component().unwrap().base_type(builder);
        let var = *var_map.get(i).unwrap();
        builder.load(comp_spv_ty, None, var, None, None).unwrap()
    }).collect::<Vec<_>>();
    let res_obj = builder.composite_construct(spv_ty, None, spv_components).unwrap();
    let variable = ty.variable(builder, var_block);
    builder.store(variable, res_obj, None, None).unwrap();
    var_map.insert(*store, variable);
}

fn process_logical(
    builder: &mut rspirv::dr::Builder,
    var_map: &mut HashMap<usize, u32>,
    lhs: &usize,
    rhs: &usize,
    res: &usize,
    var_block: usize,
    f: fn(&mut rspirv::dr::Builder, u32, Option<u32>, u32, u32) -> Result<u32, rspirv::dr::Error>,
) {
    let ty = PrimitiveType::Bool.base_type(builder);
    let lhs_pointer = *var_map.get(lhs).unwrap();
    let lhs_obj = builder.load(ty, None, lhs_pointer, None, None).unwrap();
    let rhs_pointer = *var_map.get(rhs).unwrap();
    let rhs_obj = builder.load(ty, None, rhs_pointer, None, None).unwrap();
    let res_obj = f(builder, ty, None, lhs_obj, rhs_obj).unwrap();
    let res_variable = PrimitiveType::Bool.variable(builder, var_block);

    builder.store(res_variable, res_obj, None, None).unwrap();
    var_map.insert(*res, res_variable);
}

fn process_loop(
    builder: &mut rspirv::dr::Builder,
    var_map: &mut HashMap<usize, u32>,
    condition: &mut usize,
    body: &mut Vec<Instruction>,
    function_map: &HashMap<usize, usize>,
    struct_map: &mut HashMap<TypeId, u32>,
    uniform_map: &[u32],
    storage_map: &[u32],
    inputs: &[u32],
    outputs: &[u32],
    textures: &[(u32, u32)],
    samplers: &[(u32, u32)],
    var_block: usize,
) {
    let start = builder.id();
    builder.branch(start).unwrap();
    builder.begin_block(Some(start)).unwrap();
    let merge_block = builder.id();
    let condition_block = builder.id();
    let continue_target = builder.id();
    let block = builder.selected_block().unwrap();
    builder
        .loop_merge(
            merge_block,
            continue_target,
            rspirv::spirv::LoopControl::NONE,
            None,
        )
        .unwrap();
    builder.select_block(Some(block)).unwrap();
    builder.branch(condition_block).unwrap();
    builder.begin_block(Some(condition_block)).unwrap();
    let condition_var = *var_map.get(condition).unwrap();
    let condition_type = builder.type_bool();
    let condition_obj = builder
        .load(condition_type, None, condition_var, None, None)
        .unwrap();
    let body_block = builder.id();
    builder
        .branch_conditional(condition_obj, body_block, merge_block, None)
        .unwrap();
    builder.begin_block(Some(body_block)).unwrap();
    for instruction in body {
        instruction.process(
            builder,
            var_map,
            function_map,
            struct_map,
            uniform_map,
            storage_map,
            inputs,
            outputs,
            textures,
            samplers,
            Some(merge_block),
            Some(continue_target),
            var_block,
        );
    }
    builder.branch(continue_target).unwrap();
    builder.begin_block(Some(continue_target)).unwrap();
    builder.branch(start).unwrap();
    builder.begin_block(Some(merge_block)).unwrap();
}

fn process_if_chain(
    conditions: &mut VecDeque<usize>,
    instructions: &mut VecDeque<Vec<Instruction>>,
    else_instructions: &mut Option<Vec<Instruction>>,
    var_map: &mut HashMap<usize, u32>,
    builder: &mut rspirv::dr::Builder,
    function_map: &HashMap<usize, usize>,
    struct_map: &mut HashMap<TypeId, u32>,
    uniforms: &[u32],
    storages: &[u32],
    inputs: &[u32],
    outputs: &[u32],
    textures: &[(u32, u32)],
    samplers: &[(u32, u32)],
    break_target: Option<u32>,
    continue_target: Option<u32>,
    var_block: usize,
) {
    if conditions.len() == 0 {
        if let Some(else_instructions) = else_instructions {
            for instruction in else_instructions {
                instruction.process(
                    builder,
                    var_map,
                    function_map,
                    struct_map,
                    uniforms,
                    storages,
                    inputs,
                    outputs,
                    textures,
                    samplers,
                    break_target,
                    continue_target,
                    var_block,
                );
            }
        }
        return;
    }

    let condition = conditions.pop_front().unwrap();

    let condition_var = *var_map.get(&condition).unwrap();
    let condition_type = builder.type_bool();
    let condition_obj = builder
        .load(condition_type, None, condition_var, None, None)
        .unwrap();

    let true_label = builder.id();
    let false_label = builder.id();
    let end_label = builder.id();

    let block = builder.selected_block().unwrap();
    builder
        .selection_merge(end_label, rspirv::spirv::SelectionControl::NONE)
        .unwrap();
    builder.select_block(Some(block)).unwrap();
    builder
        .branch_conditional(condition_obj, true_label, false_label, None)
        .unwrap();

    builder.begin_block(Some(true_label)).unwrap();

    for mut instruction in instructions.pop_front().unwrap() {
        instruction.process(
            builder,
            var_map,
            function_map,
            struct_map,
            uniforms,
            storages,
            inputs,
            outputs,
            textures,
            samplers,
            break_target,
            continue_target,
            var_block,
        );
    }

    builder.branch(end_label).unwrap();

    builder.begin_block(Some(false_label)).unwrap();

    process_if_chain(
        conditions,
        instructions,
        else_instructions,
        var_map,
        builder,
        function_map,
        struct_map,
        uniforms,
        storages,
        inputs,
        outputs,
        textures,
        samplers,
        break_target,
        continue_target,
        var_block,
    );

    builder.branch(end_label).unwrap();
    builder.begin_block(Some(end_label)).unwrap();
}

fn process_vector_shuffle(
    var_map: &mut HashMap<usize, u32>,
    builder: &mut rspirv::dr::Builder,
    src: &(usize, PrimitiveType),
    dst: &(usize, PrimitiveType),
    components: [u32; 4],
    var_block: usize,
) {
    let src_var = *var_map.get(&src.0).unwrap();
    let src_obj_ty = src.1.base_type(builder);
    let dst_obj_ty = dst.1.base_type(builder);

    let dst_obj = if dst.1.is_vector() {
        let src_obj = builder.load(src_obj_ty, None, src_var, None, None).unwrap();

        let component_count = dst.1.components();
        let components = components
            .iter()
            .take(component_count as _)
            .map(|i| {
                //PrimitiveVal::UInt(*i).set_constant(builder).0
                *i
            })
            .collect::<Vec<_>>();
        builder
            .vector_shuffle(
                dst_obj_ty,
                None,
                src_obj,
                src_obj,
                components,
            )
            .unwrap()
    } else {
        let pointer_ty =
            builder.type_pointer(None, rspirv::spirv::StorageClass::Function, dst_obj_ty);
        let index_obj = PrimitiveVal::UInt(components[0]).set_constant(builder).0;
        let pointer = builder
            .access_chain(pointer_ty, None, src_var, Some(index_obj))
            .unwrap();

        builder.load(dst_obj_ty, None, pointer, None, None).unwrap()
    };

    let res_var = dst.1.variable(builder, var_block);
    builder.store(res_var, dst_obj, None, None).unwrap();
    var_map.insert(dst.0, res_var);
}

fn get_objects(
    var_map: &mut HashMap<usize, u32>,
    lhs: &(usize, PrimitiveType),
    builder: &mut rspirv::dr::Builder,
    rhs: &(usize, PrimitiveType),
    res: &(usize, PrimitiveType),
) -> (u32, u32, u32) {
    let spv_lhs_id = *var_map.get(&lhs.0).unwrap();
    let lhs_ty = lhs.1.base_type(builder);
    let spv_lhs = builder.load(lhs_ty, None, spv_lhs_id, None, None).unwrap();
    let spv_rhs_id = *var_map.get(&rhs.0).unwrap();
    let rhs_ty = rhs.1.base_type(builder);
    let spv_rhs = builder.load(rhs_ty, None, spv_rhs_id, None, None).unwrap();
    let res_ty = res.1.base_type(builder);
    (spv_lhs, spv_rhs, res_ty)
}

fn process_add(
    var_map: &mut HashMap<usize, u32>,
    lhs: &(usize, PrimitiveType),
    builder: &mut rspirv::dr::Builder,
    rhs: &(usize, PrimitiveType),
    res: &(usize, PrimitiveType),
    var_block: usize,
) {
    let (spv_lhs, spv_rhs, res_ty) = get_objects(var_map, lhs, builder, rhs, res);
    let f = if lhs.1.is_float() || lhs.1.is_double() {
        rspirv::dr::Builder::f_add
    } else if lhs.1.is_int() || lhs.1.is_uint() {
        rspirv::dr::Builder::i_add
    } else {
        unreachable!()
    };
    let spv_res = f(builder, res_ty, None, spv_lhs, spv_rhs).unwrap();
    let res_var = res.1.variable(builder, var_block);
    builder.store(res_var, spv_res, None, None).unwrap();
    var_map.insert(res.0, res_var);
}

fn process_sub(
    var_map: &mut HashMap<usize, u32>,
    lhs: &(usize, PrimitiveType),
    builder: &mut rspirv::dr::Builder,
    rhs: &(usize, PrimitiveType),
    res: &(usize, PrimitiveType),
    var_block: usize,
) {
    let (spv_lhs, spv_rhs, res_ty) = get_objects(var_map, lhs, builder, rhs, res);
    let f = if lhs.1.is_float() || lhs.1.is_double() {
        rspirv::dr::Builder::f_sub
    } else if lhs.1.is_int() || lhs.1.is_uint() {
        rspirv::dr::Builder::i_sub
    } else {
        unreachable!()
    };
    let spv_res = f(builder, res_ty, None, spv_lhs, spv_rhs).unwrap();
    let res_var = res.1.variable(builder, var_block);
    builder.store(res_var, spv_res, None, None).unwrap();
    var_map.insert(res.0, res_var);
}

fn get_mul_fn_p(
    lhs: &(usize, PrimitiveType),
    rhs: &(usize, PrimitiveType),
    spv_lhs: &mut u32,
    spv_rhs: &mut u32,
) -> fn(&mut rspirv::dr::Builder, u32, Option<u32>, u32, u32) -> Result<u32, rspirv::dr::Error> {
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

fn process_mul(
    var_map: &mut HashMap<usize, u32>,
    lhs: &(usize, PrimitiveType),
    builder: &mut rspirv::dr::Builder,
    rhs: &(usize, PrimitiveType),
    res: &(usize, PrimitiveType),
    var_block: usize,
) {
    let (mut spv_lhs, mut spv_rhs, res_ty) = get_objects(var_map, lhs, builder, rhs, res);
    let f = get_mul_fn_p(lhs, rhs, &mut spv_lhs, &mut spv_rhs);
    let spv_res = f(builder, res_ty, None, spv_lhs, spv_rhs).unwrap();
    let res_var = res.1.variable(builder, var_block);
    builder.store(res_var, spv_res, None, None).unwrap();
    var_map.insert(res.0, res_var);
}

fn process_div(
    var_map: &mut HashMap<usize, u32>,
    lhs: &(usize, PrimitiveType),
    builder: &mut rspirv::dr::Builder,
    rhs: &(usize, PrimitiveType),
    res: &(usize, PrimitiveType),
    var_block: usize,
) {
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
    let res_var = res.1.variable(builder, var_block);
    builder.store(res_var, spv_res, None, None).unwrap();
    var_map.insert(res.0, res_var);
}

fn process_bit_op(
    var_map: &mut HashMap<usize, u32>,
    lhs: &(usize, PrimitiveType),
    builder: &mut rspirv::dr::Builder,
    rhs: &(usize, PrimitiveType),
    res: &(usize, PrimitiveType),
    f: fn(&mut rspirv::dr::Builder, u32, Option<u32>, u32, u32) -> Result<u32, rspirv::dr::Error>,
    var_block: usize,
) {
    let (spv_lhs, spv_rhs, res_ty) = get_objects(var_map, lhs, builder, rhs, res);
    let spv_res = f(builder, res_ty, None, spv_lhs, spv_rhs).unwrap();

    let res_var = res.1.variable(builder, var_block);
    builder.store(res_var, spv_res, None, None).unwrap();
    var_map.insert(res.0, res_var);
}

fn get_object_assign(
    var_map: &mut HashMap<usize, u32>,
    lhs: &(usize, PrimitiveType),
    builder: &mut rspirv::dr::Builder,
    rhs: &(usize, PrimitiveType),
) -> (u32, u32, u32, u32) {
    let spv_lhs_id = *var_map.get(&lhs.0).unwrap();
    let lhs_ty = lhs.1.base_type(builder);
    let spv_lhs = builder.load(lhs_ty, None, spv_lhs_id, None, None).unwrap();
    let spv_rhs_id = *var_map.get(&rhs.0).unwrap();
    let rhs_ty = rhs.1.base_type(builder);
    let spv_rhs = builder.load(rhs_ty, None, spv_rhs_id, None, None).unwrap();
    (spv_lhs_id, lhs_ty, spv_lhs, spv_rhs)
}

fn process_add_assign(
    var_map: &mut HashMap<usize, u32>,
    lhs: &(usize, PrimitiveType),
    builder: &mut rspirv::dr::Builder,
    rhs: &(usize, PrimitiveType),
) {
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

fn process_sub_assign(
    var_map: &mut HashMap<usize, u32>,
    lhs: &(usize, PrimitiveType),
    builder: &mut rspirv::dr::Builder,
    rhs: &(usize, PrimitiveType),
) {
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

fn process_mul_assign(
    var_map: &mut HashMap<usize, u32>,
    lhs: &(usize, PrimitiveType),
    builder: &mut rspirv::dr::Builder,
    rhs: &(usize, PrimitiveType),
) {
    let (spv_lhs_id, lhs_ty, mut spv_lhs, mut spv_rhs) =
        get_object_assign(var_map, lhs, builder, rhs);
    let f = get_mul_fn_p(lhs, rhs, &mut spv_lhs, &mut spv_rhs);
    let spv_res = f(builder, lhs_ty, None, spv_lhs, spv_rhs).unwrap();
    builder.store(spv_lhs_id, spv_res, None, None).unwrap();
}

fn process_div_assign(
    var_map: &mut HashMap<usize, u32>,
    lhs: &(usize, PrimitiveType),
    builder: &mut rspirv::dr::Builder,
    rhs: &(usize, PrimitiveType),
) {
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

fn process_bit_op_assign(
    var_map: &mut HashMap<usize, u32>,
    lhs: &(usize, PrimitiveType),
    builder: &mut rspirv::dr::Builder,
    rhs: &(usize, PrimitiveType),
    f: fn(&mut rspirv::dr::Builder, u32, Option<u32>, u32, u32) -> Result<u32, rspirv::dr::Error>,
) {
    let (spv_lhs_id, lhs_ty, spv_lhs, spv_rhs) = get_object_assign(var_map, lhs, builder, rhs);
    let spv_res = f(builder, lhs_ty, None, spv_lhs, spv_rhs).unwrap();

    builder.store(spv_lhs_id, spv_res, None, None).unwrap();
}
