#![feature(const_type_id)]

//! A tool to build spir-v shaders at runtime built on rspirv

use data::IsPrimitiveType;
use either::*;
pub use interface::{Input, Output, Storage, StorageAccessDesc, Uniform};
use rspirv::binary::Assemble;

use std::cell::RefCell;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::rc::Rc;

use builder::{Instruction, RawBaseBuilder};

pub use rspirv;

pub mod builder;
pub mod data;
pub mod function;
pub mod interface;
pub mod prelude;
pub mod sampler;
pub mod specialisation;
pub mod texture;

pub use specialisation::{
    ComputeBuilder, FragmentBuilder, GeometryBuilder, TessControlBuilder, TessEvalBuilder,
    VertexBuilder,
};

pub use data::*;
pub use sampler::*;
pub use texture::*;

/// The main entry point to building a spir-v module
///
/// Create a builder specifiying what kind of spir-v module it will build using [`Builder::new`]
///
/// Example recieving a vec2 as vertex input and storing it to the output positon vulkan uses
///
/// ```
/// use spv::prelude::*;
///
/// let b = spv::VertexBuilder::new();
///
/// let in_pos = b.in_vec2(0, false, None);
/// let vk_pos = b.position();
///
/// b.main(|b| {
///     let pos = b.load_in(in_pos);
///     b.store_out(vk_pos, b.vec4(&pos.x(b), &pos.y(b), &0.0, &1.0));
/// });
/// ```
pub struct Builder<T> {
    /// Well well well, look who wants implement more features and can't remember how this works.
    ///
    /// Overview:
    ///     - stage 1. build a vector of [`Instruction`] for each function
    ///     - stage 2. iterate over the instructions to compile a spir-v module
    ///
    /// Stage 1 and 2 should be combined and done at the same time. There is no reason not to do this other than
    /// I got confused and it would be alot of work to change it now.
    ///
    /// Stage 1.
    ///     - Each variable (Primitives, Structs and Arrays) is represented by a unique usize
    ///     - There are multiple Builder types to make creating instructions easier
    ///         - MainBuilder : used to create the main function (TODO implement as a wrapper around FunctionBuilder<Void>)
    ///         - FunctionBuilder : used to create arbitrary functions
    ///         - ConditionBuilder : used to create IfChain instructions
    ///         - LoopBuilder : used to create Loop instructions
    ///     - Each builder type implements the same instruction set (implemented via a macro that duplicates the same code on each builder)
    ///     - Each builder has a RawBuilder inside that points to the previous builder
    ///         - MainBuilder and FunctionBuilder both point to RawBaseBuilder
    ///         - ConditionBuilder and LoopBuilder can point to any other builder
    ///     - RawBuilders provide an interface to push instructions and get new id's
    ///
    /// Stage 2.
    ///     - basically just ```for instruction in instructions { instruction.process(..) }```
    ///     - maps store relation between my (usize) variables id's and rspirv (u32) variable ids
    ///     - all data is stored as spri-v variables behind function local pointers
    ///     - map from StructDesc to spir-v struct id caches based on pointers not data so if anything breaks this could be it

    /// Always BaseBuilder
    raw: Rc<RawBaseBuilder>,
    _marker: PhantomData<T>,
}

impl<T: specialisation::ShaderTY> Builder<T> {
    pub fn new() -> Self {
        Self {
            raw: Rc::new(RawBaseBuilder::new()),
            _marker: PhantomData,
        }
    }

    pub fn get_instructions(&self) -> Vec<builder::Instruction> {
        (*self.raw.main.borrow()).clone()
    }

    pub fn get_inputs(
        &self,
    ) -> Vec<(
        PrimitiveType,
        Either<(u32, bool), rspirv::spirv::BuiltIn>,
        Option<&'static str>,
    )> {
        (*self.raw.inputs.borrow()).clone()
    }

    pub fn get_outputs(
        &self,
    ) -> Vec<(
        PrimitiveType,
        Either<(u32, bool), rspirv::spirv::BuiltIn>,
        Option<&'static str>,
    )> {
        (*self.raw.outputs.borrow()).clone()
    }

    pub fn get_push_constant(&self) -> Option<(DataType, u32, Option<&'static str>)> {
        (*self.raw.push_constant.borrow()).clone()
    }

    pub fn get_uniforms(&self) -> Vec<(DataType, u32, u32, Option<&'static str>)> {
        (*self.raw.uniforms.borrow()).clone()
    }

    pub fn get_storage(
        &self,
    ) -> Vec<(DataType, u32, u32, StorageAccessDesc, Option<&'static str>)> {
        (*self.raw.storages.borrow()).clone()
    }

    pub fn get_textures(
        &self,
    ) -> Vec<(
        rspirv::spirv::Dim,
        crate::texture::Component,
        bool,
        u32,
        u32,
        Option<&'static str>,
    )> {
        (*self.raw.textures.borrow()).clone()
    }

    pub fn get_samplers(&self) -> Vec<(u32, u32, Option<&'static str>)> {
        (*self.raw.samplers.borrow()).clone()
    }

    pub fn get_sampled_textures(
        &self,
    ) -> Vec<(
        rspirv::spirv::Dim,
        crate::texture::Component,
        bool,
        u32,
        u32,
        Option<&'static str>,
    )> {
        (*self.raw.sampled_textures.borrow()).clone()
    }

    #[cfg(feature = "gpu")]
    pub fn get_descriptor_layout_entry(
        &self,
        set: u32,
        binding: u32,
    ) -> Option<(gpu::DescriptorLayoutEntry, Option<&'static str>)> {
        self.raw.map.borrow().get(&(set, binding)).cloned()
    }

    #[cfg(feature = "gpu")]
    pub fn get_descriptor_layout_entries(
        &self,
    ) -> HashMap<(u32, u32), (gpu::DescriptorLayoutEntry, Option<&'static str>)> {
        (*self.raw.map.borrow()).clone()
    }

    pub fn get_functions(&self) -> HashMap<usize, Vec<Instruction>> {
        (*self.raw.functions.borrow()).clone()
    }

    pub fn spv_fn<R: data::DataRef, F: FnOnce(&builder::FnHandle) -> () + 'static>(
        &self,
        f: F,
    ) -> function::Function<R> {
        let id = 0;

        let b = builder::FnHandle {
            raw: Rc::new(builder::RawFnBuilder {
                builder: Rc::clone(&self.raw),
                id,
                instructions: RefCell::new(Vec::new()),
                variables: RefCell::default(),
            }),
        };

        f(&b);

        function::Function {
            id,
            _marker: PhantomData,
        }
    }

    pub fn input<P: IsPrimitiveType>(
        &self,
        location: u32,
        flat: bool,
        name: Option<&'static str>,
    ) -> Input<P> {
        let index = self.raw.inputs.borrow().len();
        self.raw
            .inputs
            .borrow_mut()
            .push((P::TY, Left((location, flat)), name));
        Input {
            index,
            _marker: PhantomData,
        }
    }

    pub fn output<P: IsPrimitiveType>(
        &self,
        location: u32,
        flat: bool,
        name: Option<&'static str>,
    ) -> Output<P> {
        let index = self.raw.outputs.borrow().len();
        self.raw
            .outputs
            .borrow_mut()
            .push((P::TY, Left((location, flat)), name));
        Output {
            index,
            _marker: PhantomData,
        }
    }

    pub fn push_constant<D: IsDataType>(&self, offset: Option<u32>, name: Option<&'static str>) {
        if self.raw.push_constant.borrow().is_some() {
            panic!("ERROR: Cannot create shader module with more than one set of push constants");
        }
        *self.raw.push_constant.borrow_mut() = Some((D::TY, offset.unwrap_or(0), name))
    }

    pub fn uniform<D: IsDataType>(
        &self,
        set: u32,
        binding: u32,
        name: Option<&'static str>,
    ) -> Uniform<D> {
        let index = self.raw.uniforms.borrow().len();
        self.raw
            .uniforms
            .borrow_mut()
            .push((D::TY, set, binding, name));
        #[cfg(feature = "gpu")]
        self.raw.map.borrow_mut().insert(
            (set, binding),
            (
                gpu::DescriptorLayoutEntry {
                    ty: gpu::DescriptorLayoutEntryType::UniformBuffer,
                    stage: T::GPU_STAGE,
                    count: std::num::NonZeroU32::new(1).unwrap(),
                },
                name,
            ),
        );
        Uniform {
            index,
            _marker: PhantomData,
        }
    }

    pub fn uniform_struct<S: AsSpvStruct>(
        &self,
        set: u32,
        binding: u32,
        name: Option<&'static str>,
    ) -> Uniform<Struct<S>> {
        self.uniform(set, binding, name)
    }

    pub fn storage<D: IsDataType>(
        &self,
        desc: StorageAccessDesc,
        set: u32,
        binding: u32,
        name: Option<&'static str>,
    ) -> Storage<D> {
        self.raw
            .storages
            .borrow_mut()
            .push((D::TY, set, binding, desc, name));
        #[cfg(feature = "gpu")]
        self.raw.map.borrow_mut().insert(
            (set, binding),
            (
                gpu::DescriptorLayoutEntry {
                    ty: gpu::DescriptorLayoutEntryType::StorageBuffer {
                        read_only: !desc.write,
                    },
                    stage: T::GPU_STAGE,
                    count: std::num::NonZeroU32::new(1).unwrap(),
                },
                name,
            ),
        );
        // Storage {
        //     set: todo!(),
        //     binding: todo!(),
        //     _marker: PhantomData,
        // }
        todo!();
    }

    pub fn sampler(&self, set: u32, binding: u32, name: Option<&'static str>) -> Sampler {
        let index = self.raw.samplers.borrow().len();
        self.raw.samplers.borrow_mut().push((set, binding, name));
        #[cfg(feature = "gpu")]
        self.raw.map.borrow_mut().insert(
            (set, binding),
            (
                gpu::DescriptorLayoutEntry {
                    ty: gpu::DescriptorLayoutEntryType::Sampler,
                    stage: T::GPU_STAGE,
                    count: std::num::NonZeroU32::new(1).unwrap(),
                },
                name,
            ),
        );
        Sampler { index }
    }

    pub fn main<F: FnOnce(&builder::MainHandle) -> ()>(&self, f: F) {
        let b = builder::MainHandle {
            raw: Rc::new(builder::RawMainBuilder {
                builder: Rc::clone(&self.raw),
                instructions: RefCell::new(Vec::new()),
                variables: RefCell::default(),
            }),
        };

        f(&b)
    }

    pub fn compile(&self) -> Vec<u32> {
        let mut builder = rspirv::dr::Builder::new();

        //let _ext = builder.ext_inst_import("GLSL.std.450");
        builder.set_version(1, 0);
        builder.capability(rspirv::spirv::Capability::Shader);
        builder.memory_model(
            rspirv::spirv::AddressingModel::Logical,
            rspirv::spirv::MemoryModel::GLSL450,
        );
        let ext = builder.ext_inst_import("GLSL.std.450");
        builder.source(
            rspirv::spirv::SourceLanguage::GLSL,
            450,
            None,
            Option::<String>::None,
        );

        // map from my function id to rspirv function id
        let mut function_map = HashMap::new();
        let mut struct_map = HashMap::new();

        // for (_, function) in self.functions() {

        // }

        let mut var_map = HashMap::new();

        let void = builder.type_void();
        let void_f = builder.type_function(void, []);
        let main = builder
            .begin_function(void, None, rspirv::spirv::FunctionControl::empty(), void_f)
            .unwrap();
        builder.name(main, "main");

        let uniforms =
            process_uniforms(&*self.raw.uniforms.borrow(), &mut builder, &mut struct_map);
        let storages = process_storages(&mut builder, &self.raw.storages.borrow(), &mut struct_map);
        let textures = process_textures(&mut builder, &self.raw.textures.borrow());
        let samplers = process_samplers(&mut builder, &self.raw.samplers.borrow());
        let sampled_textures =
            process_sampled_textures(&mut builder, &self.raw.sampled_textures.borrow());
        let inputs = process_io(
            &mut builder,
            &self.raw.inputs.borrow(),
            rspirv::spirv::StorageClass::Input,
        );
        let outputs = process_io(
            &mut builder,
            &self.raw.outputs.borrow(),
            rspirv::spirv::StorageClass::Output,
        );
        let push_constant = process_push_constant(
            &mut builder,
            &self.raw.push_constant.borrow(),
            &mut struct_map,
        );

        let mut interface = inputs.clone();
        interface.extend_from_slice(&outputs);

        builder.entry_point(T::TY, main, "main", interface);

        T::specialize(&mut builder, main);

        builder.begin_block(None).unwrap();
        let var_block = builder.selected_block().unwrap();

        let mut s = crate::builder::instruction::CompileState {
            var_map: &mut var_map,
            function_map: &mut function_map,
            struct_map: &mut struct_map,
            uniforms: &uniforms,
            storages: &storages,
            inputs: &inputs,
            outputs: &outputs,
            textures: &textures,
            samplers: &samplers,
            sampled_textures: &sampled_textures,
            push_constant,
            var_block,
            glsl_ext: ext,
        };

        for mut instruction in self.get_instructions() {
            instruction.process(&mut builder, &mut s, None, None);
        }

        builder.ret().unwrap();
        builder.end_function().unwrap();

        builder.module().assemble()
    }
}

fn process_push_constant(
    builder: &mut rspirv::dr::Builder,
    borrow: &std::cell::Ref<Option<(DataType, u32, Option<&str>)>>,
    struct_map: &mut HashMap<std::any::TypeId, u32>,
) -> Option<(u32, DataType)> {
    borrow.map(|b| {
        let base_type = b.0.base_type(builder, struct_map);
        let outer_type = builder.type_struct([base_type]);

        builder.decorate(outer_type, rspirv::spirv::Decoration::Block, None);
        builder.member_decorate(
            outer_type,
            0,
            rspirv::spirv::Decoration::Offset,
            [rspirv::dr::Operand::LiteralInt32(b.1)],
        );

        let pointer_type =
            builder.type_pointer(None, rspirv::spirv::StorageClass::PushConstant, outer_type);

        let variable = builder.variable(
            pointer_type,
            None,
            rspirv::spirv::StorageClass::PushConstant,
            None,
        );

        if let Some(name) = b.2 {
            builder.name(variable, name)
        }

        (variable, b.0)
    })
}

fn decorate_matrix(uniform: &DataType, builder: &mut rspirv::dr::Builder, raw_outer_ty: u32) {
    if let DataType::Primitive(p) = uniform {
        if p.is_matrix() {
            builder.member_decorate(raw_outer_ty, 0, rspirv::spirv::Decoration::ColMajor, None);

            builder.member_decorate(
                raw_outer_ty,
                0,
                rspirv::spirv::Decoration::Offset,
                Some(rspirv::dr::Operand::LiteralInt32(0)),
            );

            builder.member_decorate(
                raw_outer_ty,
                0,
                rspirv::spirv::Decoration::MatrixStride,
                Some(rspirv::dr::Operand::LiteralInt32(
                    p.matrix_stride().unwrap(),
                )),
            );
        }
    }
}

fn process_uniforms(
    uniforms: &[(DataType, u32, u32, Option<&'static str>)],
    builder: &mut rspirv::dr::Builder,
    struct_map: &mut HashMap<std::any::TypeId, u32>,
) -> Vec<u32> {
    uniforms
        .iter()
        .map(|(uniform, set, binding, name)| {
            let raw_inner_ty = uniform.base_type(builder, struct_map);
            let raw_outer_ty = builder.type_struct([raw_inner_ty]);

            decorate_matrix(uniform, builder, raw_outer_ty);

            builder.decorate(raw_outer_ty, rspirv::spirv::Decoration::Block, None);
            builder.member_decorate(
                raw_outer_ty,
                0,
                rspirv::spirv::Decoration::Offset,
                [rspirv::dr::Operand::LiteralInt32(0)],
            );

            let p_ty =
                builder.type_pointer(None, rspirv::spirv::StorageClass::Uniform, raw_outer_ty);
            let variable = builder.variable(p_ty, None, rspirv::spirv::StorageClass::Uniform, None);

            builder.decorate(
                variable,
                rspirv::spirv::Decoration::DescriptorSet,
                Some(rspirv::dr::Operand::LiteralInt32(*set)),
            );
            builder.decorate(
                variable,
                rspirv::spirv::Decoration::Binding,
                Some(rspirv::dr::Operand::LiteralInt32(*binding)),
            );

            if let Some(name) = *name {
                builder.name(variable, name)
            }

            variable
        })
        .collect::<Vec<_>>()
}

fn process_storages(
    builder: &mut rspirv::dr::Builder,
    borrow: &[(DataType, u32, u32, StorageAccessDesc, Option<&str>)],
    struct_map: &mut HashMap<std::any::TypeId, u32>,
) -> Vec<u32> {
    borrow
        .iter()
        .map(|(ty, set, binding, desc, name)| {
            let raw_inner_ty = ty.base_type(builder, struct_map);
            let raw_array_ty = builder.type_runtime_array(raw_inner_ty);

            builder.decorate(
                raw_array_ty,
                rspirv::spirv::Decoration::ArrayStride,
                Some(rspirv::dr::Operand::LiteralInt32(ty.size())),
            );

            let raw_outer_ty = builder.type_struct([raw_array_ty]);

            decorate_matrix(ty, builder, raw_outer_ty);

            builder.decorate(raw_outer_ty, rspirv::spirv::Decoration::BufferBlock, None);
            builder.member_decorate(
                raw_outer_ty,
                0,
                rspirv::spirv::Decoration::Offset,
                [rspirv::dr::Operand::LiteralInt32(0)],
            );

            if !desc.read {
                builder.member_decorate(
                    raw_outer_ty,
                    0,
                    rspirv::spirv::Decoration::NonReadable,
                    None,
                );
            }

            if !desc.write {
                builder.member_decorate(
                    raw_outer_ty,
                    0,
                    rspirv::spirv::Decoration::NonWritable,
                    None,
                );
            }

            let p_ty =
                builder.type_pointer(None, rspirv::spirv::StorageClass::Uniform, raw_outer_ty);
            let variable = builder.variable(p_ty, None, rspirv::spirv::StorageClass::Uniform, None);

            builder.decorate(
                variable,
                rspirv::spirv::Decoration::DescriptorSet,
                Some(rspirv::dr::Operand::LiteralInt32(*set)),
            );
            builder.decorate(
                variable,
                rspirv::spirv::Decoration::Binding,
                Some(rspirv::dr::Operand::LiteralInt32(*binding)),
            );

            if let Some(name) = *name {
                builder.name(variable, name)
            }

            variable
        })
        .collect()
}

fn process_textures(
    builder: &mut rspirv::dr::Builder,
    textures: &[(
        rspirv::spirv::Dim,
        crate::texture::Component,
        bool,
        u32,
        u32,
        Option<&'static str>,
    )],
) -> Vec<(u32, u32)> {
    textures
        .iter()
        .map(|(dimension, component, arrayed, set, binding, name)| {
            let c_type = component.base_type(builder);
            let t_type = builder.type_image(
                c_type,
                *dimension,
                0,
                if *arrayed { 1 } else { 0 },
                0,
                1,
                rspirv::spirv::ImageFormat::Unknown,
                None,
            );

            let p_type =
                builder.type_pointer(None, rspirv::spirv::StorageClass::UniformConstant, t_type);

            let variable = builder.variable(
                p_type,
                None,
                rspirv::spirv::StorageClass::UniformConstant,
                None,
            );

            builder.decorate(
                variable,
                rspirv::spirv::Decoration::DescriptorSet,
                Some(rspirv::dr::Operand::LiteralInt32(*set)),
            );

            builder.decorate(
                variable,
                rspirv::spirv::Decoration::Binding,
                Some(rspirv::dr::Operand::LiteralInt32(*binding)),
            );

            if let Some(name) = *name {
                builder.name(variable, name)
            }

            (variable, t_type)
        })
        .collect::<Vec<_>>()
}

fn process_samplers(
    builder: &mut rspirv::dr::Builder,
    samplers: &[(u32, u32, Option<&'static str>)],
) -> Vec<(u32, u32)> {
    samplers
        .iter()
        .map(|(set, binding, name)| {
            let b_type = builder.type_sampler();
            let p_type =
                builder.type_pointer(None, rspirv::spirv::StorageClass::UniformConstant, b_type);
            let variable = builder.variable(
                p_type,
                None,
                rspirv::spirv::StorageClass::UniformConstant,
                None,
            );

            builder.decorate(
                variable,
                rspirv::spirv::Decoration::DescriptorSet,
                Some(rspirv::dr::Operand::LiteralInt32(*set)),
            );

            builder.decorate(
                variable,
                rspirv::spirv::Decoration::Binding,
                Some(rspirv::dr::Operand::LiteralInt32(*binding)),
            );

            if let Some(name) = *name {
                builder.name(variable, name)
            }

            (variable, b_type)
        })
        .collect::<Vec<_>>()
}

fn process_sampled_textures(
    builder: &mut rspirv::dr::Builder,
    textures: &[(
        rspirv::spirv::Dim,
        crate::texture::Component,
        bool,
        u32,
        u32,
        Option<&'static str>,
    )],
) -> Vec<(u32, u32)> {
    textures
        .iter()
        .map(|(dimension, component, arrayed, set, binding, name)| {
            let c_type = component.base_type(builder);
            let t_type = builder.type_image(
                c_type,
                *dimension,
                0,
                if *arrayed { 1 } else { 0 },
                0,
                1,
                rspirv::spirv::ImageFormat::Unknown,
                None,
            );

            let st_type = builder.type_sampled_image(t_type);

            let p_type =
                builder.type_pointer(None, rspirv::spirv::StorageClass::UniformConstant, st_type);

            let variable = builder.variable(
                p_type,
                None,
                rspirv::spirv::StorageClass::UniformConstant,
                None,
            );

            builder.decorate(
                variable,
                rspirv::spirv::Decoration::DescriptorSet,
                Some(rspirv::dr::Operand::LiteralInt32(*set)),
            );

            builder.decorate(
                variable,
                rspirv::spirv::Decoration::Binding,
                Some(rspirv::dr::Operand::LiteralInt32(*binding)),
            );

            if let Some(name) = *name {
                builder.name(variable, name)
            }

            (variable, st_type)
        })
        .collect::<Vec<_>>()
}

fn process_io(
    builder: &mut rspirv::dr::Builder,
    io: &[(
        PrimitiveType,
        Either<(u32, bool), rspirv::spirv::BuiltIn>,
        Option<&'static str>,
    )],
    storage: rspirv::spirv::StorageClass,
) -> Vec<u32> {
    let inputs = io
        .iter()
        .map(|(v, t, name)| {
            let ty = v.base_type(builder);
            let pointer_ty = builder.type_pointer(None, storage, ty);
            let variable = builder.variable(pointer_ty, None, storage, None);

            match t {
                Left((location, flat)) => {
                    builder.decorate(
                        variable,
                        rspirv::spirv::Decoration::Location,
                        [rspirv::dr::Operand::LiteralInt32(*location)],
                    );
                    if *flat {
                        builder.decorate(variable, rspirv::spirv::Decoration::Flat, []);
                    }
                }
                Right(built_in) => builder.decorate(
                    variable,
                    rspirv::spirv::Decoration::BuiltIn,
                    [rspirv::dr::Operand::BuiltIn(*built_in)],
                ),
            }

            if let Some(name) = name {
                builder.name(variable, *name);
            }
            variable
        })
        .collect::<Vec<_>>();
    inputs
}

macro_rules! io_interp_types {
    ($($i_name:ident, $o_name:ident, $t_name:ident,)*) => {
        impl<T: specialisation::ShaderTY> Builder<T> {
            $(
                pub fn $i_name(&self, location: u32, flat: bool, name: Option<&'static str>) -> Input<$t_name> {
                    self.input(location, flat, name)
                }

                pub fn $o_name(&self, location: u32, flat: bool, name: Option<&'static str>) -> Output<$t_name> {
                    self.output(location, flat, name)
                }
            )*
        }
    };
}

#[rustfmt::skip]
io_interp_types!(
    in_float, out_float, Float, 
    in_vec2, out_vec2, Vec2, 
    in_vec3, out_vec3, Vec3, 
    in_vec4, out_vec4, Vec4, 
    in_double, out_double, Double, 
    in_dvec2, out_dvec2, DVec2, 
    in_dvec3, out_dvec3, DVec3, 
    in_dvec4, out_dvec4, DVec4,
);

macro_rules! io_no_interp_types {
    ($($i_name:ident, $o_name:ident, $t_name:ident,)*) => {
        impl<T: specialisation::ShaderTY> Builder<T> {
            $(
                pub fn $i_name(&self, location: u32, name: Option<&'static str>) -> Input<$t_name> {
                    self.input(location, true, name)
                }

                pub fn $o_name(&self, location: u32, name: Option<&'static str>) -> Output<$t_name> {
                    self.output(location, true, name)
                }
            )*
        }
    };
}

#[rustfmt::skip]
io_no_interp_types!(
    in_bool, out_bool, Bool, 
    in_int, out_int, Int, 
    in_ivec2, out_ivec2, IVec2, 
    in_ivec3, out_ivec3, IVec3, 
    in_ivec4, out_ivec4, IVec4, 
    in_uint, out_uint, UInt, 
    in_uvec2, out_uvec2, UVec2,
    in_uvec3, out_uvec3, UVec3, 
    in_uvec4, out_uvec4, UVec4,
);

macro_rules! impl_texture {
    ($($fn_name:ident, $alias:ident, $s_name:ident, $comp:ident,)*) => {
        $(
            pub fn $fn_name(
                &self,
                set: u32,
                binding: u32,
                name: Option<&'static str>
            ) -> $alias {
                $s_name(self.raw_texture(set, binding, Component::$comp, name))
            }
        )*
    };
}

macro_rules! impl_sampled_texture {
    ($($fn_name:ident, $alias:ident, $s_name:ident, $comp:ident,)*) => {
        $(
            pub fn $fn_name(
                &self,
                set: u32,
                binding: u32,
                name: Option<&'static str>
            ) -> $alias {
                $s_name(self.sampled_raw_texture(set, binding, Component::$comp, name))
            }
        )*
    };
}

/// Texture impls
impl<T: specialisation::ShaderTY> Builder<T> {
    #[rustfmt::skip]
    impl_texture!(
        texture_1d, Texture1D, Texture, Float,
        d_texture_1d, DTexture1D, DTexture, Double,
        i_texture_1d, ITexture1D, ITexture, Int,
        u_texture_1d, UTexture1D, UTexture, UInt,
        texture_1d_array, Texture1DArray, Texture, Float,
        d_texture_1d_array, DTexture1DArray, DTexture, Double,
        i_texture_1d_array, ITexture1DArray, ITexture, Int,
        u_texture_1d_array, UTexture1DArray, UTexture, UInt,
        texture_2d, Texture2D, Texture, Float,
        d_texture_2d, DTexture2D, DTexture, Double,
        i_texture_2d, ITexture2D, ITexture, Int,
        u_texture_2d, UTexture2D, UTexture, UInt,
        texture_2d_array, Texture2DArray, Texture, Float,
        d_texture_2d_array, DTexture2DArray, DTexture, Double,
        i_texture_2d_array, ITexture2DArray, ITexture, Int,
        u_texture_2d_array, UTexture2DArray, UTexture, UInt,
        texture_3d, Texture3D, Texture, Float,
        d_texture_3d, DTexture3D, DTexture, Double,
        i_texture_3d, ITexture3D, ITexture, Int,
        u_texture_3d, UTexture3D, UTexture, UInt,
        texture_cube, TextureCube, Texture, Float,
        d_texture_cube, DTextureCube, DTexture, Double,
        i_texture_cube, ITextureCube, ITexture, Int,
        u_texture_cube, UTextureCube, UTexture, UInt,
        texture_cube_array, TextureCubeArray, Texture, Float,
        d_texture_cube_array, DTextureCubeArray, DTexture, Double,
        i_texture_cube_array, ITextureCubeArray, ITexture, Int,
        u_texture_cube_array, UTextureCubeArray, UTexture, UInt,
    );

    pub fn texture<D: AsDimension>(
        &self,
        set: u32,
        binding: u32,
        name: Option<&'static str>,
    ) -> Texture<D> {
        Texture(self.raw_texture(set, binding, Component::Float, name))
    }

    pub fn d_texture<D: AsDimension>(
        &self,
        set: u32,
        binding: u32,
        name: Option<&'static str>,
    ) -> DTexture<D> {
        DTexture(self.raw_texture(set, binding, Component::Double, name))
    }

    pub fn i_texture<D: AsDimension>(
        &self,
        set: u32,
        binding: u32,
        name: Option<&'static str>,
    ) -> ITexture<D> {
        ITexture(self.raw_texture(set, binding, Component::Int, name))
    }

    pub fn u_texture<D: AsDimension>(
        &self,
        set: u32,
        binding: u32,
        name: Option<&'static str>,
    ) -> UTexture<D> {
        UTexture(self.raw_texture(set, binding, Component::UInt, name))
    }

    fn raw_texture<D: AsDimension>(
        &self,
        set: u32,
        binding: u32,
        component: Component,
        name: Option<&'static str>,
    ) -> RawTexture<D> {
        let index = self.raw.textures.borrow().len();
        self.raw
            .textures
            .borrow_mut()
            .push((D::DIM, component, D::ARRAYED, set, binding, name));
        #[cfg(feature = "gpu")]
        self.raw.map.borrow_mut().insert(
            (set, binding),
            (
                gpu::DescriptorLayoutEntry {
                    ty: gpu::DescriptorLayoutEntryType::SampledTexture,
                    stage: T::GPU_STAGE,
                    count: std::num::NonZeroU32::new(1).unwrap(),
                },
                name,
            ),
        );
        RawTexture {
            index,
            _dmarker: PhantomData,
        }
    }

    #[rustfmt::skip]
    impl_sampled_texture!(
        sampled_texture_1d, SampledTexture1D, SampledTexture, Float,
        sampled_d_texture_1d, SampledDTexture1D, SampledDTexture, Double,
        sampled_i_texture_1d, SampledITexture1D, SampledITexture, Int,
        sampled_u_texture_1d, SampledUTexture1D, SampledUTexture, UInt,
        sampled_texture_1d_array, SampledTexture1DArray, SampledTexture, Float,
        sampled_d_texture_1d_array, SampledDTexture1DArray, SampledDTexture, Double,
        sampled_i_texture_1d_array, SampledITexture1DArray, SampledITexture, Int,
        sampled_u_texture_1d_array, SampledUTexture1DArray, SampledUTexture, UInt,
        sampled_texture_2d, SampledTexture2D, SampledTexture, Float,
        sampled_d_texture_2d, SampledDTexture2D, SampledDTexture, Double,
        sampled_i_texture_2d, SampledITexture2D, SampledITexture, Int,
        sampled_u_texture_2d, SampledUTexture2D, SampledUTexture, UInt,
        sampled_texture_2d_array, SampledTexture2DArray, SampledTexture, Float,
        sampled_d_texture_2d_array, SampledDTexture2DArray, SampledDTexture, Double,
        sampled_i_texture_2d_array, SampledITexture2DArray, SampledITexture, Int,
        sampled_u_texture_2d_array, SampledUTexture2DArray, SampledUTexture, UInt,
        sampled_texture_3d, SampledTexture3D, SampledTexture, Float,
        sampled_d_texture_3d, SampledDTexture3D, SampledDTexture, Double,
        sampled_i_texture_3d, SampledITexture3D, SampledITexture, Int,
        sampled_u_texture_3d, SampledUTexture3D, SampledUTexture, UInt,
        sampled_texture_cube, SampledTextureCube, SampledTexture, Float,
        sampled_d_texture_cube, SampledDTextureCube, SampledDTexture, Double,
        sampled_i_texture_cube, SampledITextureCube, SampledITexture, Int,
        sampled_u_texture_cube, SampledUTextureCube, SampledUTexture, UInt,
        sampled_texture_cube_array, SampledTextureCubeArray, SampledTexture, Float,
        sampled_d_texture_cube_array, SampledDTextureCubeArray, SampledDTexture, Double,
        sampled_i_texture_cube_array, SampledITextureCubeArray, SampledITexture, Int,
        sampled_u_texture_cube_array, SampledUTextureCubeArray, SampledUTexture, UInt,

    );

    pub fn sampled_texture<D: AsDimension>(
        &self,
        set: u32,
        binding: u32,
        name: Option<&'static str>,
    ) -> SampledTexture<D> {
        SampledTexture(self.sampled_raw_texture(set, binding, Component::Float, name))
    }

    pub fn sampled_d_texture<D: AsDimension>(
        &self,
        set: u32,
        binding: u32,
        name: Option<&'static str>,
    ) -> SampledDTexture<D> {
        SampledDTexture(self.sampled_raw_texture(set, binding, Component::Double, name))
    }

    pub fn sampled_i_texture<D: AsDimension>(
        &self,
        set: u32,
        binding: u32,
        name: Option<&'static str>,
    ) -> SampledITexture<D> {
        SampledITexture(self.sampled_raw_texture(set, binding, Component::Int, name))
    }

    pub fn sampled_u_texture<D: AsDimension>(
        &self,
        set: u32,
        binding: u32,
        name: Option<&'static str>,
    ) -> SampledUTexture<D> {
        SampledUTexture(self.sampled_raw_texture(set, binding, Component::UInt, name))
    }

    fn sampled_raw_texture<D: AsDimension>(
        &self,
        set: u32,
        binding: u32,
        component: Component,
        name: Option<&'static str>,
    ) -> SampledRawTexture<D> {
        let index = self.raw.sampled_textures.borrow().len();
        self.raw.sampled_textures.borrow_mut().push((
            D::DIM,
            component,
            D::ARRAYED,
            set,
            binding,
            name,
        ));
        #[cfg(feature = "gpu")]
        self.raw.map.borrow_mut().insert(
            (set, binding),
            (
                gpu::DescriptorLayoutEntry {
                    ty: gpu::DescriptorLayoutEntryType::CombinedTextureSampler,
                    stage: T::GPU_STAGE,
                    count: std::num::NonZeroU32::new(1).unwrap(),
                },
                name,
            ),
        );
        SampledRawTexture {
            id: Left(index),
            _dmarker: PhantomData,
        }
    }
}
