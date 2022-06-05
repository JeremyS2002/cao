
use std::collections::HashMap;

use either::*;

pub(crate) fn parse_vertex_states(vertex: &spv::Builder<spv::specialisation::Vertex>) -> std::sync::Arc<[crate::VertexLocationInfo]> {
    vertex
        .inputs()
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

pub(crate) fn combine_descriptor_set_layouts(
    device: &gpu::Device, 
    descriptor_set_layouts: HashMap<u32, HashMap<u32, gpu::DescriptorLayoutEntry>>,
    name: &Option<String>
) -> Result<(Vec<gpu::DescriptorLayout>, Vec<Vec<(gpu::DescriptorLayoutEntryType, u32)>>), gpu::Error> {
    todo!()
}

pub(crate) fn process_shader<T: spv::specialisation::ShaderTY>(
    builder: &spv::Builder<T>, 
    descriptor_set_layouts: &mut HashMap<u32, HashMap<u32, gpu::DescriptorLayoutEntry>>, 
    descriptor_set_names: &mut HashMap<String, (usize, usize)>, 
    _push_constants: &mut Vec<gpu::PushConstantRange>, 
    _push_constant_names: &mut HashMap<String, (u32, gpu::ShaderStages, std::any::TypeId)>
) {
    for ((set, binding), (v, n)) in builder.descriptor_layout_entries() {
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