use parking_lot::RwLock;
use std::sync::Arc;

use std::any::TypeId;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::Hash;
use std::hash::Hasher;

use super::bundle::BundleBuilder;
use super::error;

/// Allowing for caching pipelines by viewport so that the same
/// "pipeline" can be used even when the window resized eg
/// afaik this is ok but there is a good chance that i've messed something up
#[derive(Copy, Clone, Debug)]
pub struct GraphicsPipelineKey {
    pub pass_hash: u64,
    pub viewport: gpu::Viewport,
    pub vertex_ty: TypeId,
    pub spec_hash: Option<u64>,
}

impl std::cmp::PartialEq for GraphicsPipelineKey {
    fn eq(&self, other: &Self) -> bool {
        self.vertex_ty == other.vertex_ty
            && self.viewport.x == other.viewport.x
            && self.viewport.y == other.viewport.y
            && self.viewport.width == other.viewport.width
            && self.viewport.height == other.viewport.height
            && self.viewport.min_depth.to_bits() == other.viewport.min_depth.to_bits()
            && self.viewport.max_depth.to_bits() == other.viewport.max_depth.to_bits()
            && self.pass_hash == other.pass_hash
    }
}

impl std::cmp::Eq for GraphicsPipelineKey {}

impl std::hash::Hash for GraphicsPipelineKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.vertex_ty.hash(state);
        self.viewport.x.hash(state);
        self.viewport.y.hash(state);
        self.viewport.width.hash(state);
        self.viewport.height.hash(state);
        self.viewport.min_depth.to_bits().hash(state);
        self.viewport.max_depth.to_bits().hash(state);
        self.pass_hash.hash(state);
    }
}

pub struct VertexLocationInfo {
    pub name: String,
    pub format: gpu::VertexFormat,
}

#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub struct PipelineData {
    pub layout: gpu::PipelineLayout,
    pub rasterizer: gpu::Rasterizer,
    pub blend_states: Arc<[gpu::BlendState]>,
    pub vertex: gpu::ShaderModule,
    pub fragment: Option<gpu::ShaderModule>,
    pub geometry: Option<gpu::ShaderModule>,
    pub depth_stencil: Option<gpu::DepthStencilState>,
    pub cache: Option<gpu::PipelineCache>,
    pub name: Option<String>,
}

/// A a reflected collection of graphics pipelines with a map from vertex to pipeline
#[derive(Clone)]
pub struct ReflectedGraphics {
    pub(crate) id: u64,
    /// Map from RenderPassDesc to RenderPass
    pub(crate) pass_map: Arc<RwLock<HashMap<gpu::RenderPassDesc<'static>, gpu::RenderPass>>>,
    /// Map from (raw_render_pass, vertex_type) to pipeline
    /// Example usage
    /// ```
    /// let raw_render_pass = unsafe {
    ///     std::mem::transmute(render_pass.raw_pass())  
    /// };
    /// let vertex_type = TypeId::of::<MyVertex>();
    /// let pipeline = self.pipeline_map.read().get(&(raw_render_pass, vertex_type)).unwrap();
    /// ```
    pub(crate) pipeline_map: Arc<RwLock<HashMap<GraphicsPipelineKey, gpu::GraphicsPipeline>>>,
    /// Copies of data needed to build more pipelines
    pub(crate) pipeline_data: PipelineData,
    /// ordered list of vertex inputs required
    pub(crate) vertex_map: Arc<[super::graphics::VertexLocationInfo]>,
    /// Data needed to build bundles and for push_T functions
    pub(crate) reflect_data: super::ReflectData,
}

impl std::cmp::PartialEq for ReflectedGraphics {
    fn eq(&self, other: &ReflectedGraphics) -> bool {
        self.id == other.id
    }
}

impl std::cmp::Eq for ReflectedGraphics {}

impl std::hash::Hash for ReflectedGraphics {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

impl std::fmt::Debug for ReflectedGraphics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "ReflectedGraphics id {}", self.id)
    }
}


#[cfg(feature = "reflect")]
impl ReflectedGraphics {
    /// Create a new Graphics from spirv data
    ///
    /// TODO check shader compatibility
    /// TODO check if shader stages are the same with multiple entry points
    pub fn from_spirv(
        device: &gpu::Device,
        vertex: &[u32],
        geometry: Option<&[u32]>,
        fragment: Option<&[u32]>,
        rasterizer: gpu::Rasterizer,
        blend_states: &[gpu::BlendState],
        depth_stencil: Option<gpu::DepthStencilState>,
        cache: Option<gpu::PipelineCache>,
        name: Option<&str>,
    ) -> Result<Self, error::ReflectedError> {
        let mut reflect_builder = super::ReflectDataBuilder::new();

        let vertex_entry = reflect_builder.parse(
            vertex,
            spirq::ExecutionModel::Vertex,
        )?;
        let vertex_map = super::parse_vertex_states(vertex)?;

        let vertex_name = name.as_ref().map(|n| format!("{}_vertex_module", n));

        let vertex_module = device.create_shader_module(&gpu::ShaderModuleDesc {
            entries: &[(gpu::ShaderStages::VERTEX, &vertex_entry)],
            spirv: vertex,
            name: vertex_name,
        })?;

        let geometry_module = if let Some(geometry) = geometry {
            super::check_stage_compatibility(
                vertex, 
                spirq::ExecutionModel::Vertex,
                "vertex", 
                geometry, 
                spirq::ExecutionModel::Geometry,
                "geometry"
            )?;

            let geometry_name = name.as_ref().map(|n| format!("{}_geometry_module", n));

            let entry = reflect_builder.parse(
                geometry,
                spirq::ExecutionModel::Geometry,
            )?;
            Some(device.create_shader_module(&gpu::ShaderModuleDesc {
                entries: &[(gpu::ShaderStages::GEOMETRY, &entry)],
                spirv: geometry,
                name: geometry_name,
            })?)
        } else {
            None
        };

        let fragment_module = if let Some(fragment) = fragment {
            if geometry.is_some() {
                super::check_stage_compatibility(
                    geometry.unwrap(),
                    spirq::ExecutionModel::Geometry,
                    "geometry",
                    fragment,
                    spirq::ExecutionModel::Fragment,
                    "fragment",
                )?;
            } else {
                super::check_stage_compatibility(
                    vertex, 
                    spirq::ExecutionModel::Vertex,
                    "vertex", 
                    fragment, 
                    spirq::ExecutionModel::Fragment,
                    "fragment",
                )?;
            }

            let fragment_name = name.as_ref().map(|n| format!("{}_fragment_module", n));

            let entry = reflect_builder.parse(
                fragment,
                spirq::ExecutionModel::Fragment,
            )?;
            Some(device.create_shader_module(&gpu::ShaderModuleDesc {
                entries: &[(gpu::ShaderStages::FRAGMENT, &entry)],
                spirv: fragment,
                name: fragment_name,
            })?)
        } else {
            None
        };

        let (pipeline_layout, reflect_data) = reflect_builder.build(device, name)?;

        let mut hasher = DefaultHasher::new();

        vertex_module.hash(&mut hasher);
        fragment_module.hash(&mut hasher);
        geometry_module.hash(&mut hasher);

        Ok(Self {
            id: hasher.finish(),
            pass_map: Arc::new(RwLock::default()),
            pipeline_map: Arc::new(RwLock::default()),
            vertex_map: vertex_map.into(),
            reflect_data,
            pipeline_data: PipelineData {
                layout: pipeline_layout,
                vertex: vertex_module,
                fragment: fragment_module,
                geometry: geometry_module,
                rasterizer,
                blend_states: blend_states.to_vec().into(),
                depth_stencil,
                name: name.map(|n| n.to_string()),
                cache,
            },
        })
    }
}

impl ReflectedGraphics {
    /// Create a new BundleBuilder for this Graphics
    ///
    /// Returns none if the shaders stages have no binding
    pub fn bundle(&self) -> Option<BundleBuilder<'_>> {
        if self.reflect_data.descriptor_set_layouts.is_some() {
            Some(BundleBuilder {
                parent_id: self.id,
                parent_name: self.pipeline_data.name.as_ref().map(|n| &**n),
                // map: self.reflect_data.descriptor_set_map.as_ref().unwrap(),
                // types: self.reflect_data.descriptor_set_types.as_ref().unwrap(),
                // layouts: self.reflect_data.descriptor_set_layouts.as_ref().unwrap(),
                reflect_data: &self.reflect_data,
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

    /// Returns if the graphics requires a bundle to run
    pub fn bundle_needed(&self) -> bool {
        self.reflect_data.descriptor_set_layouts.is_some()
    }

    /// create vertex attributes for a type that implements vertex
    /// to match the pipeline contained in self
    pub fn vertex_attributes<V: crate::Vertex>(&self) -> Vec<gpu::VertexAttribute> {
        let attribs = self
            .vertex_map
            .iter()
            .enumerate()
            .map(|(i, info)| {
                if let Some((offset, format)) = V::get(&info.name) {
                    if format == info.format {
                        gpu::VertexAttribute {
                            location: i as _,
                            format: info.format,
                            offset,
                        }
                    } else {
                        panic!("ERROR: Vertex format type mismatch at position name = {}\nexpected {:?} found {:?}", info.name, info.format, format)
                    }
                } else {
                    panic!("ERROR: Vertex doesn't have attribute with name {}", info.name)
                }
            })
            .collect::<Vec<_>>();

        attribs
    }

    /// Get the id of the ReflectedGraphics
    pub fn id(&self) -> u64 {
        self.id
    }

    /// The Reflected Graphics caches [`gpu::RenderPass`] and [`gpu::GraphicsPipeline`] to be reused
    /// this function will clear all the old pipelines
    pub fn clear(&self) {
        self.pass_map.write().clear();
        self.pipeline_map.write().clear();
    }
}
