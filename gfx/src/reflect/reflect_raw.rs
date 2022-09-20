
use std::any::TypeId;
use std::collections::HashMap;

use super::error;

pub(crate) fn parse_vertex_states(
    vertex: &[u32],
) -> Result<Vec<super::graphics::VertexLocationInfo>, error::ParseSpirvError> {
    let entry_points = spirq::ReflectConfig::new()
        .spv(vertex)
        .ref_all_rscs(true)
        .reflect()
        .unwrap();

    let mut info = Vec::new();

    for entry in entry_points {
        if entry.exec_model != spirq::ExecutionModel::Vertex {
            continue;
        }

        for var in entry.vars {
            if let spirq::Variable::Input { name, location, ty } = var {
                if let Some(name) = name {
                    info.push((location.loc(), super::graphics::VertexLocationInfo {
                        name,
                        format: match ty {
                            spirq::ty::Type::Scalar(s) => match s {
                                spirq::ty::ScalarType::Float(c) => match c {
                                    4 => gpu::VertexFormat::Float,
                                    _ => unimplemented!(),
                                },
                                _ => unimplemented!(),
                            },
                            spirq::ty::Type::Vector(v) => match v.nscalar {
                                2 => match v.scalar_ty {
                                    spirq::ty::ScalarType::Float(c) => match c {
                                        4 => gpu::VertexFormat::Vec2,
                                        _ => unimplemented!()
                                    },
                                    _ => unimplemented!(),
                                },
                                3 => match v.scalar_ty {
                                    spirq::ty::ScalarType::Float(c) => match c {
                                        4 => gpu::VertexFormat::Vec3,
                                        _ => unimplemented!()
                                    },
                                    _ => unimplemented!(),
                                },
                                4 => match v.scalar_ty {
                                    spirq::ty::ScalarType::Float(c) => match c {
                                        4 => gpu::VertexFormat::Vec4,
                                        _ => unimplemented!()
                                    },
                                    _ => unimplemented!(),
                                },
                                _ => unreachable!(),
                            },
                            spirq::ty::Type::Matrix(_) => unimplemented!(),
                            _ => unreachable!(),
                        }
                    }))
                }
            }
        }
    }

    info.sort_unstable_by(|a, b| a.0.cmp(&b.0));

    Ok(info.into_iter().map(|i| i.1).collect::<Vec<_>>())
}

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

pub(crate) fn parse_spirv(
    descriptor_sets: &mut HashMap<u32, HashMap<u32, gpu::DescriptorLayoutEntry>>,
    descriptor_set_names: &mut HashMap<String, (usize, usize)>,
    push_constants: &mut Vec<gpu::PushConstantRange>,
    push_constant_names: &mut HashMap<String, (u32, gpu::ShaderStages, TypeId)>,
    specialization_names: &mut HashMap<String, (TypeId, Vec<(u32, gpu::ShaderStages)>)>,
    spirv: &[u32],
    execution: spirq::ExecutionModel,
) -> Result<String, error::ParseSpirvError> {
    let stage = match execution {
        spirq::ExecutionModel::Vertex => gpu::ShaderStages::VERTEX,
        spirq::ExecutionModel::TessellationControl => gpu::ShaderStages::TESSELLATION_CONTROL,
        spirq::ExecutionModel::TessellationEvaluation => gpu::ShaderStages::TESSELLATION_EVAL,
        spirq::ExecutionModel::Geometry => gpu::ShaderStages::GEOMETRY,
        spirq::ExecutionModel::Fragment => gpu::ShaderStages::FRAGMENT,
        spirq::ExecutionModel::GLCompute => gpu::ShaderStages::COMPUTE,
        _ => unimplemented!(),
    };

    let mut name = None;

    let entry_points = spirq::ReflectConfig::new()
        .spv(spirv)
        .ref_all_rscs(true)
        .reflect()
        .unwrap();
    for entry in entry_points {
        if entry.exec_model != execution {
            continue;
        }

        name = Some(entry.name);

        for var in entry.vars {
            match var {
                spirq::Variable::Input { .. } => (), // do in check stage compatibility
                spirq::Variable::Output { .. } => (), // do in check stage compatibility
                spirq::Variable::Descriptor { 
                    name, 
                    desc_bind, 
                    desc_ty, 
                    nbind ,
                    ..
                } => {
                    let set = desc_bind.set();
                    let bind = desc_bind.bind();

                    if let Some(name) = name {
                        let prev = descriptor_set_names.insert(name.clone(), (set as _, bind as _));
                        if let Some((pset, pbind)) = prev {
                            if pset != set as _ || pbind != bind as _ {
                                return Err(error::ParseSpirvError::DescriptorNameUndecidable(name, set as _, bind as _, pset, pbind));
                            }
                        }   
                    }

                    let gpu_ty = match desc_ty {
                        spirq::DescriptorType::Sampler() => gpu::DescriptorLayoutEntryType::Sampler,
                        spirq::DescriptorType::CombinedImageSampler() => gpu::DescriptorLayoutEntryType::CombinedTextureSampler,
                        spirq::DescriptorType::SampledImage() => gpu::DescriptorLayoutEntryType::SampledTexture,
                        spirq::DescriptorType::StorageImage(a) => gpu::DescriptorLayoutEntryType::StorageTexture { read_only: a == spirq::AccessType::ReadOnly },
                        spirq::DescriptorType::UniformTexelBuffer() => gpu::DescriptorLayoutEntryType::UniformBuffer,
                        spirq::DescriptorType::UniformBuffer() => gpu::DescriptorLayoutEntryType::UniformBuffer,
                        spirq::DescriptorType::StorageBuffer(a) => gpu::DescriptorLayoutEntryType::StorageBuffer { read_only: a == spirq::AccessType::ReadOnly },
                        t => unimplemented!("Descriptor type {:?} not supported at the moment", t),
                    };

                    let map = descriptor_sets.entry(set).or_insert(HashMap::new());
                    let e = map.entry(bind).or_insert(gpu::DescriptorLayoutEntry {
                        ty: gpu_ty,
                        stage,
                        count: std::num::NonZeroU32::new(nbind).unwrap(),
                    });
                    e.stage |= stage;
                    if e.ty != gpu_ty {
                        return Err(error::ParseSpirvError::DescriptorTypeConflict(set, bind, e.ty, gpu_ty))
                    }
                },
                spirq::Variable::PushConstant { 
                    ty ,
                    ..
                } => {
                    // push constants are stored as structs idk what to do if they aren't 
                    if let spirq::ty::Type::Struct(s) = ty {
                        push_constants.push(gpu::PushConstantRange {
                            stage,
                            offset: 0,
                            size: s.nbyte() as _,
                        });

                        for member in s.members {
                            if let Some(n) = member.name {
                                let ty_id = get_type_id(member.ty);
                                let prev = push_constant_names.entry(n.clone()).or_insert((member.offset as _, stage, ty_id));
                                prev.1 |= stage;
                                if prev.0 != member.offset as _ || prev.2 != ty_id {
                                    return Err(error::ParseSpirvError::PushNameConflict(n, member.offset as _, ty_id, prev.0, prev.2))
                                }
                            }
                        }
                    } else {
                        // please nobody ever see this
                        eprintln!("Good luck :)");
                    }
                },
                spirq::Variable::SpecConstant { name, spec_id, ty  } => {
                    if let Some(name) = name {
                        let ty_id = get_type_id(ty);
                        let e = specialization_names.entry(name.clone()).or_insert((ty_id, vec![(spec_id, stage)]));
                        if e.0 != ty_id {
                            return Err(error::ParseSpirvError::ConstantNameConflict(name, ty_id, e.0));
                        }
                    }
                },
            }
        }
    }

    Ok(name.unwrap())
}

fn get_type_id(ty: spirq::ty::Type) -> TypeId {
    match ty {
        spirq::ty::Type::Void() => TypeId::of::<()>(),
        spirq::ty::Type::Scalar(s) => match s {
            spirq::ty::ScalarType::Boolean => TypeId::of::<bool>(),
            spirq::ty::ScalarType::Signed(c) => match c {
                1 => TypeId::of::<i8>(),
                2 => TypeId::of::<i16>(),
                4 => TypeId::of::<i32>(),
                8 => TypeId::of::<i64>(),
                c => panic!("unsupported bit count in shader {}", c)
            },
            spirq::ty::ScalarType::Unsigned(c) => match c {
                1 => TypeId::of::<u8>(),
                2 => TypeId::of::<u16>(),
                4 => TypeId::of::<u32>(),
                8 => TypeId::of::<u64>(),
                c => panic!("unsupported bit count in shader {}", c)
            },
            spirq::ty::ScalarType::Float(c) => match c {
                4 => TypeId::of::<f32>(),
                8 => TypeId::of::<f64>(),
                c => panic!("unsupported bit count in shader {}", c)
            },
        },
        spirq::ty::Type::Vector(v) => match v.nscalar {
            2 => match v.scalar_ty {
                spirq::ty::ScalarType::Boolean => TypeId::of::<[bool; 2]>(),
                spirq::ty::ScalarType::Signed(c) => match c {
                    1 => TypeId::of::<[i8; 2]>(),
                    2 => TypeId::of::<[i16; 2]>(),
                    4 => TypeId::of::<[i32; 2]>(),
                    8 => TypeId::of::<[i64; 2]>(),
                    c => panic!("unsupported bit count in shader {}", c)
                },
                spirq::ty::ScalarType::Unsigned(c) => match c {
                    1 => TypeId::of::<[u8; 2]>(),
                    2 => TypeId::of::<[u16; 2]>(),
                    4 => TypeId::of::<[u32; 2]>(),
                    8 => TypeId::of::<[u64; 2]>(),
                    c => panic!("unsupported bit count in shader {}", c)
                },
                spirq::ty::ScalarType::Float(c) => match c {
                    4 => TypeId::of::<[f32; 2]>(),
                    8 => TypeId::of::<[f64; 2]>(),
                    c => panic!("unsupported bit count in shader {}", c)
                },
            },
            3 => match v.scalar_ty {
                spirq::ty::ScalarType::Boolean => TypeId::of::<[bool; 3]>(),
                spirq::ty::ScalarType::Signed(c) => match c {
                    1 => TypeId::of::<[i8; 3]>(),
                    2 => TypeId::of::<[i16; 3]>(),
                    4 => TypeId::of::<[i32; 3]>(),
                    8 => TypeId::of::<[i64; 3]>(),
                    c => panic!("unsupported bit count in shader {}", c)
                },
                spirq::ty::ScalarType::Unsigned(c) => match c {
                    1 => TypeId::of::<[u8; 3]>(),
                    2 => TypeId::of::<[u16; 3]>(),
                    4 => TypeId::of::<[u32; 3]>(),
                    8 => TypeId::of::<[u64; 3]>(),
                    c => panic!("unsupported bit count in shader {}", c)
                },
                spirq::ty::ScalarType::Float(c) => match c {
                    4 => TypeId::of::<[f32; 3]>(),
                    8 => TypeId::of::<[f64; 3]>(),
                    c => panic!("unsupported bit count in shader {}", c)
                },
            },
            4 => match v.scalar_ty {
                spirq::ty::ScalarType::Boolean => TypeId::of::<[bool; 4]>(),
                spirq::ty::ScalarType::Signed(c) => match c {
                    1 => TypeId::of::<[i8; 4]>(),
                    2 => TypeId::of::<[i16; 4]>(),
                    4 => TypeId::of::<[i32; 4]>(),
                    8 => TypeId::of::<[i64; 4]>(),
                    c => panic!("unsupported bit count in shader {}", c)
                },
                spirq::ty::ScalarType::Unsigned(c) => match c {
                    1 => TypeId::of::<[u8; 4]>(),
                    2 => TypeId::of::<[u16; 4]>(),
                    4 => TypeId::of::<[u32; 4]>(),
                    8 => TypeId::of::<[u64; 4]>(),
                    c => panic!("unsupported bit count in shader {}", c)
                },
                spirq::ty::ScalarType::Float(c) => match c {
                    4 => TypeId::of::<[f32; 4]>(),
                    8 => TypeId::of::<[f64; 4]>(),
                    c => panic!("unsupported bit count in shader {}", c)
                },
            }
            _ => panic!("unsupported vector size in push constant"),
        },
        spirq::ty::Type::Matrix(m) => {
            assert_eq!(m.nvec, m.vec_ty.nscalar, "ERROR only square matrices are supported in push constant blocks of shaders at the moment");
            match m.nvec {
                2 => match m.vec_ty.scalar_ty {
                    spirq::ty::ScalarType::Boolean => TypeId::of::<[[bool; 2]; 2]>(),
                    spirq::ty::ScalarType::Signed(c) => match c {
                        1 => TypeId::of::<[[i8; 2]; 2]>(),
                        2 => TypeId::of::<[[i16; 2]; 2]>(),
                        4 => TypeId::of::<[[i32; 2]; 2]>(),
                        8 => TypeId::of::<[[i64; 2]; 2]>(),
                        c => panic!("unsupported bit count in shader {}", c)
                    },
                    spirq::ty::ScalarType::Unsigned(c) => match c {
                        1 => TypeId::of::<[[u8; 2]; 2]>(),
                        2 => TypeId::of::<[[u16; 2]; 2]>(),
                        4 => TypeId::of::<[[u32; 2]; 2]>(),
                        8 => TypeId::of::<[[u64; 2]; 2]>(),
                        c => panic!("unsupported bit count in shader {}", c)
                    },
                    spirq::ty::ScalarType::Float(c) => match c {
                        4 => TypeId::of::<[[f32; 2]; 2]>(),
                        8 => TypeId::of::<[[f64; 2]; 2]>(),
                        c => panic!("unsupported bit count in shader {}", c)
                    },
                },
                3 => match m.vec_ty.scalar_ty {
                    spirq::ty::ScalarType::Boolean => TypeId::of::<[[bool; 3]; 3]>(),
                    spirq::ty::ScalarType::Signed(c) => match c {
                        1 => TypeId::of::<[[i8; 3]; 3]>(),
                        2 => TypeId::of::<[[i16; 3]; 3]>(),
                        4 => TypeId::of::<[[i32; 3]; 3]>(),
                        8 => TypeId::of::<[[i64; 3]; 3]>(),
                        c => panic!("unsupported bit count in shader {}", c)
                    },
                    spirq::ty::ScalarType::Unsigned(c) => match c {
                        1 => TypeId::of::<[[u8; 3]; 3]>(),
                        2 => TypeId::of::<[[u16; 3]; 3]>(),
                        4 => TypeId::of::<[[u32; 3]; 3]>(),
                        8 => TypeId::of::<[[u64; 3]; 3]>(),
                        c => panic!("unsupported bit count in shader {}", c)
                    },
                    spirq::ty::ScalarType::Float(c) => match c {
                        4 => TypeId::of::<[[f32; 3]; 3]>(),
                        8 => TypeId::of::<[[f64; 3]; 3]>(),
                        c => panic!("unsupported bit count in shader {}", c)
                    },
                },
                4 => match m.vec_ty.scalar_ty {
                    spirq::ty::ScalarType::Boolean => TypeId::of::<[[bool; 4]; 4]>(),
                    spirq::ty::ScalarType::Signed(c) => match c {
                        1 => TypeId::of::<[[i8; 4]; 4]>(),
                        2 => TypeId::of::<[[i16; 4]; 4]>(),
                        4 => TypeId::of::<[[i32; 4]; 4]>(),
                        8 => TypeId::of::<[[i64; 4]; 4]>(),
                        c => panic!("unsupported bit count in shader {}", c)
                    },
                    spirq::ty::ScalarType::Unsigned(c) => match c {
                        1 => TypeId::of::<[[u8; 4]; 4]>(),
                        2 => TypeId::of::<[[u16; 4]; 4]>(),
                        4 => TypeId::of::<[[u32; 4]; 4]>(),
                        8 => TypeId::of::<[[u64; 4]; 4]>(),
                        c => panic!("unsupported bit count in shader {}", c)
                    },
                    spirq::ty::ScalarType::Float(c) => match c {
                        4 => TypeId::of::<[[f32; 4]; 4]>(),
                        8 => TypeId::of::<[[f64; 4]; 4]>(),
                        c => panic!("unsupported bit count in shader {}", c)
                    },
                },
                _ => panic!("unsupported matrix size in push constant")
            }
        },
        spirq::ty::Type::Array(_) => unimplemented!(),
        spirq::ty::Type::Struct(_) => unimplemented!(),
        _ => unimplemented!()
    }
}

pub(crate) fn check_stage_compatibility(
    src: &[u32],
    src_stage: spirq::ExecutionModel,
    src_stage_name: &str,
    dst: &[u32],
    dst_stage: spirq::ExecutionModel,
    dst_stage_name: &str,
) -> Result<(), error::ParseSpirvError> {

    let mut src_outputs = None;

    let entry_points = spirq::ReflectConfig::new()
        .spv(src)
        .ref_all_rscs(true)
        .reflect()
        .expect(&format!("cannot reflect shader {}", src_stage_name));

    for entry in entry_points {
        if entry.exec_model != src_stage {
            continue;
        }

        src_outputs = Some(entry.vars.into_iter()
            .filter_map(|v| match v {
                spirq::Variable::Output { .. } => Some(v),
                _ => None,
            })
            .collect::<Vec<_>>());
        break;
    }

    let mut src_outputs = if let Some(src_outputs) = src_outputs {
        src_outputs
    } else {
        panic!("src stage doesn't have an entry for required shader stage");
    };

    let mut dst_inputs = None;

    let entry_points = spirq::ReflectConfig::new()
        .spv(dst)
        .ref_all_rscs(true)
        .reflect()
        .expect(&format!("cannot reflect shader {}", dst_stage_name));

    for entry in entry_points {
        if entry.exec_model != dst_stage {
            continue;
        }

        dst_inputs = Some(entry.vars.into_iter()
            .filter_map(|v| match v {
                spirq::Variable::Input { .. } => Some(v),
                _ => None,
            })
            .collect::<Vec<_>>());
        break;
    }

    let mut dst_inputs = if let Some(dst_inputs) = dst_inputs {
        dst_inputs
    } else {
        panic!("src stage doesn't have an entry for required shader stage");
    };

    dst_inputs.sort_unstable_by(|a, b| {
        let a_loc = if let spirq::Variable::Input { location, .. } = a {
            location.loc()
        } else {
            unreachable!()
        };
        let b_loc = if let spirq::Variable::Input { location, .. } = b {
            location.loc()
        } else {
            unreachable!()
        };
        a_loc.cmp(&b_loc)
    });

    src_outputs.sort_unstable_by(|a, b| {
        let a_loc = if let spirq::Variable::Output { location, .. } = a {
            location.loc()
        } else {
            unreachable!()
        };
        let b_loc = if let spirq::Variable::Output { location, .. } = b {
            location.loc()
        } else {
            unreachable!()
        };
        a_loc.cmp(&b_loc)
    });

    for input in dst_inputs.iter() {
        let (input_loc, input_ty) = if let spirq::Variable::Input { location, ty, .. } = input {
            (location.loc(), ty)
        } else {
            unreachable!()
        };

        // src stages can write to outputs not consumed by the input so find rather than iter zip
        if let Some(output) = src_outputs.iter().find(|v| {
            let loc = if let spirq::Variable::Output { location, .. } = v {
                location.loc()
            } else {
                unreachable!()
            };
            loc == input_loc
        }) {
            let (output_loc, output_ty) = if let spirq::Variable::Output { location, ty, .. } = output {
                (location.loc(), ty)
            } else {
                unreachable!()
            };
            if output_ty != input_ty
            {
                Err(error::ParseSpirvError::StageIncompatibility {
                    location: output_loc,
                    src_stage_name: src_stage_name.to_owned(),
                    src_type: output_ty.clone(),
                    dst_stage_name: dst_stage_name.to_owned(),
                    dst_type: input_ty.clone(),
                })?;
            }
        }
    }

    Ok(())
}
