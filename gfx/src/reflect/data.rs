use std::collections::HashMap;
use std::sync::Arc;
use std::any::TypeId;

#[derive(Clone)]
pub struct ReflectData {
    pub descriptor_set_map: Option<HashMap<String, (usize, usize)>>,
    pub descriptor_set_types: Option<Arc<[Vec<(gpu::DescriptorLayoutEntryType, u32)>]>>,
    pub descriptor_set_layouts: Option<Arc<[gpu::DescriptorLayout]>>,
    pub push_constant_names: Option<HashMap<String, (u32, gpu::ShaderStages, TypeId)>>,
    pub specialization_names: Option<HashMap<String, (TypeId, Vec<(u32, gpu::ShaderStages)>)>>,
}

impl ReflectData {
    /// Get a reference to the descriptor layouts if any
    pub fn descriptor_layouts<'a>(&'a self) -> Option<&'a [gpu::DescriptorLayout]> {
        self.descriptor_set_layouts.as_ref().map(|l| &**l)
    }
}
