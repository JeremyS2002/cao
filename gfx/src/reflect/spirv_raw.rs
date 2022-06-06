use std::collections::HashMap;

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

pub(crate) fn process_shader<T: spv::specialisation::ShaderTY>(
    builder: &spv::Builder<T>,
    descriptor_set_layouts: &mut HashMap<u32, HashMap<u32, gpu::DescriptorLayoutEntry>>,
    descriptor_set_names: &mut HashMap<String, (usize, usize)>,
    _push_constants: &mut Vec<gpu::PushConstantRange>,
    _push_constant_names: &mut HashMap<String, (u32, gpu::ShaderStages, std::any::TypeId)>,
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
    outputs.sort_unstable_by(|a, b| {
        match a.1 {
            Left((a, _)) => {
                match b.1 {
                    Left((b, _)) => a.cmp(&b),
                    Right(_) => std::cmp::Ordering::Less,
                }
            },
            Right(_) => {
                match b.1 {
                    Left((_, _)) => std::cmp::Ordering::Greater,
                    Right(_) => std::cmp::Ordering::Equal,
                }
            }
        }
    });
    let outputs = outputs
        .iter()
        .filter_map(|a| {
            match a.1 {
                Left((l, _)) => Some((a.0, l)),
                Right(_) => None,
            }
        })
        .collect::<Vec<_>>();

    let mut inputs = dst.get_inputs();
    inputs.sort_unstable_by(|a, b| {
        match a.1 {
            Left((a, _)) => {
                match b.1 {
                    Left((b, _)) => a.cmp(&b),
                    Right(_) => std::cmp::Ordering::Less,
                }
            },
            Right(_) => {
                match b.1 {
                    Left((_, _)) => std::cmp::Ordering::Greater,
                    Right(_) => std::cmp::Ordering::Equal,
                }
            }
        }
    });
    let inputs = inputs
        .iter()
        .filter_map(|a| {
            match a.1 {
                Left((l, _)) => Some((a.0, l)),
                Right(_) => None,
            }
        })
        .collect::<Vec<_>>();

    for input in inputs.iter() {
        // I don't know why I did this when they are already sorted by location but don't want to change it
        // TODO test this
        if let Some(output) = outputs.iter().find(|v| v.1 == input.1) {
            if output.0 != input.0
            {
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
