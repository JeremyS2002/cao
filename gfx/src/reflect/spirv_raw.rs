use std::{any::TypeId, collections::HashMap};

use super::error;

use either::*;

pub(crate) fn parse_vertex_states(
    vertex: &spv::Builder,
) -> std::sync::Arc<[super::graphics::VertexLocationInfo]> {
    vertex
        .get_inputs()
        .iter()
        .filter_map(|input| {
            if let Left(_) = input.location {
                Some(super::graphics::VertexLocationInfo {
                    name: input.name.unwrap().to_string(),
                    format: match input.ty {
                        spv::IOType::Float => gpu::VertexFormat::Float,
                        spv::IOType::Vec2 => gpu::VertexFormat::Vec2,
                        spv::IOType::Vec3 => gpu::VertexFormat::Vec3,
                        spv::IOType::Vec4 => gpu::VertexFormat::Vec4,
                        _ => panic!("ERROR: Cannot have input of type {:?} in vertex shader", input.ty),
                    },
                })
            } else {
                None
            }
        })
        .collect()
}

// Same as super::reflect_raw::combine_descriptor_set_layouts
// TODO move into common module between spirv_raw and reflect_raw
pub(crate) fn combine_descriptor_set_layouts(
    device: &gpu::Device,
    descriptor_set_layouts: HashMap<u32, HashMap<u32, gpu::DescriptorLayoutEntry>>,
    name: Option<&str>,
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

pub(crate) fn process_shader(
    builder: &spv::Builder,
    stage: spv::Stage,
    descriptor_set_layouts: &mut HashMap<u32, HashMap<u32, gpu::DescriptorLayoutEntry>>,
    descriptor_set_names: &mut HashMap<String, (usize, usize)>,
    push_constants: &mut Vec<gpu::PushConstantRange>,
    push_constant_names: &mut HashMap<String, (u32, gpu::ShaderStages, std::any::TypeId)>,
) {
    let stages = match stage {
        spv::Stage::Vertex => gpu::ShaderStages::VERTEX,
        spv::Stage::TessellationEval => gpu::ShaderStages::TESSELLATION_EVAL,
        spv::Stage::TessellationControl => gpu::ShaderStages::TESSELLATION_CONTROL,
        spv::Stage::Geometry => gpu::ShaderStages::GEOMETRY,
        spv::Stage::Fragment => gpu::ShaderStages::FRAGMENT,
        spv::Stage::Compute => gpu::ShaderStages::COMPUTE,
    };

    let mut spv_descriptor_set_layouts = HashMap::new();

    for uniform in builder.get_uniforms() {
        let e = spv_descriptor_set_layouts.entry(
            (uniform.set, uniform.binding)
        ).or_insert((gpu::DescriptorLayoutEntry {
            ty: gpu::DescriptorLayoutEntryType::UniformBuffer,
            stage: stages,
            count: std::num::NonZeroU32::new(1).unwrap(),
        }, uniform.name));
        e.0.stage |= stages;
    }

    for storage in builder.get_storages() {
        let e = spv_descriptor_set_layouts.entry(
            (storage.set, storage.binding)
        ).or_insert((gpu::DescriptorLayoutEntry {
            ty: gpu::DescriptorLayoutEntryType::StorageBuffer { read_only: !storage.write },
            stage: stages,
            count: std::num::NonZeroU32::new(1).unwrap(),
        }, storage.name));
        e.0.stage |= stages;
    }

    for texture in builder.get_textures() {
        let e = spv_descriptor_set_layouts.entry(
            (texture.set, texture.binding)
        ).or_insert((gpu::DescriptorLayoutEntry {
            ty: gpu::DescriptorLayoutEntryType::SampledTexture,
            stage: stages,
            count: std::num::NonZeroU32::new(1).unwrap(),
        }, texture.name));
        e.0.stage |= stages;
    }

    for sampled_texture in builder.get_sampled_textures() {
        let e = spv_descriptor_set_layouts.entry(
            (sampled_texture.set, sampled_texture.binding)
        ).or_insert((gpu::DescriptorLayoutEntry {
            ty: gpu::DescriptorLayoutEntryType::CombinedTextureSampler,
            stage: stages,
            count: std::num::NonZeroU32::new(1).unwrap(),
        }, sampled_texture.name));
        e.0.stage |= stages;
    }

    for sampler in builder.get_samplers() {
        let e = spv_descriptor_set_layouts.entry(
            (sampler.set, sampler.binding)
        ).or_insert((gpu::DescriptorLayoutEntry {
            ty: gpu::DescriptorLayoutEntryType::Sampler,
            stage: stages,
            count: std::num::NonZeroU32::new(1).unwrap(),
        }, sampler.name));
        e.0.stage |= stages;
    }

    for ((set, binding), (v, n)) in spv_descriptor_set_layouts {
        let binding_map = if let Some(binding_map) = descriptor_set_layouts.get_mut(&set) {
            binding_map
        } else {
            descriptor_set_layouts.insert(set, HashMap::new());
            descriptor_set_layouts.get_mut(&set).unwrap()
        };
        binding_map.insert(binding, v);
        if let Some(n) = n {
            descriptor_set_names.insert(n.to_string(), (set as _, binding as _));
        }
    }

    if let Some(p) = builder.get_push_constants() {
        let stages = match stage {
            spv::Stage::Vertex => gpu::ShaderStages::VERTEX,
            spv::Stage::TessellationEval => gpu::ShaderStages::TESSELLATION_EVAL,
            spv::Stage::TessellationControl => gpu::ShaderStages::TESSELLATION_CONTROL,
            spv::Stage::Geometry => gpu::ShaderStages::GEOMETRY,
            spv::Stage::Fragment => gpu::ShaderStages::FRAGMENT,
            spv::Stage::Compute => gpu::ShaderStages::COMPUTE,
        };

        let new = gpu::PushConstantRange {
            stage: stages,
            offset: 0,
            size: p.ty.size().expect("ERROR Push constants must be sized"),
        };

        push_constants.push(new);

        process_push_constant_ty(&p.ty, p.name, push_constant_names, 0, stages);
    }

    //push_constant_names.insert(name, ())
}

fn process_push_constant_ty(
    ty: &spv::Type,
    name: Option<&str>,
    push_constant_names: &mut HashMap<String, (u32, gpu::ShaderStages, TypeId)>,
    mut offset: u32,
    stage: gpu::ShaderStages,
) {
    let id = |p: spv::Type| match p {
        spv::Type::Void => TypeId::of::<()>(),
        spv::Type::Scalar(s) => match s {
            spv::ScalarType::Bool => TypeId::of::<bool>(),
            spv::ScalarType::Signed(c) => match c {
                8 => TypeId::of::<i8>(),
                16 => TypeId::of::<i16>(),
                32 => TypeId::of::<i32>(),
                64 => TypeId::of::<i64>(),
                c => panic!("unsupported bit count in shader {}", c)
            },
            spv::ScalarType::Unsigned(c) => match c {
                8 => TypeId::of::<u8>(),
                16 => TypeId::of::<u16>(),
                32 => TypeId::of::<u32>(),
                64 => TypeId::of::<u64>(),
                c => panic!("unsupported bit count in shader {}", c)
            },
            spv::ScalarType::Float(c) => match c {
                32 => TypeId::of::<f32>(),
                64 => TypeId::of::<f64>(),
                c => panic!("unsupported bit count in shader {}", c)
            },
        },
        spv::Type::Vector(v) => match v.n_scalar {
            2 => match v.scalar_ty {
                spv::ScalarType::Bool => TypeId::of::<[bool; 2]>(),
                spv::ScalarType::Signed(c) => match c {
                    8 => TypeId::of::<[i8; 2]>(),
                    16 => TypeId::of::<[i16; 2]>(),
                    32 => TypeId::of::<[i32; 2]>(),
                    64 => TypeId::of::<[i64; 2]>(),
                    c => panic!("unsupported bit count in shader {}", c)
                },
                spv::ScalarType::Unsigned(c) => match c {
                    8 => TypeId::of::<[u8; 2]>(),
                    16 => TypeId::of::<[u16; 2]>(),
                    32 => TypeId::of::<[u32; 2]>(),
                    64 => TypeId::of::<[u64; 2]>(),
                    c => panic!("unsupported bit count in shader {}", c)
                },
                spv::ScalarType::Float(c) => match c {
                    32 => TypeId::of::<[f32; 2]>(),
                    64 => TypeId::of::<[f64; 2]>(),
                    c => panic!("unsupported bit count in shader {}", c)
                },
            },
            3 => match v.scalar_ty {
                spv::ScalarType::Bool => TypeId::of::<[bool; 3]>(),
                spv::ScalarType::Signed(c) => match c {
                    8 => TypeId::of::<[i8; 3]>(),
                    16 => TypeId::of::<[i16; 3]>(),
                    32 => TypeId::of::<[i32; 3]>(),
                    64 => TypeId::of::<[i64; 3]>(),
                    c => panic!("unsupported bit count in shader {}", c)
                },
                spv::ScalarType::Unsigned(c) => match c {
                    8 => TypeId::of::<[u8; 3]>(),
                    16 => TypeId::of::<[u16; 3]>(),
                    32 => TypeId::of::<[u32; 3]>(),
                    64 => TypeId::of::<[u64; 3]>(),
                    c => panic!("unsupported bit count in shader {}", c)
                },
                spv::ScalarType::Float(c) => match c {
                    32 => TypeId::of::<[f32; 3]>(),
                    64 => TypeId::of::<[f64; 3]>(),
                    c => panic!("unsupported bit count in shader {}", c)
                },
            },
            4 => match v.scalar_ty {
                spv::ScalarType::Bool => TypeId::of::<[bool; 4]>(),
                spv::ScalarType::Signed(c) => match c {
                    8 => TypeId::of::<[i8; 4]>(),
                    16 => TypeId::of::<[i16; 4]>(),
                    32 => TypeId::of::<[i32; 4]>(),
                    64 => TypeId::of::<[i64; 4]>(),
                    c => panic!("unsupported bit count in shader {}", c)
                },
                spv::ScalarType::Unsigned(c) => match c {
                    8 => TypeId::of::<[u8; 4]>(),
                    16 => TypeId::of::<[u16; 4]>(),
                    32 => TypeId::of::<[u32; 4]>(),
                    64 => TypeId::of::<[u64; 4]>(),
                    c => panic!("unsupported bit count in shader {}", c)
                },
                spv::ScalarType::Float(c) => match c {
                    32 => TypeId::of::<[f32; 4]>(),
                    64 => TypeId::of::<[f64; 4]>(),
                    c => panic!("unsupported bit count in shader {}", c)
                },
            }
            _ => panic!("unsupported vector size in push constant"),
        },
        spv::Type::Matrix(m) => {
            assert_eq!(m.n_vec, m.vec_ty.n_scalar, "ERROR only square matrices are supported in push constant blocks of shaders at the moment");
            match m.n_vec {
                2 => match m.vec_ty.scalar_ty {
                    spv::ScalarType::Bool => TypeId::of::<[[bool; 2]; 2]>(),
                    spv::ScalarType::Signed(c) => match c {
                        8 => TypeId::of::<[[i8; 2]; 2]>(),
                        16 => TypeId::of::<[[i16; 2]; 2]>(),
                        32 => TypeId::of::<[[i32; 2]; 2]>(),
                        64 => TypeId::of::<[[i64; 2]; 2]>(),
                        c => panic!("unsupported bit count in shader {}", c)
                    },
                    spv::ScalarType::Unsigned(c) => match c {
                        8 => TypeId::of::<[[u8; 2]; 2]>(),
                        16 => TypeId::of::<[[u16; 2]; 2]>(),
                        32 => TypeId::of::<[[u32; 2]; 2]>(),
                        64 => TypeId::of::<[[u64; 2]; 2]>(),
                        c => panic!("unsupported bit count in shader {}", c)
                    },
                    spv::ScalarType::Float(c) => match c {
                        32 => TypeId::of::<[[f32; 2]; 2]>(),
                        64 => TypeId::of::<[[f64; 2]; 2]>(),
                        c => panic!("unsupported bit count in shader {}", c)
                    },
                },
                3 => match m.vec_ty.scalar_ty {
                    spv::ScalarType::Bool => TypeId::of::<[[bool; 3]; 3]>(),
                    spv::ScalarType::Signed(c) => match c {
                        8 => TypeId::of::<[[i8; 3]; 3]>(),
                        16 => TypeId::of::<[[i16; 3]; 3]>(),
                        32 => TypeId::of::<[[i32; 3]; 3]>(),
                        64 => TypeId::of::<[[i64; 3]; 3]>(),
                        c => panic!("unsupported bit count in shader {}", c)
                    },
                    spv::ScalarType::Unsigned(c) => match c {
                        8 => TypeId::of::<[[u8; 3]; 3]>(),
                        16 => TypeId::of::<[[u16; 3]; 3]>(),
                        32 => TypeId::of::<[[u32; 3]; 3]>(),
                        64 => TypeId::of::<[[u64; 3]; 3]>(),
                        c => panic!("unsupported bit count in shader {}", c)
                    },
                    spv::ScalarType::Float(c) => match c {
                        32 => TypeId::of::<[[f32; 3]; 3]>(),
                        64 => TypeId::of::<[[f64; 3]; 3]>(),
                        c => panic!("unsupported bit count in shader {}", c)
                    },
                },
                4 => match m.vec_ty.scalar_ty {
                    spv::ScalarType::Bool => TypeId::of::<[[bool; 4]; 4]>(),
                    spv::ScalarType::Signed(c) => match c {
                        8 => TypeId::of::<[[i8; 4]; 4]>(),
                        16 => TypeId::of::<[[i16; 4]; 4]>(),
                        32 => TypeId::of::<[[i32; 4]; 4]>(),
                        64 => TypeId::of::<[[i64; 4]; 4]>(),
                        c => panic!("unsupported bit count in shader {}", c)
                    },
                    spv::ScalarType::Unsigned(c) => match c {
                        8 => TypeId::of::<[[u8; 4]; 4]>(),
                        16 => TypeId::of::<[[u16; 4]; 4]>(),
                        32 => TypeId::of::<[[u32; 4]; 4]>(),
                        64 => TypeId::of::<[[u64; 4]; 4]>(),
                        c => panic!("unsupported bit count in shader {}", c)
                    },
                    spv::ScalarType::Float(c) => match c {
                        32 => TypeId::of::<[[f32; 4]; 4]>(),
                        64 => TypeId::of::<[[f64; 4]; 4]>(),
                        c => panic!("unsupported bit count in shader {}", c)
                    },
                },
                _ => panic!("unsupported matrix size in push constant")
            }
        },
        spv::Type::Array(_) => unimplemented!(),
        spv::Type::Struct(_) => unimplemented!(),
        _ => unimplemented!()
    };

    let set_stages_or_panic = |id: TypeId,
                               e: &mut (u32, gpu::ShaderStages, TypeId),
                               name: &str,
                               offset: u32,
                               stage: gpu::ShaderStages| {
        if id != e.2 {
            panic!(
                "ERROR: Multiple push constant variables of different type share the same name {}",
                name
            );
        } else if e.0 != offset {
            panic!("ERROR: Multiple push constant variables of the same name name have different offsets {}", name);
        } else {
            e.1 |= stage;
        }
    };

    match ty {
        spv::Type::Scalar(_) => {
            if let Some(name) = name {
                let id = id(ty.clone());
                let e = push_constant_names.entry(name.to_string()).or_insert((
                    0,
                    gpu::ShaderStages::empty(),
                    id,
                ));
                set_stages_or_panic(id, e, name, offset, stage);
            }
        }
        spv::Type::Vector(_) => {
            if let Some(name) = name {
                let id = id(ty.clone());
                let e = push_constant_names.entry(name.to_string()).or_insert((
                    0,
                    gpu::ShaderStages::empty(),
                    id,
                ));
                set_stages_or_panic(id, e, name, offset, stage);
            }
        },
        spv::Type::Matrix(_) => {
            if let Some(name) = name {
                let id = id(ty.clone());
                let e = push_constant_names.entry(name.to_string()).or_insert((
                    0,
                    gpu::ShaderStages::empty(),
                    id,
                ));
                set_stages_or_panic(id, e, name, offset, stage);
            }
        }
        spv::Type::Array(ty) => {
            if let Some(name) = name {
                // kinda not great but arrays have to be set index by index or manually
                // without diving into how TypeId works and forcing a creation of TypeId::of::<[p; n]>() (<- doesn't work as n isn't const)
                // (technically could match on every n and code the relation but just no)
                let element_ty = match &ty.element_ty {
                    Left(t) => (*t).clone(),
                    Right(t) => (**t).clone(),
                };
                let size = element_ty.size().expect("ERROR push constant types must be sized");
                let id = id(element_ty);
                for i in 0..ty.length.expect("ERROR Arrays in push constants must have known length") {
                    let e = push_constant_names
                        .entry(format!("{}[{}]", name, i))
                        .or_insert((offset, gpu::ShaderStages::empty(), id));
                    offset += size;
                    set_stages_or_panic(id, e, name, offset, stage);
                }
            }
        }
        spv::Type::Struct(ty) => {
            // if let Some(name) = ty.name {
            //     let name = match name {
            //         Left(n) => n,
            //         Right(n) => &*n,
            //     };
            //     let e = push_constant_names.entry(name.to_string()).or_insert((
            //         offset,
            //         gpu::ShaderStages::empty(),
            //         *ty_id,
            //     ));
            // }
            
            // set_stages_or_panic(*ty_id, e, name, offset, stage);

            for member in &*ty.members {
                let name = member.name.as_ref().map(|n| match n {
                    Left(n) => *n,
                    Right(n) => &**n,
                });
                process_push_constant_ty(&member.ty, name, push_constant_names, offset, stage);
                offset += member.ty.size().expect("ERROR: Unsized types in push constant blocks arn't allowed");
            }
        },
        _ => unreachable!(),
    }
}

pub(crate) fn check_stage_compatibility(
    src: &spv::Builder,
    src_stage: spv::Stage,
    dst: &spv::Builder,
    dst_stage: spv::Stage,
) -> Result<(), error::BuilderConfigError> {
    let src_o = src.get_outputs();

    let sort_fn = |a: &spv::IOData, b: &spv::IOData| match a.location {
        Left(a) => match b.location {
            Left(b) => a.cmp(&b),
            Right(_) => std::cmp::Ordering::Less,
        },
        Right(_) => match b.location {
            Left(_) => std::cmp::Ordering::Greater,
            Right(_) => std::cmp::Ordering::Equal,
        },
    };

    let filter_fn = |a: &spv::IOData| match a.location {
        Left(l) => Some((a.ty, l)),
        Right(_) => None,
    };

    let mut outputs = src_o;
    outputs.sort_unstable_by(sort_fn);
    let outputs = outputs
        .iter()
        .filter_map(filter_fn)
        .collect::<Vec<_>>();

    let mut inputs = dst.get_inputs();
    inputs.sort_unstable_by(sort_fn);
    let inputs = inputs
        .iter()
        .filter_map(filter_fn)
        .collect::<Vec<_>>();

    for input in inputs.iter() {
        // I don't know why I did this when they are already sorted by location but don't want to change it
        // TODO test this
        if let Some(output) = outputs.iter().find(|v| v.1 == input.1) {
            if output.0 != input.0 {
                Err(error::BuilderConfigError::StageIncompatibility {
                    location: output.1,
                    src_stage_name: format!("{:?}", src_stage),
                    src_type: output.0.ty(),
                    dst_stage_name: format!("{:?}", dst_stage),
                    dst_type: input.0.ty(),
                })?;
            }
        }
    }

    Ok(())
}
