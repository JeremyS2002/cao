//! Utilities for creating pipelines
//!
//! If reflect feature is enabled then there are methods for creating pipeline layouts from spir-v data
//!
//! If spirv feature is enabled then there are methods for creating pipeline layouts from [`spv::Builder`] objects
//!
//! This isn't as fast as hard coding the values but speeds up prototyping a lot for me.
//!
//! [`ReflectedGraphics`] wraps a [`gpu::GraphicsPipeline`] and also manages [`gpu::RenderPass`] so that one ReflectedGraphics can render to different targets
//!
//! [`ReflectedCompute`] wraps a [`gpu::ComputePipeline`]
//!
//! [`Bundle`] manages [`gpu::DescriptorSet`] and [`BundleBuilder`] is used to assign resources to locations by name

pub mod bundle;
pub mod compute;
pub mod error;
pub mod graphics;
pub mod resource;

pub use bundle::*;
pub use compute::ReflectedCompute;
pub use error::*;
pub use graphics::ReflectedGraphics;
pub use resource::*;

use std::collections::HashMap;
use std::any::TypeId;
use std::sync::Arc;

#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) struct PushConstantInfo {
    pub offset: u32,
    pub stages: gpu::ShaderStages,
    pub type_id: TypeId,
}

#[derive(Clone, PartialEq, Debug)]
pub(crate) struct SpecConstantInfo {
    /// the id and stage which the push constant by this name is used
    pub stages: Vec<(u32, gpu::ShaderStages)>,
    pub type_id: TypeId,
}

pub(crate) struct ReflectDataBuilder {
    /// map from descriptor set to (map from descriptor_binding to gpu::DescriptorLayoutEntry)
    pub descriptor_set_layout_entries: HashMap<u32, HashMap<u32, gpu::DescriptorLayoutEntry>>,
    /// map from name: String to (set, binding)
    pub descriptor_set_names: HashMap<String, (u32, u32)>,
    /// the push constant ranges that will be used
    pub push_constant_ranges: Vec<gpu::PushConstantRange>,
    /// map from name to infomation about the push constant at that position
    pub push_constant_names: HashMap<String, PushConstantInfo>,
    /// map from name to information about the spec constant at that name
    pub specialization_names: HashMap<String, SpecConstantInfo>,
}

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

impl ReflectDataBuilder {
    pub fn new() -> Self {
        Self {
            descriptor_set_layout_entries: HashMap::new(),
            descriptor_set_names: HashMap::new(),
            push_constant_ranges: Vec::new(),
            push_constant_names: HashMap::new(),
            specialization_names: HashMap::new(),
        }
    }

    /// Parse the spir-v returning the entry point for this stage and updating selfs internal state
    pub fn parse(&mut self, spirv: &[u32], stage: spirq::ExecutionModel) -> Result<String, error::ParseSpirvError> {
        let stages = match stage {
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
            if entry.exec_model != stage {
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
                            let prev = self.descriptor_set_names.insert(name.clone(), (set as _, bind as _));
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
    
                        let map = self.descriptor_set_layout_entries.entry(set).or_insert(HashMap::new());
                        let e = map.entry(bind).or_insert(gpu::DescriptorLayoutEntry {
                            ty: gpu_ty,
                            stage: stages,
                            count: std::num::NonZeroU32::new(nbind).unwrap(),
                        });
                        e.stage |= stages;
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
                            self.push_constant_ranges.push(gpu::PushConstantRange {
                                stage: stages,
                                offset: 0,
                                size: s.nbyte() as _,
                            });
    
                            for member in s.members {
                                if let Some(n) = member.name {
                                    let ty_id = get_type_id(member.ty);
                                    let info = super::PushConstantInfo {
                                        offset: member.offset as _,
                                        stages,
                                        type_id: ty_id,
                                    };
                                    let prev = self.push_constant_names.entry(n.clone()).or_insert(info);
                                    prev.stages |= stages;
                                    if prev.offset != member.offset as _ || prev.type_id != ty_id {
                                        return Err(error::ParseSpirvError::PushNameConflict(n, member.offset as _, ty_id, prev.offset, prev.type_id))
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
                            let info = super::SpecConstantInfo {
                                stages: Vec::new(),
                                type_id: ty_id,
                            };
                            let e = self.specialization_names.entry(name.clone()).or_insert(info);
                            e.stages.push((spec_id, stages));
                            if e.type_id != ty_id {
                                return Err(error::ParseSpirvError::ConstantNameConflict(name, ty_id, e.type_id));
                            }
                        }
                    },
                }
            }
        }
    
        Ok(name.unwrap())
    }

    pub fn build(self, device: &gpu::Device, name: Option<&str>) -> Result<(gpu::PipelineLayout, ReflectData), gpu::Error> {
        // sort the hashmaps into ordered vecs
        let mut sorted = self.descriptor_set_layout_entries
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
        let descriptor_set_types = sorted
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

        let pipeline_layout_name = name.as_ref().map(|n| format!("{}_pipeline_layout", n));

        let pipeline_layout = device.create_pipeline_layout(&gpu::PipelineLayoutDesc {
            name: pipeline_layout_name,
            descriptor_sets: &descriptor_set_layouts.iter().collect::<Vec<_>>(),
            push_constants: &self.push_constant_ranges,
        })?;

        let bundle_needed = !(descriptor_set_layouts.len() == 0);
        let push_needed = self.push_constant_ranges.len() != 0;
        let spec_needed = self.specialization_names.len() != 0;

        let reflect_data = ReflectData {
            descriptor_set_layouts: if bundle_needed {
                Some(descriptor_set_layouts.into())
            } else {
                None
            },
            descriptor_set_map: if bundle_needed {
                Some(self.descriptor_set_names)
            } else {
                None
            },
            descriptor_set_types: if bundle_needed {
                Some(descriptor_set_types.into())
            } else {
                None
            },
            push_constant_names: if push_needed {
                Some(self.push_constant_names)
            } else {
                None
            },
            specialization_names: if spec_needed {
                Some(self.specialization_names)
            } else {
                None
            },
        };

        Ok((pipeline_layout, reflect_data))
    }
}


#[derive(Clone)]
pub(crate) struct ReflectData {
    pub descriptor_set_map: Option<HashMap<String, (u32, u32)>>,
    pub descriptor_set_types: Option<Arc<[Vec<(gpu::DescriptorLayoutEntryType, u32)>]>>,
    pub descriptor_set_layouts: Option<Arc<[gpu::DescriptorLayout]>>,
    pub push_constant_names: Option<HashMap<String, PushConstantInfo>>,
    pub specialization_names: Option<HashMap<String, SpecConstantInfo>>,
}

pub enum SpecVal {
    Int(i32),
    UInt(u32),
    Float(f32),
    Double(f64),
    IVec2([i32; 2]),
    IVec3([i32; 3]),
    IVec4([i32; 4]),
    UVec2([u32; 2]),
    UVec3([u32; 3]),
    UVec4([u32; 4]),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Vec4([f32; 4]),
    DVec2([f64; 2]),
    DVec3([f64; 3]),
    DVec4([f64; 4]),
}

impl SpecVal {
    pub fn type_id(&self) -> TypeId {
        match self {
            SpecVal::Int(_) => TypeId::of::<i32>(),
            SpecVal::UInt(_) => TypeId::of::<u32>(),
            SpecVal::Float(_) => TypeId::of::<f32>(),
            SpecVal::Double(_) => TypeId::of::<f64>(),
            SpecVal::IVec2(_) => TypeId::of::<[i32; 2]>(),
            SpecVal::IVec3(_) => TypeId::of::<[i32; 3]>(),
            SpecVal::IVec4(_) => TypeId::of::<[i32; 4]>(),
            SpecVal::UVec2(_) => TypeId::of::<[u32; 2]>(),
            SpecVal::UVec3(_) => TypeId::of::<[u32; 3]>(),
            SpecVal::UVec4(_) => TypeId::of::<[u32; 4]>(),
            SpecVal::Vec2(_) => TypeId::of::<[f32; 2]>(),
            SpecVal::Vec3(_) => TypeId::of::<[f32; 3]>(),
            SpecVal::Vec4(_) => TypeId::of::<[f32; 4]>(),
            SpecVal::DVec2(_) => TypeId::of::<[f64; 2]>(),
            SpecVal::DVec3(_) => TypeId::of::<[f64; 3]>(),
            SpecVal::DVec4(_) => TypeId::of::<[f64; 4]>(),
        }
    }

    pub fn bytes<'a>(&'a self) -> &'a [u8] {
        match self {
            SpecVal::Int(d) => bytemuck::bytes_of(d),
            SpecVal::UInt(d) => bytemuck::bytes_of(d),
            SpecVal::Float(d) => bytemuck::bytes_of(d),
            SpecVal::Double(d) => bytemuck::bytes_of(d),
            SpecVal::IVec2(d) => bytemuck::bytes_of(d),
            SpecVal::IVec3(d) => bytemuck::bytes_of(d),
            SpecVal::IVec4(d) => bytemuck::bytes_of(d),
            SpecVal::UVec2(d) => bytemuck::bytes_of(d),
            SpecVal::UVec3(d) => bytemuck::bytes_of(d),
            SpecVal::UVec4(d) => bytemuck::bytes_of(d),
            SpecVal::Vec2(d) => bytemuck::bytes_of(d),
            SpecVal::Vec3(d) => bytemuck::bytes_of(d),
            SpecVal::Vec4(d) => bytemuck::bytes_of(d),
            SpecVal::DVec2(d) => bytemuck::bytes_of(d),
            SpecVal::DVec3(d) => bytemuck::bytes_of(d),
            SpecVal::DVec4(d) => bytemuck::bytes_of(d),
        }
    }
}