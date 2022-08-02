use spirv_reflect::types::ReflectTypeFlags;

use std::any::TypeId;
use std::collections::HashMap;
use std::num::NonZeroU32;

use super::error;

pub(crate) fn parse_vertex_states(
    vertex: &[u32],
    vertex_entry: &str,
) -> Result<Vec<super::graphics::VertexLocationInfo>, error::ParseSpirvError> {
    let module = match spirv_reflect::ShaderModule::load_u32_data(vertex) {
        Ok(m) => m,
        Err(m) => return Err(error::ReflectError(m).into()),
    };

    let mut t = match module.enumerate_input_variables(Some(vertex_entry)) {
        Ok(t) => t
            .into_iter()
            .filter_map(|i| {
                if *i.built_in == spirv_headers::BuiltIn::Position {
                    let vertex_format = match i.format {
                        spirv_reflect::types::image::ReflectFormat::R32_SFLOAT => {
                            gpu::VertexFormat::Float
                        }
                        spirv_reflect::types::image::ReflectFormat::R32G32_SFLOAT => {
                            gpu::VertexFormat::Vec2
                        }
                        spirv_reflect::types::image::ReflectFormat::R32G32B32_SFLOAT => {
                            gpu::VertexFormat::Vec3
                        }
                        spirv_reflect::types::image::ReflectFormat::R32G32B32A32_SFLOAT => {
                            gpu::VertexFormat::Vec4
                        }
                        f => panic!("ERROR: Input format {:?} is not supported at the moment", f),
                    };
                    Some((i.location, i.name, vertex_format))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>(),
        Err(e) => return Err(error::ReflectError(e).into()),
    };

    t.sort_unstable_by(|a, b| a.0.cmp(&b.0));

    Ok(t.into_iter()
        .map(|(_, name, format)| super::graphics::VertexLocationInfo { name, format })
        .collect::<Vec<_>>())
}

pub(crate) fn combine_descriptor_set_layouts(
    device: &gpu::Device,
    descriptor_set_layouts: HashMap<u32, HashMap<u32, gpu::DescriptorLayoutEntry>>,
    name: &Option<String>,
) -> Result<
    (
        Vec<gpu::DescriptorLayout>,
        Vec<Vec<(gpu::DescriptorLayoutEntryType, u32)>>,
    ),
    gpu::Error,
> {
    // sort the hashmaps into ordered vecs
    let mut sorted = descriptor_set_layouts
        .into_iter()
        .map(|(k, v)| {
            let mut v = v.into_iter().collect::<Vec<_>>();
            v.sort_unstable_by(|a, b| a.0.cmp(&b.0));
            (k, v.into_iter().map(|a| a.1).collect::<Vec<_>>())
        })
        .collect::<Vec<_>>();
    sorted.sort_unstable_by(|a, b| a.0.cmp(&b.0));
    let sorted = sorted.into_iter().map(|a| a.1).collect::<Vec<_>>();

    // get the resource types
    let resource_types = sorted
        .iter()
        .map(|v| v.iter().map(|e| (e.ty, e.count.get())).collect::<Vec<_>>())
        .collect::<Vec<_>>();

    let mut i = 0;
    // create descriptor set layouts from the entries
    let descriptor_set_layouts = sorted
        .into_iter()
        .map(|v| {
            let layout_name = name
                .as_ref()
                .map(|n| format!("{}_descriptor_layout_{}", n, i));
            let l = device.create_descriptor_layout(&gpu::DescriptorLayoutDesc {
                name: layout_name,
                entries: &v,
            });
            i += 1;
            l
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok((descriptor_set_layouts, resource_types))
}

pub(crate) fn parse_spirv(
    descriptor_sets: &mut HashMap<u32, HashMap<u32, gpu::DescriptorLayoutEntry>>,
    descriptor_set_names: &mut HashMap<String, (usize, usize)>,
    push_constants: &mut Vec<gpu::PushConstantRange>,
    push_constant_names: &mut HashMap<String, (u32, gpu::ShaderStages, TypeId)>,
    spirv: &[u32],
    stage: spirv_reflect::types::variable::ReflectShaderStageFlags,
) -> Result<String, error::ParseSpirvError> {
    let module = match spirv_reflect::ShaderModule::load_u32_data(spirv) {
        Ok(m) => m,
        Err(m) => return Err(error::ReflectError(m).into()),
    };

    let reflect_stage = module.get_shader_stage();

    let shader_stage;
    if reflect_stage.contains(spirv_reflect::types::variable::ReflectShaderStageFlags::VERTEX) {
        shader_stage = gpu::ShaderStages::VERTEX;
    } else if reflect_stage
        .contains(spirv_reflect::types::variable::ReflectShaderStageFlags::FRAGMENT)
    {
        shader_stage = gpu::ShaderStages::FRAGMENT;
    } else if reflect_stage
        .contains(spirv_reflect::types::variable::ReflectShaderStageFlags::GEOMETRY)
    {
        shader_stage = gpu::ShaderStages::GEOMETRY;
    } else if reflect_stage
        .contains(spirv_reflect::types::variable::ReflectShaderStageFlags::COMPUTE)
    {
        shader_stage = gpu::ShaderStages::COMPUTE;
    } else {
        unimplemented!();
    }

    parse_descriptor_set_layouts(descriptor_sets, descriptor_set_names, &module, shader_stage)?;
    parse_push_constants(push_constants, push_constant_names, &module, shader_stage)?;

    Ok(get_entry_point(&module, stage)?)
}

pub(crate) fn check_stage_compatibility(
    src: &[u32],
    src_stage_name: &str,
    dst: &[u32],
    dst_stage_name: &str,
) -> Result<(), error::ParseSpirvError> {
    let src_module = match spirv_reflect::ShaderModule::load_u32_data(src) {
        Ok(m) => m,
        Err(m) => return Err(error::ReflectError(m).into()),
    };

    let dst_module = match spirv_reflect::ShaderModule::load_u32_data(dst) {
        Ok(m) => m,
        Err(m) => return Err(error::ReflectError(m).into()),
    };

    let mut outputs = match src_module.enumerate_output_variables(None) {
        Ok(i) => i,
        Err(m) => return Err(error::ReflectError(m).into()),
    };

    outputs.sort_unstable_by(|a, b| a.location.cmp(&b.location));

    let mut inputs = match dst_module.enumerate_input_variables(None) {
        Ok(i) => i,
        Err(m) => return Err(error::ReflectError(m).into()),
    };

    inputs.sort_unstable_by(|a, b| a.location.cmp(&b.location));

    for input in inputs.iter() {
        // I don't know why I did this when they are already sorted by location but don't want to change it
        // TODO test this
        if let Some(output) = outputs.iter().find(|v| v.location == input.location) {
            let undefined = spirv_reflect::types::ReflectFormat::Undefined;
            if output.format != input.format
                && output.format != undefined
                && input.format != undefined
            {
                Err(error::ParseSpirvError::StageIncompatibility {
                    location: output.location,
                    src_stage_name: src_stage_name.to_owned(),
                    src_type: output.format,
                    dst_stage_name: dst_stage_name.to_owned(),
                    dst_type: input.format,
                })?;
            }
        }
    }

    Ok(())
}

pub(crate) fn get_entry_point(
    module: &spirv_reflect::ShaderModule,
    stage: spirv_reflect::types::variable::ReflectShaderStageFlags,
) -> Result<String, error::ParseSpirvError> {
    let entry_points = match module.enumerate_entry_points() {
        Ok(e) => e,
        Err(m) => return Err(error::ReflectError(m).into()),
    };

    for entry in entry_points {
        if entry.shader_stage.contains(stage) {
            return Ok(entry.name);
        }
    }

    return Err(error::ParseSpirvError::EntryPointNotFound(
        stage, 
        module.enumerate_entry_points().unwrap().into_iter().map(|e| e.shader_stage).collect()).into()
    );
}

pub(crate) fn parse_descriptor_set_layouts(
    descriptor_sets: &mut HashMap<u32, HashMap<u32, gpu::DescriptorLayoutEntry>>,
    descriptor_set_names: &mut HashMap<String, (usize, usize)>,
    module: &spirv_reflect::ShaderModule,
    shader_stage: gpu::ShaderStages,
) -> Result<(), error::ParseSpirvError> {
    let mut sets = match module.enumerate_descriptor_sets(None) {
        Ok(s) => s,
        Err(m) => return Err(error::ReflectError(m).into()),
    };
    for set in sets.iter_mut() {
        let bindings_map = descriptor_sets.entry(set.set).or_insert(HashMap::new());
        set.bindings
            .sort_unstable_by(|x, y| x.binding.cmp(&y.binding));
        let set_idx = set.set;
        for binding in &mut set.bindings {
            let binding_idx = binding.binding;
            match descriptor_set_names.get(&binding.name) {
                None => {
                    descriptor_set_names.insert(
                        binding.name.clone(),
                        (set.set as usize, binding.binding as usize),
                    );
                }
                #[cfg(not(feature = "logging"))]
                _ => (),
                #[cfg(feature = "logging")]
                Some((s, b)) => {
                    if *s != set_idx as _ || *b != binding_idx as _ {
                        log::info!("multiple bindings share the same name set_resource_by_location needed to build bundles\nset 1: {}, binding 1: {}\n set 2: {}, binding 2: {}", *s, *b, set_idx, binding_idx)
                    }
                }
            };
            match binding.descriptor_type {
                spirv_reflect::types::descriptor::ReflectDescriptorType::Sampler => {
                    if let Some(b) = bindings_map.get_mut(&binding.binding) {
                        if let gpu::DescriptorLayoutEntryType::Sampler = b.ty {
                            b.stage |= shader_stage;
                        } else {
                            let n1 = binding.name.clone();
                            let n2 = descriptor_set_names
                                .iter()
                                .find(|(n, (s, b))| {
                                    set_idx == *s as u32
                                        && binding_idx == *b as u32
                                        && **n != binding.name
                                })
                                .unwrap()
                                .0
                                .clone();
                            return Err(error::ParseSpirvError::SetConflict(
                                set.set,
                                binding.binding,
                                n1,
                                n2,
                            )
                            .into());
                        }
                    } else {
                        bindings_map.insert(
                            binding.binding,
                            gpu::DescriptorLayoutEntry {
                                ty: gpu::DescriptorLayoutEntryType::Sampler,
                                stage: shader_stage,
                                count: NonZeroU32::new(binding.count).unwrap(),
                            },
                        );
                    }
                }
                spirv_reflect::types::descriptor::ReflectDescriptorType::UniformBuffer => {
                    if let Some(b) = bindings_map.get_mut(&binding.binding) {
                        if let gpu::DescriptorLayoutEntryType::UniformBuffer = b.ty {
                            b.stage |= shader_stage;
                        } else {
                            let n1 = binding.name.clone();
                            let n2 = descriptor_set_names
                                .iter()
                                .find(|(n, (s, b))| {
                                    set_idx == *s as u32
                                        && binding_idx == *b as u32
                                        && **n != binding.name
                                })
                                .unwrap()
                                .0
                                .clone();
                            return Err(error::ParseSpirvError::SetConflict(
                                set.set,
                                binding.binding,
                                n1,
                                n2,
                            )
                            .into());
                        }
                    } else {
                        bindings_map.insert(
                            binding.binding,
                            gpu::DescriptorLayoutEntry {
                                ty: gpu::DescriptorLayoutEntryType::UniformBuffer,
                                stage: shader_stage,
                                count: NonZeroU32::new(binding.count).unwrap(),
                            },
                        );
                    }
                }
                spirv_reflect::types::descriptor::ReflectDescriptorType::StorageBuffer => {
                    if let Some(b) = bindings_map.get_mut(&binding.binding) {
                        if let gpu::DescriptorLayoutEntryType::StorageBuffer { .. } = b.ty {
                            b.stage |= shader_stage;
                        } else {
                            let n1 = binding.name.clone();
                            let n2 = descriptor_set_names
                                .iter()
                                .find(|(n, (s, b))| {
                                    set_idx == *s as u32
                                        && binding_idx == *b as u32
                                        && **n != binding.name
                                })
                                .unwrap()
                                .0
                                .clone();
                            return Err(error::ParseSpirvError::SetConflict(
                                set.set,
                                binding.binding,
                                n1,
                                n2,
                            )
                            .into());
                        }
                    } else {
                        bindings_map.insert(
                            binding.binding,
                            gpu::DescriptorLayoutEntry {
                                ty: gpu::DescriptorLayoutEntryType::StorageBuffer {
                                    read_only: false,
                                },
                                stage: shader_stage,
                                count: NonZeroU32::new(binding.count).unwrap(),
                            },
                        );
                    }
                }
                spirv_reflect::types::descriptor::ReflectDescriptorType::SampledImage => {
                    if let Some(b) = bindings_map.get_mut(&binding.binding) {
                        if let gpu::DescriptorLayoutEntryType::SampledTexture = b.ty {
                            b.stage |= shader_stage;
                        } else {
                            let n1 = binding.name.clone();
                            let n2 = descriptor_set_names
                                .iter()
                                .find(|(n, (s, b))| {
                                    set_idx == *s as u32
                                        && binding_idx == *b as u32
                                        && **n != binding.name
                                })
                                .unwrap()
                                .0
                                .clone();
                            return Err(error::ParseSpirvError::SetConflict(
                                set.set,
                                binding.binding,
                                n1,
                                n2,
                            )
                            .into());
                        }
                    } else {
                        bindings_map.insert(
                            binding.binding,
                            gpu::DescriptorLayoutEntry {
                                ty: gpu::DescriptorLayoutEntryType::SampledTexture,
                                stage: shader_stage,
                                count: NonZeroU32::new(binding.count).unwrap(),
                            },
                        );
                    }
                }
                spirv_reflect::types::descriptor::ReflectDescriptorType::StorageImage => {
                    if let Some(b) = bindings_map.get_mut(&binding.binding) {
                        if let gpu::DescriptorLayoutEntryType::Sampler = b.ty {
                            b.stage |= shader_stage;
                        } else {
                            let n1 = binding.name.clone();
                            let n2 = descriptor_set_names
                                .iter()
                                .find(|(n, (s, b))| {
                                    set_idx == *s as u32
                                        && binding_idx == *b as u32
                                        && **n != binding.name
                                })
                                .unwrap()
                                .0
                                .clone();
                            return Err(error::ParseSpirvError::SetConflict(
                                set.set,
                                binding.binding,
                                n1,
                                n2,
                            )
                            .into());
                        }
                    } else {
                        bindings_map.insert(
                            binding.binding,
                            gpu::DescriptorLayoutEntry {
                                ty: gpu::DescriptorLayoutEntryType::StorageTexture {
                                    read_only: false,
                                },
                                stage: shader_stage,
                                count: NonZeroU32::new(binding.count).unwrap(),
                            },
                        );
                    }
                }
                spirv_reflect::types::descriptor::ReflectDescriptorType::CombinedImageSampler => {
                    if let Some(b) = bindings_map.get_mut(&binding.binding) {
                        if let gpu::DescriptorLayoutEntryType::CombinedTextureSampler = b.ty {
                            b.stage |= shader_stage
                        } else {
                            let n1 = binding.name.clone();
                            let n2 = descriptor_set_names
                                .iter()
                                .find(|(n, (s, b))| {
                                    set_idx == *s as u32
                                        && binding_idx == *b as u32
                                        && **n != binding.name
                                })
                                .unwrap()
                                .0
                                .clone();
                            return Err(error::ParseSpirvError::SetConflict(
                                set.set,
                                binding.binding,
                                n1,
                                n2,
                            )
                            .into());
                        }
                    } else {
                        bindings_map.insert(
                            binding.binding,
                            gpu::DescriptorLayoutEntry {
                                ty: gpu::DescriptorLayoutEntryType::CombinedTextureSampler,
                                stage: shader_stage,
                                count: NonZeroU32::new(binding.count).unwrap(),
                            },
                        );
                    }
                }
                n => panic!(
                    "Attempt to use descriptor type {:?}\nInvalid for GraphicsRenderer",
                    n
                ),
            }
        }
    }

    Ok(())
}

pub(crate) fn parse_push_constants(
    push_constants: &mut Vec<gpu::PushConstantRange>,
    push_constant_names: &mut HashMap<String, (u32, gpu::ShaderStages, TypeId)>,
    module: &spirv_reflect::ShaderModule,
    shader_stage: gpu::ShaderStages,
) -> Result<(), error::ParseSpirvError> {
    let mut constants = match module.enumerate_push_constant_blocks(None) {
        Ok(p) => p,
        Err(m) => return Err(error::ReflectError(m).into()),
    };

    for constant in &mut constants {
        push_constants.push(gpu::PushConstantRange {
            stage: shader_stage,
            offset: constant.offset,
            size: constant.size,
        });
        for member in &constant.members {
            if let Some(desc) = &member.type_description {
                let type_id = if desc.type_flags.contains(ReflectTypeFlags::FLOAT) {
                    parse_push_float(desc, member)
                } else if desc.type_flags.contains(ReflectTypeFlags::INT) {
                    if member.numeric.scalar.signedness == 0 {
                        parse_push_uint(desc, member)
                    } else {
                        parse_push_int(desc, member)
                    }
                } else if desc.type_flags.contains(ReflectTypeFlags::BOOL) {
                    parse_push_bool(desc, member)
                } else {
                    None
                };

                if let Some(ty) = type_id {
                    let (o, s, id) = push_constant_names.entry(member.name.clone()).or_insert((
                        member.offset,
                        gpu::ShaderStages::empty(),
                        ty,
                    ));

                    if *id != ty {
                        panic!("ERROR: Multiple push constant variables of different type share the same name {}", member.name);
                    } else if *o != member.offset {
                        panic!("ERROR: Multiple push constant variables of the same name name have different offsets {}", member.name);
                    } else {
                        *s |= shader_stage;
                    }
                } else {
                    #[cfg(feature = "logging")]
                    log::warn!("GFX: No rust type found that matches with push constant field with name {}", desc.struct_member_name);
                }
            }
        }
    }

    Ok(())
}

fn parse_push_uint(
    desc: &spirv_reflect::types::ReflectTypeDescription,
    member: &spirv_reflect::types::ReflectBlockVariable,
) -> Option<TypeId> {
    if desc.type_flags.contains(ReflectTypeFlags::ARRAY) {
        None
    } else if desc.type_flags.contains(ReflectTypeFlags::MATRIX) {
        assert_eq!(
            member.numeric.matrix.row_count, member.numeric.matrix.column_count,
            "ERROR: Only square matrices are supported at the moment found {}x{} matrix",
            member.numeric.matrix.row_count, member.numeric.matrix.column_count,
        );
        if member.numeric.scalar.width == 8 {
            if member.numeric.matrix.row_count == 2 {
                Some(TypeId::of::<[u8; 4]>())
            } else if member.numeric.matrix.column_count == 3 {
                Some(TypeId::of::<[u8; 9]>())
            } else if member.numeric.matrix.column_count == 4 {
                Some(TypeId::of::<[u8; 16]>())
            } else {
                panic!(
                    "ERROR: Matrices must be of dimension 2x2, 3x3 or 4x4 found {}x{}",
                    member.numeric.matrix.row_count, member.numeric.matrix.column_count
                )
            }
        } else if member.numeric.scalar.width == 16 {
            if member.numeric.matrix.row_count == 2 {
                Some(TypeId::of::<[u16; 4]>())
            } else if member.numeric.matrix.column_count == 3 {
                Some(TypeId::of::<[u16; 9]>())
            } else if member.numeric.matrix.column_count == 4 {
                Some(TypeId::of::<[u16; 16]>())
            } else {
                panic!(
                    "ERROR: Matrices must be of dimension 2x2, 3x3 or 4x4 found {}x{}",
                    member.numeric.matrix.row_count, member.numeric.matrix.column_count
                )
            }
        } else if member.numeric.scalar.width == 32 {
            if member.numeric.matrix.row_count == 2 {
                Some(TypeId::of::<[u32; 4]>())
            } else if member.numeric.matrix.column_count == 3 {
                Some(TypeId::of::<[u32; 9]>())
            } else if member.numeric.matrix.column_count == 4 {
                Some(TypeId::of::<[u32; 16]>())
            } else {
                panic!(
                    "ERROR: Matrices must be of dimension 2x2, 3x3 or 4x4 found {}x{}",
                    member.numeric.matrix.row_count, member.numeric.matrix.column_count
                )
            }
        } else if member.numeric.scalar.width == 64 {
            if member.numeric.matrix.row_count == 2 {
                Some(TypeId::of::<[u64; 4]>())
            } else if member.numeric.matrix.column_count == 3 {
                Some(TypeId::of::<[u64; 9]>())
            } else if member.numeric.matrix.column_count == 4 {
                Some(TypeId::of::<[u64; 16]>())
            } else {
                panic!(
                    "ERROR: Matrices must be of dimension 2x2, 3x3 or 4x4 found {}x{}",
                    member.numeric.matrix.row_count, member.numeric.matrix.column_count
                )
            }
        } else {
            None
        }
    } else if desc.type_flags.contains(ReflectTypeFlags::VECTOR) {
        if member.numeric.scalar.width == 8 {
            if member.numeric.vector.component_count == 2 {
                Some(TypeId::of::<[u8; 2]>())
            } else if member.numeric.vector.component_count == 3 {
                Some(TypeId::of::<[u8; 3]>())
            } else if member.numeric.vector.component_count == 4 {
                Some(TypeId::of::<[u8; 4]>())
            } else {
                panic!(
                    "ERROR: Vectors must have 2, 3, or 4 components found {}",
                    member.numeric.vector.component_count
                )
            }
        } else if member.numeric.scalar.width == 16 {
            if member.numeric.vector.component_count == 2 {
                Some(TypeId::of::<[u16; 2]>())
            } else if member.numeric.vector.component_count == 3 {
                Some(TypeId::of::<[u16; 3]>())
            } else if member.numeric.vector.component_count == 4 {
                Some(TypeId::of::<[u16; 4]>())
            } else {
                panic!(
                    "ERROR: Vectors must have 2, 3, or 4 components found {}",
                    member.numeric.vector.component_count
                )
            }
        } else if member.numeric.scalar.width == 32 {
            if member.numeric.vector.component_count == 2 {
                Some(TypeId::of::<[u32; 2]>())
            } else if member.numeric.vector.component_count == 3 {
                Some(TypeId::of::<[u32; 3]>())
            } else if member.numeric.vector.component_count == 4 {
                Some(TypeId::of::<[u32; 4]>())
            } else {
                panic!(
                    "ERROR: Vectors must have 2, 3, or 4 components found {}",
                    member.numeric.vector.component_count
                )
            }
        } else if member.numeric.scalar.width == 64 {
            if member.numeric.vector.component_count == 2 {
                Some(TypeId::of::<[u64; 2]>())
            } else if member.numeric.vector.component_count == 3 {
                Some(TypeId::of::<[u64; 3]>())
            } else if member.numeric.vector.component_count == 4 {
                Some(TypeId::of::<[u64; 4]>())
            } else {
                panic!(
                    "ERROR: Vectors must have 2, 3, or 4 components found {}",
                    member.numeric.vector.component_count
                )
            }
        } else {
            None
        }
    } else {
        if member.numeric.scalar.width == 8 {
            Some(TypeId::of::<u8>())
        } else if member.numeric.scalar.width == 16 {
            Some(TypeId::of::<u16>())
        } else if member.numeric.scalar.width == 32 {
            Some(TypeId::of::<u32>())
        } else if member.numeric.scalar.width == 64 {
            Some(TypeId::of::<u64>())
        } else {
            None
        }
    }
}

fn parse_push_int(
    desc: &spirv_reflect::types::ReflectTypeDescription,
    member: &spirv_reflect::types::ReflectBlockVariable,
) -> Option<TypeId> {
    if desc.type_flags.contains(ReflectTypeFlags::ARRAY) {
        None
    } else if desc.type_flags.contains(ReflectTypeFlags::MATRIX) {
        assert_eq!(
            member.numeric.matrix.row_count, member.numeric.matrix.column_count,
            "ERROR: Only square matrices are supported at the moment found {}x{} matrix",
            member.numeric.matrix.row_count, member.numeric.matrix.column_count,
        );
        if member.numeric.scalar.width == 8 {
            if member.numeric.matrix.row_count == 2 {
                Some(TypeId::of::<[i8; 4]>())
            } else if member.numeric.matrix.column_count == 3 {
                Some(TypeId::of::<[i8; 9]>())
            } else if member.numeric.matrix.column_count == 4 {
                Some(TypeId::of::<[i8; 16]>())
            } else {
                panic!(
                    "ERROR: Matrices must be of dimension 2x2, 3x3 or 4x4 found {}x{}",
                    member.numeric.matrix.row_count, member.numeric.matrix.column_count
                )
            }
        } else if member.numeric.scalar.width == 16 {
            if member.numeric.matrix.row_count == 2 {
                Some(TypeId::of::<[i16; 4]>())
            } else if member.numeric.matrix.column_count == 3 {
                Some(TypeId::of::<[i16; 9]>())
            } else if member.numeric.matrix.column_count == 4 {
                Some(TypeId::of::<[i16; 16]>())
            } else {
                panic!(
                    "ERROR: Matrices must be of dimension 2x2, 3x3 or 4x4 found {}x{}",
                    member.numeric.matrix.row_count, member.numeric.matrix.column_count
                )
            }
        } else if member.numeric.scalar.width == 32 {
            if member.numeric.matrix.row_count == 2 {
                Some(TypeId::of::<[i32; 4]>())
            } else if member.numeric.matrix.column_count == 3 {
                Some(TypeId::of::<[i32; 9]>())
            } else if member.numeric.matrix.column_count == 4 {
                Some(TypeId::of::<[i32; 16]>())
            } else {
                panic!(
                    "ERROR: Matrices must be of dimension 2x2, 3x3 or 4x4 found {}x{}",
                    member.numeric.matrix.row_count, member.numeric.matrix.column_count
                )
            }
        } else if member.numeric.scalar.width == 64 {
            if member.numeric.matrix.row_count == 2 {
                Some(TypeId::of::<[i64; 4]>())
            } else if member.numeric.matrix.column_count == 3 {
                Some(TypeId::of::<[i64; 9]>())
            } else if member.numeric.matrix.column_count == 4 {
                Some(TypeId::of::<[i64; 16]>())
            } else {
                panic!(
                    "ERROR: Matrices must be of dimension 2x2, 3x3 or 4x4 found {}x{}",
                    member.numeric.matrix.row_count, member.numeric.matrix.column_count
                )
            }
        } else {
            None
        }
    } else if desc.type_flags.contains(ReflectTypeFlags::VECTOR) {
        if member.numeric.scalar.width == 8 {
            if member.numeric.vector.component_count == 2 {
                Some(TypeId::of::<[i8; 2]>())
            } else if member.numeric.vector.component_count == 3 {
                Some(TypeId::of::<[i8; 3]>())
            } else if member.numeric.vector.component_count == 4 {
                Some(TypeId::of::<[i8; 4]>())
            } else {
                panic!(
                    "ERROR: Vectors must have 2, 3, or 4 components found {}",
                    member.numeric.vector.component_count
                )
            }
        } else if member.numeric.scalar.width == 16 {
            if member.numeric.vector.component_count == 2 {
                Some(TypeId::of::<[i16; 2]>())
            } else if member.numeric.vector.component_count == 3 {
                Some(TypeId::of::<[i16; 3]>())
            } else if member.numeric.vector.component_count == 4 {
                Some(TypeId::of::<[i16; 4]>())
            } else {
                panic!(
                    "ERROR: Vectors must have 2, 3, or 4 components found {}",
                    member.numeric.vector.component_count
                )
            }
        } else if member.numeric.scalar.width == 32 {
            if member.numeric.vector.component_count == 2 {
                Some(TypeId::of::<[i32; 2]>())
            } else if member.numeric.vector.component_count == 3 {
                Some(TypeId::of::<[i32; 3]>())
            } else if member.numeric.vector.component_count == 4 {
                Some(TypeId::of::<[i32; 4]>())
            } else {
                panic!(
                    "ERROR: Vectors must have 2, 3, or 4 components found {}",
                    member.numeric.vector.component_count
                )
            }
        } else if member.numeric.scalar.width == 64 {
            if member.numeric.vector.component_count == 2 {
                Some(TypeId::of::<[i64; 2]>())
            } else if member.numeric.vector.component_count == 3 {
                Some(TypeId::of::<[i64; 3]>())
            } else if member.numeric.vector.component_count == 4 {
                Some(TypeId::of::<[i64; 4]>())
            } else {
                panic!(
                    "ERROR: Vectors must have 2, 3, or 4 components found {}",
                    member.numeric.vector.component_count
                )
            }
        } else {
            None
        }
    } else {
        if member.numeric.scalar.width == 8 {
            Some(TypeId::of::<i8>())
        } else if member.numeric.scalar.width == 16 {
            Some(TypeId::of::<i16>())
        } else if member.numeric.scalar.width == 32 {
            Some(TypeId::of::<i32>())
        } else if member.numeric.scalar.width == 64 {
            Some(TypeId::of::<i64>())
        } else {
            None
        }
    }
}

fn parse_push_float(
    desc: &spirv_reflect::types::ReflectTypeDescription,
    member: &spirv_reflect::types::ReflectBlockVariable,
) -> Option<TypeId> {
    if desc.type_flags.contains(ReflectTypeFlags::ARRAY) {
        None
    } else if desc.type_flags.contains(ReflectTypeFlags::MATRIX) {
        assert_eq!(
            member.numeric.matrix.row_count, member.numeric.matrix.column_count,
            "ERROR: Only square matrices are supported at the moment found {}x{} matrix",
            member.numeric.matrix.row_count, member.numeric.matrix.column_count,
        );
        if member.numeric.scalar.width == 32 {
            if member.numeric.matrix.row_count == 2 {
                Some(TypeId::of::<[f32; 4]>())
            } else if member.numeric.matrix.column_count == 3 {
                Some(TypeId::of::<[f32; 9]>())
            } else if member.numeric.matrix.column_count == 4 {
                Some(TypeId::of::<[f32; 16]>())
            } else {
                panic!(
                    "ERROR: Matrices must be of dimension 2x2, 3x3 or 4x4 found {}x{}",
                    member.numeric.matrix.row_count, member.numeric.matrix.column_count
                )
            }
        } else if member.numeric.scalar.width == 64 {
            if member.numeric.matrix.row_count == 2 {
                Some(TypeId::of::<[f64; 4]>())
            } else if member.numeric.matrix.column_count == 3 {
                Some(TypeId::of::<[f64; 9]>())
            } else if member.numeric.matrix.column_count == 4 {
                Some(TypeId::of::<[f64; 16]>())
            } else {
                panic!(
                    "ERROR: Matrices must be of dimension 2x2, 3x3 or 4x4 found {}x{}",
                    member.numeric.matrix.row_count, member.numeric.matrix.column_count
                )
            }
        } else {
            None
        }
    } else if desc.type_flags.contains(ReflectTypeFlags::VECTOR) {
        if member.numeric.scalar.width == 32 {
            if member.numeric.vector.component_count == 2 {
                Some(TypeId::of::<[f32; 2]>())
            } else if member.numeric.vector.component_count == 3 {
                Some(TypeId::of::<[f32; 3]>())
            } else if member.numeric.vector.component_count == 4 {
                Some(TypeId::of::<[f32; 4]>())
            } else {
                panic!(
                    "ERROR: Vectors must have 2, 3, or 4 components found {}",
                    member.numeric.vector.component_count
                )
            }
        } else if member.numeric.scalar.width == 64 {
            if member.numeric.vector.component_count == 2 {
                Some(TypeId::of::<[f64; 2]>())
            } else if member.numeric.vector.component_count == 3 {
                Some(TypeId::of::<[f64; 3]>())
            } else if member.numeric.vector.component_count == 4 {
                Some(TypeId::of::<[f64; 4]>())
            } else {
                panic!(
                    "ERROR: Vectors must have 2, 3, or 4 components found {}",
                    member.numeric.vector.component_count
                )
            }
        } else {
            None
        }
    } else {
        if member.numeric.scalar.width == 32 {
            Some(TypeId::of::<f32>())
        } else if member.numeric.scalar.width == 64 {
            Some(TypeId::of::<f64>())
        } else {
            None
        }
    }
}

fn parse_push_bool(
    desc: &spirv_reflect::types::ReflectTypeDescription,
    member: &spirv_reflect::types::ReflectBlockVariable,
) -> Option<TypeId> {
    if desc.type_flags.contains(ReflectTypeFlags::ARRAY) {
        None
    } else if desc.type_flags.contains(ReflectTypeFlags::MATRIX) {
        assert_eq!(
            member.numeric.matrix.row_count, member.numeric.matrix.column_count,
            "ERROR: Only square matrices are supported at the moment found {}x{} matrix",
            member.numeric.matrix.row_count, member.numeric.matrix.column_count,
        );
        if member.numeric.matrix.row_count == 2 {
            Some(TypeId::of::<[bool; 4]>())
        } else if member.numeric.matrix.column_count == 3 {
            Some(TypeId::of::<[bool; 9]>())
        } else if member.numeric.matrix.column_count == 4 {
            Some(TypeId::of::<[bool; 16]>())
        } else {
            panic!(
                "ERROR: Matrices must be of dimension 2x2, 3x3 or 4x4 found {}x{}",
                member.numeric.matrix.row_count, member.numeric.matrix.column_count
            )
        }
    } else if desc.type_flags.contains(ReflectTypeFlags::VECTOR) {
        if member.numeric.vector.component_count == 2 {
            Some(TypeId::of::<[bool; 2]>())
        } else if member.numeric.vector.component_count == 3 {
            Some(TypeId::of::<[bool; 3]>())
        } else if member.numeric.vector.component_count == 4 {
            Some(TypeId::of::<[bool; 4]>())
        } else {
            panic!(
                "ERROR: Vectors must have 2, 3, or 4 components found {}",
                member.numeric.vector.component_count
            )
        }
    } else {
        Some(TypeId::of::<bool>())
    }
}
