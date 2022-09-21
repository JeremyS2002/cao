
use either::*;
use rspirv::binary::Assemble;

use std::collections::HashMap;

pub(crate) struct RSpirvBuilder {
    pub(crate) raw: rspirv::dr::Builder,
    pub(crate) ext: u32,
    pub(crate) struct_map: HashMap<crate::StructType, u32>,
}

impl std::ops::Deref for RSpirvBuilder {
    type Target = rspirv::dr::Builder;

    fn deref(&self) -> &Self::Target {
        &self.raw
    }
}

impl std::ops::DerefMut for RSpirvBuilder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.raw
    }
}

#[derive(Clone, Debug)]
pub struct IOData {
    pub ty: crate::IOType,
    pub location: Either<u32, rspirv::spirv::BuiltIn>,
    pub flat: bool,
    pub name: Option<&'static str>
}

#[derive(Clone, Debug)]
pub struct PushData {
    pub ty: crate::Type,
    pub name: Option<&'static str>,
}

pub(crate) struct FuncData {
    pub ret: crate::Type,
    pub arguments: Vec<crate::Type>,
    pub instructions: Vec<crate::Instruction>,
    pub name: Option<&'static str>,
}

#[derive(Clone, Debug)]
pub struct UniformData {
    pub ty: crate::Type,
    pub set: u32,
    pub binding: u32,
    pub name: Option<&'static str>,
}

#[derive(Clone, Debug)]
pub struct StorageData {
    pub ty: crate::Type,
    pub read: bool,
    pub write: bool,
    pub set: u32, 
    pub binding: u32,
    pub name: Option<&'static str>,
}

#[derive(Clone, Debug)]
pub struct TextureData {
    pub set: u32,
    pub binding: u32,
    pub ty: crate::TextureType,
    pub name: Option<&'static str>,
}

#[derive(Clone, Debug)]
pub struct SampledTextureData {
    pub set: u32,
    pub binding: u32,
    pub ty: crate::TextureType,
    pub name: Option<&'static str>,
}

#[derive(Clone, Debug)]
pub struct SamplerData {
    pub set: u32,
    pub binding: u32,
    pub name: Option<&'static str>,
}

pub struct BuilderInner {
    pub(crate) inputs: Vec<IOData>,
    pub(crate) outputs: Vec<IOData>,
    pub(crate) push_constants: Option<PushData>,
    pub(crate) uniforms: Vec<UniformData>,
    pub(crate) storages: Vec<StorageData>,
    pub(crate) textures: Vec<TextureData>,
    pub(crate) sampled_textures: Vec<SampledTextureData>,
    pub(crate) samplers: Vec<SamplerData>,
    pub(crate) functions: HashMap<usize, FuncData>,
    pub(crate) entry_points: HashMap<crate::Stage, usize>,
    pub(crate) scope: Option<Box<dyn crate::Scope>>,
}

impl BuilderInner {
    pub fn new() -> Self {
        Self {
            inputs: Vec::new(),
            outputs: Vec::new(),
            push_constants: None,
            uniforms: Vec::new(),
            storages: Vec::new(),
            textures: Vec::new(),
            sampled_textures: Vec::new(),
            samplers: Vec::new(),
            functions: HashMap::new(),
            entry_points: HashMap::new(),
            scope: None,
        }
    }

    pub fn __scope<'a>(&'a mut self) -> Option<&'a mut dyn crate::Scope> {
        if let Some(scope) = &mut self.scope {
            Some(&mut **scope)
        } else {
            None
        }
    }
}

pub(crate) struct ShaderMapInfo {
    pub inputs: Vec<u32>,
    pub outputs: Vec<u32>,
    pub push_constants: Option<u32>,
    pub uniforms: Vec<u32>,
    pub storages: Vec<u32>,
    pub textures: Vec<u32>,
    pub sampled_textures: Vec<u32>,
    pub samplers: Vec<u32>,
    pub functions: HashMap<usize, (u32, usize)>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum BlockInfo {
    If {
        end_label: u32,
    },
    #[allow(dead_code)]
    Loop {
        condition_label: u32,
        continue_label: u32,
        end_label: u32,
    },
    None,
}

pub(crate) struct FuncMapInfo {
    pub vars: HashMap<usize, u32>,
    pub var_block: usize,
    pub block_info: BlockInfo,
}

impl FuncMapInfo {
    pub(crate) fn var(&mut self, b: &mut RSpirvBuilder, id: usize, ty: &crate::Type) -> u32 {
        if let Some(id) = self.vars.get(&id) {
            *id
        } else {
            let p_spv_ty = ty.pointer(b);
            let current_block = b.selected_block().unwrap();
            b.select_block(Some(self.var_block)).unwrap();
            let spv_id = b.id();
            b.insert_into_block(
                rspirv::dr::InsertPoint::Begin,
                rspirv::dr::Instruction::new(
                    rspirv::spirv::Op::Variable,
                    Some(p_spv_ty),
                    Some(spv_id),
                    vec![rspirv::dr::Operand::StorageClass(
                        rspirv::spirv::StorageClass::Function,
                    )],
                ),
            ).unwrap();
            b.select_block(Some(current_block)).unwrap();
            self.vars.insert(id, spv_id);
            spv_id
        }
    }
}

impl BuilderInner {
    /// Compile self into spir-v data
    pub fn compile(&self) -> Vec<u32> {
        let mut raw_builder = rspirv::dr::Builder::new();

        let ext = raw_builder.ext_inst_import("GLSL.std.450");

        let mut b = RSpirvBuilder {
            raw: raw_builder,
            ext,
            struct_map: HashMap::new(),
        };

        b.set_version(1, 0);
        b.capability(rspirv::spirv::Capability::Shader);
        b.memory_model(
            rspirv::spirv::AddressingModel::Logical, 
            rspirv::spirv::MemoryModel::GLSL450,
        );

        b.source(
            rspirv::spirv::SourceLanguage::Unknown,
            450,
            None,
            Option::<String>::None,
        );

        let shader_info = self.map_info(&mut b);

        for (id, func) in self.functions.iter() {
            let (_, fn_idx) = *shader_info.functions.get(id).unwrap();
            b.select_function(Some(fn_idx)).unwrap();
            
            b.begin_block(None).unwrap();
            let var_block = b.selected_block().unwrap();

            let mut func_info = FuncMapInfo {
                var_block,
                vars: HashMap::new(),
                block_info: BlockInfo::None,
            };

            let mut bl = false;
            for instruction in &func.instructions {
                bl |= instruction.compile(&mut b, &shader_info, &mut func_info);
                if bl {
                    break;
                }
            }

            if !bl {
                b.ret().unwrap();
            }
        }

        let mut interface = shader_info.inputs.clone();
        interface.extend_from_slice(&shader_info.outputs);

        for (stage, fn_id) in &self.entry_points {
            let (spv_fn, _) = *shader_info.functions.get(fn_id).unwrap();
            stage.specialize(&mut b, spv_fn);

            let func = self.functions.get(fn_id).unwrap();

            b.entry_point(stage.rspirv(), spv_fn, func.name.unwrap(), &interface)
        }

        b.raw.module().assemble()
    }

    fn map_info(&self, b: &mut RSpirvBuilder) -> ShaderMapInfo {
        ShaderMapInfo { 
            inputs: self.map_inputs(b), 
            outputs: self.map_outputs(b), 
            push_constants: self.map_push_constants(b), 
            uniforms: self.map_uniforms(b), 
            storages: self.map_storages(b),
            textures: self.map_textures(b),
            sampled_textures: self.map_sampled_textures(b),
            samplers: self.map_samplers(b),
            functions: self.map_functions(b),
        }
    }

    fn map_io<'a>(b: &mut RSpirvBuilder, iter: impl Iterator<Item = &'a IOData>, class: rspirv::spirv::StorageClass) -> Vec<u32> {
        iter.map(|i| {
            let spv_ty = i.ty.ty().rspirv(b);
            let p_spv_ty = b.type_pointer(None, class, spv_ty);
            let spv_var = b.variable(p_spv_ty, None, class, None);
            match i.location {
                Left(location) => {
                    b.decorate(
                        spv_var,
                        rspirv::spirv::Decoration::Location,
                        [rspirv::dr::Operand::LiteralInt32(location)]
                    );
                    if i.flat {
                        b.decorate(
                            spv_var,
                            rspirv::spirv::Decoration::Flat, 
                            []
                        );
                    }
                },
                Right(built_in) => {
                    b.decorate(
                        spv_var,
                        rspirv::spirv::Decoration::BuiltIn,
                        [rspirv::dr::Operand::BuiltIn(built_in)]
                    )
                },
            }

            if let Some(name) = i.name {
                b.name(spv_var, name);
            }

            spv_var
        })
        .collect::<Vec<_>>()
    }

    fn map_inputs(&self, b: &mut RSpirvBuilder) -> Vec<u32> {
        Self::map_io(b, self.inputs.iter(), rspirv::spirv::StorageClass::Input)
    }

    fn map_outputs(&self, b: &mut RSpirvBuilder) -> Vec<u32> {
        Self::map_io(b, self.outputs.iter(), rspirv::spirv::StorageClass::Output)
    }

    fn map_push_constants(&self, b: &mut RSpirvBuilder) -> Option<u32> {
        self.push_constants.as_ref().map(|p| {
            let spv_ty = p.ty.rspirv(b);
            let outer_spv_ty = b.type_struct([spv_ty]);

            b.decorate(
                outer_spv_ty, 
                rspirv::spirv::Decoration::Block, 
                None
            );

            b.member_decorate(
                outer_spv_ty, 
                0, 
                rspirv::spirv::Decoration::Offset, 
                Some(rspirv::dr::Operand::LiteralInt32(0))
            );

            let p_spv_ty = b.type_pointer(None, rspirv::spirv::StorageClass::PushConstant, outer_spv_ty);

            let var = b.variable(
                p_spv_ty, 
                None, 
                rspirv::spirv::StorageClass::PushConstant, 
                None
            );

            if let Some(name) = p.name {
                b.name(var, name);
            }

            var
        })
    }

    fn map_uniforms(&self, b: &mut RSpirvBuilder) -> Vec<u32> {
        self.uniforms
            .iter()
            .map(|u| {
                let spv_ty = u.ty.rspirv(b);
                let outer_spv_ty = b.type_struct([spv_ty]);

                b.decorate(outer_spv_ty, rspirv::spirv::Decoration::Block, None);
                b.member_decorate(
                    outer_spv_ty, 
                    0, 
                    rspirv::spirv::Decoration::Offset, 
                    [rspirv::dr::Operand::LiteralInt32(0)]
                );

                let p_spv_ty = b.type_pointer(None, rspirv::spirv::StorageClass::Uniform, outer_spv_ty);
                let var = b.variable(p_spv_ty, None, rspirv::spirv::StorageClass::Uniform, None);

                b.decorate(
                    var, 
                    rspirv::spirv::Decoration::DescriptorSet, 
                    Some(rspirv::dr::Operand::LiteralInt32(u.set))
                );

                b.decorate(
                    var,
                    rspirv::spirv::Decoration::Binding,
                    Some(rspirv::dr::Operand::LiteralInt32(u.binding))
                );

                if let Some(name) = u.name {
                    b.name(var, name);
                }

                var
            })
            .collect::<Vec<_>>()
    }

    fn map_storages(&self, b: &mut RSpirvBuilder) -> Vec<u32> {
        self.storages
            .iter()
            .map(|s| {
                let spv_ty = s.ty.rspirv(b);
                let array_spv_ty = b.type_runtime_array(spv_ty);

                b.decorate(
                    array_spv_ty,
                    rspirv::spirv::Decoration::ArrayStride,
                    // made sure sized on creation
                    Some(rspirv::dr::Operand::LiteralInt32(s.ty.size().unwrap()))
                );

                let outer_spv_ty = b.type_struct([array_spv_ty]);
                
                b.decorate(
                    outer_spv_ty, 
                    rspirv::spirv::Decoration::BufferBlock, 
                    None,
                );

                b.member_decorate(
                    outer_spv_ty, 
                    0, 
                    rspirv::spirv::Decoration::Offset, 
                    Some(rspirv::dr::Operand::LiteralInt32(0))
                );

                if !s.read {
                    b.member_decorate(
                        outer_spv_ty,
                        0,
                        rspirv::spirv::Decoration::NonReadable,
                        None,
                    );
                }
    
                if !s.write {
                    b.member_decorate(
                        outer_spv_ty,
                        0,
                        rspirv::spirv::Decoration::NonWritable,
                        None,
                    );
                }
                
                let p_spv_ty = b.type_pointer(None, rspirv::spirv::StorageClass::Uniform, outer_spv_ty);
                let var = b.variable(p_spv_ty, None, rspirv::spirv::StorageClass::Uniform, None);

                b.decorate(
                    var,
                    rspirv::spirv::Decoration::DescriptorSet,
                    Some(rspirv::dr::Operand::LiteralInt32(s.set))
                );

                b.decorate(
                    var,
                    rspirv::spirv::Decoration::Binding,
                    Some(rspirv::dr::Operand::LiteralInt32(s.binding))
                );

                if let Some(name) = s.name {
                    b.name(var, name);
                }

                var
            })  
            .collect::<Vec<_>>()
    }

    fn map_functions(&self, b: &mut RSpirvBuilder) -> HashMap<usize, (u32, usize)> {
        self.functions.iter().map(|(id, func)| {
            let spv_ret_ty = func.ret.rspirv(b);
            let spv_arguments_ty = func.arguments
                .iter()
                .map(|t| t.rspirv(b))
                .collect::<Vec<_>>();
            let spv_f_ty = b.type_function(spv_ret_ty, spv_arguments_ty);
            let spv_f = b.begin_function(spv_ret_ty, None, rspirv::spirv::FunctionControl::empty(), spv_f_ty).unwrap();
            let fn_idx = b.selected_function().unwrap();

            if let Some(name) = func.name {
                b.name(spv_f, name);
            }

            b.end_function().unwrap();

            (*id, (spv_f, fn_idx))
        }).collect()
    }

    fn map_textures(&self, b: &mut RSpirvBuilder) -> Vec<u32> {
        self.textures.iter()
            .map(|t| {
                let spv_tex_ty = t.ty.rspirv(b);

                let spv_p_ty = b.type_pointer(None, rspirv::spirv::StorageClass::UniformConstant, spv_tex_ty);

                let var = b.variable(spv_p_ty, None, rspirv::spirv::StorageClass::UniformConstant, None);
            
                b.decorate(
                    var,
                    rspirv::spirv::Decoration::DescriptorSet,
                    Some(rspirv::dr::Operand::LiteralInt32(t.set)),
                );

                b.decorate(
                    var,
                    rspirv::spirv::Decoration::Binding,
                    Some(rspirv::dr::Operand::LiteralInt32(t.binding)),
                );

                if let Some(name) = t.name {
                    b.name(var, name);
                }

                var
            })
            .collect()
    }

    fn map_sampled_textures(&self, b: &mut RSpirvBuilder) -> Vec<u32> {
        self.sampled_textures.iter()
            .map(|t| {
                let spv_tex_ty = t.ty.rspirv(b);

                let spv_sampled_tex_ty = b.type_sampled_image(spv_tex_ty);

                let spv_p_ty = b.type_pointer(None, rspirv::spirv::StorageClass::UniformConstant, spv_sampled_tex_ty);

                let var = b.variable(spv_p_ty, None, rspirv::spirv::StorageClass::UniformConstant, None);
            
                b.decorate(
                    var,
                    rspirv::spirv::Decoration::DescriptorSet,
                    Some(rspirv::dr::Operand::LiteralInt32(t.set)),
                );

                b.decorate(
                    var,
                    rspirv::spirv::Decoration::Binding,
                    Some(rspirv::dr::Operand::LiteralInt32(t.binding)),
                );

                if let Some(name) = t.name {
                    b.name(var, name);
                }

                var
            })
            .collect()
    }

    fn map_samplers(&self, b: &mut RSpirvBuilder) -> Vec<u32> {
        self.samplers.iter()
            .map(|s| {
                let spv_ty = b.type_sampler();
                let spv_p_ty = b.type_pointer(None, rspirv::spirv::StorageClass::UniformConstant, spv_ty);
                let var = b.variable(
                    spv_p_ty,
                    None,
                    rspirv::spirv::StorageClass::UniformConstant,
                    None,
                );

                b.decorate(
                    var,
                    rspirv::spirv::Decoration::DescriptorSet,
                    Some(rspirv::dr::Operand::LiteralInt32(s.set)),
                );

                b.decorate(
                    var,
                    rspirv::spirv::Decoration::Binding,
                    Some(rspirv::dr::Operand::LiteralInt32(s.binding)),
                );

                if let Some(name) = s.name {
                    b.name(var, name);
                }

                var
            })
            .collect()
    }
}