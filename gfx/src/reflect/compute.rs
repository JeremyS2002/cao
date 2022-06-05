use std::any::TypeId;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::Hash;
use std::hash::Hasher;

use super::bundle::BundleBuilder;
use super::error;

/// Reflected compute pipeline
pub struct ReflectedCompute {
    pub(crate) id: u64,
    pub(crate) pipeline: gpu::ComputePipeline,

    pub(crate) descriptor_set_names: Option<HashMap<String, (usize, usize)>>,
    pub(crate) descriptor_set_types: Option<Vec<Vec<(gpu::DescriptorLayoutEntryType, u32)>>>,
    pub(crate) descriptor_set_layouts: Option<Vec<gpu::DescriptorLayout>>,

    pub(crate) push_constant_names: Option<HashMap<String, (u32, gpu::ShaderStages, TypeId)>>,
}

impl std::fmt::Debug for ReflectedCompute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "ReflectedGraphics id {}", self.id)
    }
}

impl ReflectedCompute {
    /// Create a new ReflectedCompute from spirv data
    pub fn new(
        device: &gpu::Device,
        compute: &[u32],
        name: Option<String>,
    ) -> Result<Self, error::ReflectedError> {
        let mut descriptor_set_layouts = HashMap::new();
        let mut descriptor_set_names = HashMap::new();
        let mut push_constants = Vec::new();
        let mut push_constant_names = HashMap::new();
        let entry = super::reflect_raw::parse_spirv(
            &mut descriptor_set_layouts,
            &mut descriptor_set_names,
            &mut push_constants,
            &mut push_constant_names,
            compute,
            spirv_reflect::types::variable::ReflectShaderStageFlags::COMPUTE,
        )?;

        let module_name = name.as_ref().map(|n| format!("{}_shader_module", n));

        let module = device.create_shader_module(&gpu::ShaderModuleDesc {
            name: module_name,
            entries: &[(gpu::ShaderStages::COMPUTE, &entry)],
            spirv: compute,
        })?;

        let (descriptor_set_layouts, descriptor_set_types) =
            super::reflect_raw::combine_descriptor_set_layouts(device, descriptor_set_layouts, &name)?;

        let pipeline_layout_name = name.as_ref().map(|n| format!("{}_pipeline_layout", n));

        let pipeline_layout = device.create_pipeline_layout(&gpu::PipelineLayoutDesc {
            name: pipeline_layout_name,
            descriptor_sets: &descriptor_set_layouts.iter().collect::<Vec<_>>(),
            push_constants: &push_constants,
        })?;

        let pipeline = device.create_compute_pipeline(&gpu::ComputePipelineDesc {
            name: name.map(|n| format!("{}_pipeline", n)),
            layout: &pipeline_layout,
            shader: &module,
        })?;

        let mut hasher = DefaultHasher::new();
        module.hash(&mut hasher);

        let bundle_needed = !(descriptor_set_layouts.len() == 0);

        Ok(Self {
            id: hasher.finish(),
            pipeline,
            descriptor_set_layouts: if bundle_needed {
                Some(descriptor_set_layouts)
            } else {
                None
            },
            descriptor_set_names: if bundle_needed {
                Some(descriptor_set_names)
            } else {
                None
            },
            descriptor_set_types: if bundle_needed {
                Some(descriptor_set_types)
            } else {
                None
            },
            push_constant_names: if push_constants.len() != 0 {
                Some(push_constant_names)
            } else {
                None
            },
        })
    }

    /// Create a new BundleBuilder for this Compute
    pub fn bundle(&self) -> Option<BundleBuilder<'_>> {
        if self.descriptor_set_layouts.is_some() {
            Some(BundleBuilder {
                parent_id: self.id,
                parent_name: self.pipeline.name().map(|s| s.to_string()),
                map: self.descriptor_set_names.as_ref().unwrap(),
                types: self.descriptor_set_types.as_ref().unwrap(),
                layouts: self.descriptor_set_layouts.as_ref().unwrap(),
                descriptors: self
                    .descriptor_set_types
                    .as_ref()
                    .unwrap()
                    .iter()
                    .map(|v| v.iter().map(|_| None).collect::<Vec<_>>())
                    .collect::<Vec<_>>(),
            })
        } else {
            None
        }
    }

    /// Returns if the Compute pipeline requires a bundle to run
    pub fn bundle_needed(&self) -> bool {
        self.descriptor_set_layouts.is_some()
    }

    /// Get the id of the ReflectedCompute
    pub fn id(&self) -> u64 {
        self.id
    }
}
