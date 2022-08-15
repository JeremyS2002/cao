use std::{any::TypeId, collections::HashMap};

use super::error;

use either::*;

pub(crate) fn parse_vertex_states(
    vertex: &spv::Builder<spv::specialisation::Vertex>,
) -> std::sync::Arc<[crate::VertexLocationInfo]> {
    vertex
        .get_inputs()
        .iter()
        .filter_map(|(ty, e, n)| {
            if let Left((_, _)) = e {
                Some(crate::VertexLocationInfo {
                    name: n.unwrap().to_string(),
                    format: match ty {
                        spv::PrimitiveType::Float => gpu::VertexFormat::Float,
                        spv::PrimitiveType::Vec2 => gpu::VertexFormat::Vec2,
                        spv::PrimitiveType::Vec3 => gpu::VertexFormat::Vec3,
                        spv::PrimitiveType::Vec4 => gpu::VertexFormat::Vec4,
                        _ => panic!("ERROR: Cannot have input of type {:?} in vertex shader", ty),
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

pub(crate) fn process_shader<T: spv::specialisation::ShaderTY>(
    builder: &spv::Builder<T>,
    descriptor_set_layouts: &mut HashMap<u32, HashMap<u32, gpu::DescriptorLayoutEntry>>,
    descriptor_set_names: &mut HashMap<String, (usize, usize)>,
    push_constants: &mut Vec<gpu::PushConstantRange>,
    push_constant_names: &mut HashMap<String, (u32, gpu::ShaderStages, std::any::TypeId)>,
) {
    for ((set, binding), (v, n)) in builder.get_descriptor_layout_entries() {
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

    if let Some((ty, offset, name)) = builder.get_push_constant() {
        let stage = match T::TY {
            spv::rspirv::spirv::ExecutionModel::Vertex => gpu::ShaderStages::VERTEX,
            spv::rspirv::spirv::ExecutionModel::TessellationControl => {
                gpu::ShaderStages::TESSELLATION_CONTROL
            }
            spv::rspirv::spirv::ExecutionModel::TessellationEvaluation => {
                gpu::ShaderStages::TESSELLATION_EVAL
            }
            spv::rspirv::spirv::ExecutionModel::Geometry => gpu::ShaderStages::GEOMETRY,
            spv::rspirv::spirv::ExecutionModel::Fragment => gpu::ShaderStages::FRAGMENT,
            spv::rspirv::spirv::ExecutionModel::GLCompute => gpu::ShaderStages::COMPUTE,
            // spv::rspirv::spirv::ExecutionModel::Kernel => gpu::ShaderStages::,
            // spv::rspirv::spirv::ExecutionModel::TaskNV => gpu::ShaderStages::,
            // spv::rspirv::spirv::ExecutionModel::MeshNV => gpu::ShaderStages::,
            // spv::rspirv::spirv::ExecutionModel::RayGenerationNV => gpu::ShaderStages::,
            // spv::rspirv::spirv::ExecutionModel::IntersectionNV => gpu::ShaderStages::,
            // spv::rspirv::spirv::ExecutionModel::AnyHitNV => gpu::ShaderStages::,
            // spv::rspirv::spirv::ExecutionModel::ClosestHitNV => gpu::ShaderStages::,
            // spv::rspirv::spirv::ExecutionModel::MissNV => gpu::ShaderStages::,
            // spv::rspirv::spirv::ExecutionModel::CallableNV => gpu::ShaderStages::,
            _ => unimplemented!(),
        };

        let new = gpu::PushConstantRange {
            stage,
            offset,
            size: ty.size(),
        };

        push_constants.push(new);

        process_push_constant_ty(&ty, name, push_constant_names, offset, stage);
    }

    //push_constant_names.insert(name, ())
}

fn process_push_constant_ty(
    ty: &spv::DataType,
    name: Option<&str>,
    push_constant_names: &mut HashMap<String, (u32, gpu::ShaderStages, TypeId)>,
    mut offset: u32,
    stage: gpu::ShaderStages,
) {
    let id = |p: &spv::PrimitiveType| match p {
        spv::PrimitiveType::Bool => TypeId::of::<bool>(),
        spv::PrimitiveType::Int => TypeId::of::<i32>(),
        spv::PrimitiveType::UInt => TypeId::of::<u32>(),
        spv::PrimitiveType::Float => TypeId::of::<f32>(),
        spv::PrimitiveType::Double => TypeId::of::<f64>(),
        spv::PrimitiveType::IVec2 => TypeId::of::<[i32; 2]>(),
        spv::PrimitiveType::IVec3 => TypeId::of::<[i32; 3]>(),
        spv::PrimitiveType::IVec4 => TypeId::of::<[i32; 4]>(),
        spv::PrimitiveType::UVec2 => TypeId::of::<[u32; 2]>(),
        spv::PrimitiveType::UVec3 => TypeId::of::<[u32; 3]>(),
        spv::PrimitiveType::UVec4 => TypeId::of::<[u32; 4]>(),
        spv::PrimitiveType::Vec2 => TypeId::of::<[f32; 2]>(),
        spv::PrimitiveType::Vec3 => TypeId::of::<[f32; 3]>(),
        spv::PrimitiveType::Vec4 => TypeId::of::<[f32; 4]>(),
        spv::PrimitiveType::DVec2 => TypeId::of::<[f64; 2]>(),
        spv::PrimitiveType::DVec3 => TypeId::of::<[f64; 3]>(),
        spv::PrimitiveType::DVec4 => TypeId::of::<[f64; 4]>(),
        spv::PrimitiveType::Mat2 => TypeId::of::<[f32; 2 * 2]>(),
        spv::PrimitiveType::Mat3 => TypeId::of::<[f32; 3 * 3]>(),
        spv::PrimitiveType::Mat4 => TypeId::of::<[f32; 4 * 4]>(),
        spv::PrimitiveType::DMat2 => TypeId::of::<[f64; 2 * 2]>(),
        spv::PrimitiveType::DMat3 => TypeId::of::<[f64; 3 * 3]>(),
        spv::PrimitiveType::DMat4 => TypeId::of::<[f64; 4 * 4]>(),
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
        spv::DataType::Primitive(p) => {
            if let Some(name) = name {
                let id = id(p);
                let e = push_constant_names.entry(name.to_string()).or_insert((
                    0,
                    gpu::ShaderStages::empty(),
                    id,
                ));
                set_stages_or_panic(id, e, name, offset, stage);
            }
        }
        spv::DataType::Array(p, n) => {
            if let Some(name) = name {
                // kinda not great but arrays have to be set index by index or manually
                // without diving into how TypeId works and forcing a creation of TypeId::of::<[p; n]>() (<- doesn't work as n isn't const)
                // (technically could match on every n and code the relation but just no)
                let id = id(p);
                for i in 0..*n {
                    let e = push_constant_names
                        .entry(format!("{}{}", name, i))
                        .or_insert((0, gpu::ShaderStages::empty(), id));
                    set_stages_or_panic(id, e, name, offset, stage);
                }
            }
        }
        spv::DataType::Struct(ty_id, name, names, fields) => {
            let e = push_constant_names.entry(name.to_string()).or_insert((
                offset,
                gpu::ShaderStages::empty(),
                *ty_id,
            ));

            set_stages_or_panic(*ty_id, e, name, offset, stage);

            for (&name, field) in names.iter().zip(*fields) {
                process_push_constant_ty(field, Some(name), push_constant_names, offset, stage);
                offset += field.size();
            }
        }
    }
}

pub(crate) fn check_stage_compatibility<A, B>(
    src: &spv::Builder<A>,
    dst: &spv::Builder<B>,
) -> Result<(), error::BuilderConfigError>
where
    A: spv::specialisation::ShaderTY,
    B: spv::specialisation::ShaderTY,
{
    let src_o = src.get_outputs();

    let mut outputs = src_o;
    outputs.sort_unstable_by(|a, b| match a.1 {
        Left((a, _)) => match b.1 {
            Left((b, _)) => a.cmp(&b),
            Right(_) => std::cmp::Ordering::Less,
        },
        Right(_) => match b.1 {
            Left((_, _)) => std::cmp::Ordering::Greater,
            Right(_) => std::cmp::Ordering::Equal,
        },
    });
    let outputs = outputs
        .iter()
        .filter_map(|a| match a.1 {
            Left((l, _)) => Some((a.0, l)),
            Right(_) => None,
        })
        .collect::<Vec<_>>();

    let mut inputs = dst.get_inputs();
    inputs.sort_unstable_by(|a, b| match a.1 {
        Left((a, _)) => match b.1 {
            Left((b, _)) => a.cmp(&b),
            Right(_) => std::cmp::Ordering::Less,
        },
        Right(_) => match b.1 {
            Left((_, _)) => std::cmp::Ordering::Greater,
            Right(_) => std::cmp::Ordering::Equal,
        },
    });
    let inputs = inputs
        .iter()
        .filter_map(|a| match a.1 {
            Left((l, _)) => Some((a.0, l)),
            Right(_) => None,
        })
        .collect::<Vec<_>>();

    for input in inputs.iter() {
        // I don't know why I did this when they are already sorted by location but don't want to change it
        // TODO test this
        if let Some(output) = outputs.iter().find(|v| v.1 == input.1) {
            if output.0 != input.0 {
                Err(error::BuilderConfigError::StageIncompatibility {
                    location: output.1,
                    src_stage_name: format!("{:?}", A::TY),
                    src_type: output.0,
                    dst_stage_name: format!("{:?}", B::TY),
                    dst_type: input.0,
                })?;
            }
        }
    }

    Ok(())
}
