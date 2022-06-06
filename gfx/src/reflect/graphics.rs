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
#[derive(Copy, Clone)]
pub struct GraphicsPipelineKey {
    pub pass_hash: u64,
    pub viewport: gpu::Viewport,
    pub vertex_ty: TypeId,
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

pub(crate) struct VertexLocationInfo {
    pub(crate) name: String,
    pub(crate) format: gpu::VertexFormat,
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
    pub name: Option<String>,
}

#[derive(Clone)]
#[allow(missing_docs, missing_debug_implementations)]
pub struct ReflectData {
    pub(crate) vertex_map: Arc<[VertexLocationInfo]>,
    pub(crate) descriptor_set_map: Option<HashMap<String, (usize, usize)>>,
    pub(crate) descriptor_set_types: Option<Arc<[Vec<(gpu::DescriptorLayoutEntryType, u32)>]>>,
    pub(crate) descriptor_set_layouts: Option<Arc<[gpu::DescriptorLayout]>>,
    pub(crate) push_constant_names: Option<HashMap<String, (u32, gpu::ShaderStages, TypeId)>>,
}

impl ReflectData {
    /// Get a reference to the descriptor layouts if any
    pub fn descriptor_layouts<'a>(&'a self) -> Option<&'a [gpu::DescriptorLayout]> {
        self.descriptor_set_layouts.as_ref().map(|l| &**l)
    }
}

/// A a reflected collection of graphics pipelines with a map from vertex to pipeline
#[derive(Clone)]
pub struct ReflectedGraphics {
    pub(crate) id: u64,
    /// Map from RenderPassDesc to RenderPass
    pub pass_map: Arc<RwLock<HashMap<gpu::RenderPassDesc<'static>, gpu::RenderPass>>>,
    /// Map from (raw_render_pass, vertex_type) to pipeline
    /// Example usage
    /// ```
    /// let raw_render_pass = unsafe {
    ///     std::mem::transmute(render_pass.raw_pass())  
    /// };
    /// let vertex_type = TypeId::of::<MyVertex>();
    /// let pipeline = self.pipeline_map.read().get(&(raw_render_pass, vertex_type)).unwrap();
    /// ```
    pub pipeline_map: Arc<RwLock<HashMap<GraphicsPipelineKey, gpu::GraphicsPipeline>>>,
    /// Copies of data needed to build more pipelines
    pub pipeline_data: PipelineData,
    /// Data needed to build bundles and for push_T functions
    pub reflect_data: ReflectData,
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

#[cfg(feature = "spirv")]
impl ReflectedGraphics {
    pub fn from_builders(
        device: &gpu::Device,
        vertex: spv::VertexBuilder,
        geometry: Option<spv::GeometryBuilder>,
        fragment: Option<spv::FragmentBuilder>,
        rasterizer: gpu::Rasterizer,
        blend_states: &[gpu::BlendState],
        depth_stencil: Option<gpu::DepthStencilState>,
        name: Option<String>,
    ) -> Result<Self, error::ReflectedError> {
        let vertex_map = super::spirv_raw::parse_vertex_states(&vertex);
        let mut descriptor_set_layouts = HashMap::new();
        let mut descriptor_set_names = HashMap::new();
        let mut push_constants = Vec::new();
        let mut push_constant_names = HashMap::new();

        if let Some(geometry) = &geometry {
            super::spirv_raw::check_stage_compatibility(
                &vertex, 
                geometry,
            )?;
            
            if let Some(fragment) = &fragment {
                super::spirv_raw::check_stage_compatibility(
                    geometry, 
                    fragment,
                )?;
            }
        } else {
            if let Some(fragment) = &fragment {
                super::spirv_raw::check_stage_compatibility(
                    &vertex, 
                    fragment,
                )?;
            }
        }

        super::spirv_raw::process_shader(
            &vertex,
            &mut descriptor_set_layouts,
            &mut descriptor_set_names,
            &mut push_constants,
            &mut push_constant_names,
        );

        let vertex_spv = vertex.compile();
        let vertex_module = device.create_shader_module(&gpu::ShaderModuleDesc {
            name: name.as_ref().map(|n| format!("{}_vertex_module", n)),
            entries: &[(gpu::ShaderStages::VERTEX, "main")],
            spirv: &vertex_spv,
        })?;

        let geometry_module = geometry
            .map(|g| {
                super::spirv_raw::process_shader(
                    &g,
                    &mut descriptor_set_layouts,
                    &mut descriptor_set_names,
                    &mut push_constants,
                    &mut push_constant_names,
                );
                let spv = g.compile();
                device.create_shader_module(&gpu::ShaderModuleDesc {
                    name: name.as_ref().map(|n| format!("{}_geometry_module", n)),
                    entries: &[(gpu::ShaderStages::GEOMETRY, "main")],
                    spirv: &spv,
                })
            })
            .transpose()?;

        let fragment_module = fragment
            .map(|f| {
                super::spirv_raw::process_shader(
                    &f,
                    &mut descriptor_set_layouts,
                    &mut descriptor_set_names,
                    &mut push_constants,
                    &mut push_constant_names,
                );
                let spv = f.compile();
                device.create_shader_module(&gpu::ShaderModuleDesc {
                    name: name.as_ref().map(|n| format!("{}_fragment_module", n)),
                    entries: &[(gpu::ShaderStages::FRAGMENT, "main")],
                    spirv: &spv,
                })
            })
            .transpose()?;

        let (descriptor_set_layouts, descriptor_set_types) =
            super::spirv_raw::combine_descriptor_set_layouts(
                device,
                descriptor_set_layouts,
                &name,
            )?;

        let pipeline_layout = device.create_pipeline_layout(&gpu::PipelineLayoutDesc {
            name: name.as_ref().map(|n| format!("{}_pipeline_layout", n)),
            descriptor_sets: &descriptor_set_layouts.iter().collect::<Vec<_>>(),
            push_constants: &push_constants,
        })?;

        let mut hasher = DefaultHasher::new();

        vertex_module.hash(&mut hasher);
        geometry_module.hash(&mut hasher);
        fragment_module.hash(&mut hasher);

        let bundle_needed = !(descriptor_set_layouts.len() == 0);

        Ok(Self {
            id: hasher.finish(),
            pass_map: Arc::new(RwLock::default()),
            pipeline_map: Arc::new(RwLock::default()),
            reflect_data: ReflectData {
                vertex_map,
                descriptor_set_map: if bundle_needed {
                    Some(descriptor_set_names.into())
                } else {
                    None
                },
                descriptor_set_layouts: if bundle_needed {
                    Some(descriptor_set_layouts.into())
                } else {
                    None
                },
                descriptor_set_types: if bundle_needed {
                    Some(descriptor_set_types.into())
                } else {
                    None
                },
                push_constant_names: if push_constants.len() != 0 {
                    Some(push_constant_names)
                } else {
                    None
                },
            },
            pipeline_data: PipelineData {
                layout: pipeline_layout,
                vertex: vertex_module,
                fragment: fragment_module,
                geometry: geometry_module,
                rasterizer,
                blend_states: blend_states.to_vec().into(),
                depth_stencil,
                name,
            },
        })
    }
}

#[cfg(feature = "reflect")]
impl ReflectedGraphics {
    /// Create a new Graphics from spirv data
    ///
    /// TODO check shader compatibility
    /// TODO check if shader stages are the same with multiple entry points
    pub fn from_spv(
        device: &gpu::Device,
        vertex: &[u32],
        geometry: Option<&[u32]>,
        fragment: Option<&[u32]>,
        rasterizer: gpu::Rasterizer,
        blend_states: &[gpu::BlendState],
        depth_stencil: Option<gpu::DepthStencilState>,
        name: Option<String>,
    ) -> Result<Self, error::ReflectedError> {
        let mut descriptor_set_layouts = HashMap::new();
        let mut descriptor_set_names = HashMap::new();
        let mut push_constants = Vec::new();
        let mut push_constant_names = HashMap::new();
        let vertex_entry = super::reflect_raw::parse_spirv(
            &mut descriptor_set_layouts,
            &mut descriptor_set_names,
            &mut push_constants,
            &mut push_constant_names,
            vertex,
            spirv_reflect::types::variable::ReflectShaderStageFlags::VERTEX,
        )?;
        let vertex_map = super::reflect_raw::parse_vertex_states(vertex, &vertex_entry)?;

        let vertex_name = name.as_ref().map(|n| format!("{}_vertex_module", n));

        let vertex_module = device.create_shader_module(&gpu::ShaderModuleDesc {
            entries: &[(gpu::ShaderStages::VERTEX, &vertex_entry)],
            spirv: vertex,
            name: vertex_name,
        })?;

        let geometry_module = if let Some(geometry) = geometry {
            super::reflect_raw::check_stage_compatibility(vertex, "vertex", geometry, "geometry")?;

            let geometry_name = name.as_ref().map(|n| format!("{}_geometry_module", n));

            let entry = super::reflect_raw::parse_spirv(
                &mut descriptor_set_layouts,
                &mut descriptor_set_names,
                &mut push_constants,
                &mut push_constant_names,
                geometry,
                spirv_reflect::types::variable::ReflectShaderStageFlags::GEOMETRY,
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
                super::reflect_raw::check_stage_compatibility(
                    geometry.unwrap(),
                    "geometry",
                    fragment,
                    "fragment",
                )?;
            } else {
                super::reflect_raw::check_stage_compatibility(
                    vertex, "vertex", fragment, "fragment",
                )?;
            }

            let fragment_name = name.as_ref().map(|n| format!("{}_fragment_module", n));

            let entry = super::reflect_raw::parse_spirv(
                &mut descriptor_set_layouts,
                &mut descriptor_set_names,
                &mut push_constants,
                &mut push_constant_names,
                fragment,
                spirv_reflect::types::variable::ReflectShaderStageFlags::FRAGMENT,
            )?;
            Some(device.create_shader_module(&gpu::ShaderModuleDesc {
                entries: &[(gpu::ShaderStages::FRAGMENT, &entry)],
                spirv: fragment,
                name: fragment_name,
            })?)
        } else {
            None
        };

        let (descriptor_set_layouts, descriptor_set_types) =
            super::reflect_raw::combine_descriptor_set_layouts(
                device,
                descriptor_set_layouts,
                &name,
            )?;

        let pipeline_layout_name = name.as_ref().map(|n| format!("{}_pipeline_layout", n));

        let pipeline_layout = device.create_pipeline_layout(&gpu::PipelineLayoutDesc {
            name: pipeline_layout_name,
            descriptor_sets: &descriptor_set_layouts.iter().collect::<Vec<_>>(),
            push_constants: &push_constants,
        })?;

        let mut hasher = DefaultHasher::new();

        vertex_module.hash(&mut hasher);
        fragment_module.hash(&mut hasher);
        geometry_module.hash(&mut hasher);

        let bundle_needed = !(descriptor_set_layouts.len() == 0);

        Ok(Self {
            id: hasher.finish(),
            pass_map: Arc::new(RwLock::default()),
            pipeline_map: Arc::new(RwLock::default()),
            reflect_data: ReflectData {
                vertex_map: vertex_map.into(),
                descriptor_set_map: if bundle_needed {
                    Some(descriptor_set_names.into())
                } else {
                    None
                },
                descriptor_set_layouts: if bundle_needed {
                    Some(descriptor_set_layouts.into())
                } else {
                    None
                },
                descriptor_set_types: if bundle_needed {
                    Some(descriptor_set_types.into())
                } else {
                    None
                },
                push_constant_names: if push_constants.len() != 0 {
                    Some(push_constant_names)
                } else {
                    None
                },
            },
            pipeline_data: PipelineData {
                layout: pipeline_layout,
                vertex: vertex_module,
                fragment: fragment_module,
                geometry: geometry_module,
                rasterizer,
                blend_states: blend_states.to_vec().into(),
                depth_stencil,
                name,
            },
        })
    }
}

impl ReflectedGraphics {
    // /// Insert a pipeline to be used with meshes of type V into self and return it
    // ///
    // /// This function will clone the graphics pipeline created and return the copy
    // pub fn pipeline<V: crate::Vertex>(
    //     &self,
    //     device: &gpu::Device,
    // ) -> Result<gpu::GraphicsPipeline, gpu::Error> {
    //     let cache = self.pipelines.read();
    //     if let None = cache.get(&TypeId::of::<V>()) {
    //         drop(cache);
    //         let vertex_state = gpu::VertexState {
    //             stride: std::mem::size_of::<V>() as u32,
    //             input_rate: gpu::VertexInputRate::Vertex,
    //             attributes: &self.vertex_attributes::<V>(),
    //         };

    //         let pipeline = unsafe {
    //             device.create_graphics_pipeline(&gpu::GraphicsPipelineDesc::from_ref(
    //                 self.name.as_ref().map(|a| &**a),
    //                 &self.pipeline_layout,
    //                 self.rasterizer,
    //                 &self.vertex,
    //                 self.fragment.as_ref(),
    //                 self.geometry.as_ref(),
    //                 None,
    //                 &self.blend_states,
    //                 self.blend_constant,
    //                 self.depth_state,
    //                 &[vertex_state],
    //             ))?
    //         };
    //         let mut cache = self.pipelines.write();
    //         cache.insert(TypeId::of::<V>(), pipeline);
    //     } else {
    //         drop(cache)
    //     }
    //     let pipelines = self.pipelines.read();
    //     Ok(pipelines.get(&TypeId::of::<V>()).unwrap().clone())
    // }

    // /// Insert a pipeline to be used with meshes of type V into self and return a reference to it
    // ///
    // /// This function requires a mutable reference to self to be able to insert the pipeline as it by-passes
    // /// the locking mechanisms to be able to match the lifetime of self with the lifetime of the return
    // pub fn pipeline_ref<'a, V: crate::Vertex>(
    //     &'a mut self,
    //     device: &gpu::Device,
    // ) -> Result<&'a gpu::GraphicsPipeline, error::ReflectedError> {
    //     let pipelines = self.pipelines.get_mut();
    //     if let None = pipelines.get(&TypeId::of::<V>()) {
    //         drop(pipelines);
    //         let vertex_state = gpu::VertexState {
    //             stride: std::mem::size_of::<V>() as u32,
    //             input_rate: gpu::VertexInputRate::Vertex,
    //             attributes: &self.vertex_attributes::<V>(),
    //         };

    //         let pipeline = unsafe {
    //             device.create_graphics_pipeline(&gpu::GraphicsPipelineDesc {
    //                 name: self.name.as_ref().map(|a| &**a),
    //                 layout: &self.pipeline_layout,
    //                 rasterizer: self.rasterizer,
    //                 vertex: &self.vertex,
    //                 fragment: self.fragment.as_ref(),
    //                 geometry: self.geometry.as_ref(),
    //                 tessellation: None,
    //                 blend_states: &self.blend_states,
    //                 blend_constant: self.blend_constant,
    //                 depth_state: self.depth_state,
    //                 vertex_states: &[vertex_state],
    //             })?
    //         };
    //         let pipelines = self.pipelines.get_mut();
    //         pipelines.insert(TypeId::of::<V>(), pipeline);
    //     }
    //     let pipelines = self.pipelines.get_mut();
    //     Ok(pipelines.get(&TypeId::of::<V>()).unwrap())
    // }

    /// Create a new BundleBuilder for this Graphics
    ///
    /// Returns none if the shaders stages have no binding
    pub fn bundle(&self) -> Option<BundleBuilder<'_>> {
        if self.reflect_data.descriptor_set_layouts.is_some() {
            Some(BundleBuilder {
                parent_id: self.id,
                parent_name: self.pipeline_data.name.clone(),
                map: self.reflect_data.descriptor_set_map.as_ref().unwrap(),
                types: self.reflect_data.descriptor_set_types.as_ref().unwrap(),
                layouts: self.reflect_data.descriptor_set_layouts.as_ref().unwrap(),
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
        let attribs = self.reflect_data
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
}
