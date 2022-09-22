use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::Hash;
use std::hash::Hasher;
use std::sync::Arc;
use parking_lot::RwLock;

use super::bundle::BundleBuilder;
use super::error;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ComputePipelineKey {
    pub specialization: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct PipelineData {
    pub layout: gpu::PipelineLayout,
    pub shader: gpu::ShaderModule,
    pub cache: gpu::PipelineCache,
    pub name: Option<String>,
}

/// Reflected compute pipeline
#[derive(Clone)]
pub struct ReflectedCompute {
    pub(crate) id: u64,
    pub(crate) reflect_data: super::ReflectData,
    pub(crate) pipeline_data: PipelineData,
    pub(crate) pipeline_map: Arc<RwLock<HashMap<ComputePipelineKey, gpu::ComputePipeline>>>,
}

impl std::fmt::Debug for ReflectedCompute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "ReflectedGraphics id {}", self.id)
    }
}

impl ReflectedCompute {
    /// Create a new ReflectedCompute from spirv data
    pub fn from_spirv(
        device: &gpu::Device,
        compute: &[u32],
        cache: Option<gpu::PipelineCache>,
        name: Option<&str>,
    ) -> Result<Self, error::ReflectedError> {
        let mut reflect_builder = super::ReflectDataBuilder::new();
        let entry = reflect_builder.parse(
            compute,
            spirq::ExecutionModel::GLCompute,
        )?;

        let module_name = name.as_ref().map(|n| format!("{}_shader_module", n));

        let module = device.create_shader_module(&gpu::ShaderModuleDesc {
            name: module_name,
            entries: &[(gpu::ShaderStages::COMPUTE, &entry)],
            spirv: compute,
        })?;

        let (pipeline_layout, reflect_data) = reflect_builder.build(device, name)?;

        let mut hasher = DefaultHasher::new();
        module.hash(&mut hasher);

        let cache = if let Some(cache) = cache {
            cache
        } else {
            device.create_pipeline_cache(&gpu::PipelineCacheDesc {
                name: name.as_ref().map(|n| format!("{}_pipeline_cache", n)),
                initial_data: None,
            })?
        };

        Ok(Self {
            id: hasher.finish(),
            pipeline_map: Arc::default(),
            pipeline_data: PipelineData {
                layout: pipeline_layout,
                shader: module,
                cache,
                name: name.map(|n| n.to_string()),
            },
            reflect_data,
        })
    }

    /// Create a new BundleBuilder for this Compute
    pub fn bundle(&self) -> Option<BundleBuilder<'_>> {
        if self.reflect_data.descriptor_set_layouts.is_some() {
            Some(BundleBuilder {
                parent_id: self.id,
                parent_name: self.pipeline_data.name.as_ref().map(|n| &**n),
                reflect_data: &self.reflect_data,
                // map: self.reflect_data.descriptor_set_map.as_ref().unwrap(),
                // types: self.reflect_data.descriptor_set_types.as_ref().unwrap(),
                // layouts: self.reflect_data.descriptor_set_layouts.as_ref().unwrap(),
                descriptors: self
                    .reflect_data
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
        self.reflect_data.descriptor_set_layouts.is_some()
    }

    /// Get the id of the ReflectedCompute
    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn clear(&self) {
        self.pipeline_map.write().clear();
    }
}
