//! [`PipelineLayout`] describes the inputs to either a [`GraphicsPipeline`] or [`ComputePipeline`]

use std::mem::ManuallyDrop as Md;
use std::ptr;
use std::sync::Arc;

use ash::vk;

/// Describes a pipeline layout
#[derive(Debug)]
pub struct PipelineLayoutDesc<'a> {
    /// The name of the pipeline layout
    pub name: Option<String>,
    /// All the DescriptorSets that can be attached to the pipeline created from this layout
    pub descriptor_sets: &'a [&'a crate::DescriptorLayout],
    /// All the push constant ranges that can be sent to the pipeline from this layout
    pub push_constants: &'a [crate::PushConstantRange],
}

/// PipelineLayout
///
/// Describes the layout of a pipeline (the DescriptorSets/pushconstants) that can be used
/// <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkPipelineLayout.html>
pub struct PipelineLayout {
    pub(crate) name: Option<String>,
    pub(crate) raw: Md<Arc<vk::PipelineLayout>>,
    pub(crate) device: Arc<crate::RawDevice>,
}

impl std::hash::Hash for PipelineLayout {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (**self.raw).hash(state)
    }
}

impl PartialEq for PipelineLayout {
    fn eq(&self, other: &PipelineLayout) -> bool {
        **self.raw == **other.raw
    }
}

impl Eq for PipelineLayout {}

impl Clone for PipelineLayout {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            raw: Md::new(Arc::clone(&self.raw)),
            device: Arc::clone(&self.device),
        }
    }
}

impl std::fmt::Debug for PipelineLayout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PipelineLayout id: {:?} name: {:?}",
            **self.raw, self.name
        )
    }
}

impl PipelineLayout {
    pub unsafe fn raw_pipeline_layout(&self) -> vk::PipelineLayout {
        **self.raw
    }
}

impl PipelineLayout {
    /// Create a new PipelineLayout
    pub fn new(
        device: &crate::Device,
        desc: &PipelineLayoutDesc<'_>,
    ) -> Result<Self, crate::Error> {
        let descriptor_sets = desc
            .descriptor_sets
            .iter()
            .map(|b| **b.raw)
            .collect::<Vec<_>>();
        let push_constants = desc
            .push_constants
            .iter()
            .map(|p| (*p).into())
            .collect::<Vec<_>>();

        let create_info = vk::PipelineLayoutCreateInfo {
            s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineLayoutCreateFlags::empty(),
            set_layout_count: descriptor_sets.len() as u32,
            p_set_layouts: descriptor_sets.as_ptr(),
            push_constant_range_count: push_constants.len() as u32,
            p_push_constant_ranges: push_constants.as_ptr(),
            ..Default::default()
        };

        let raw_result = unsafe { device.raw.create_pipeline_layout(&create_info, None) };

        let raw = match raw_result {
            Ok(r) => r,
            Err(e) => return Err(e.into()),
        };

        let s = Self {
            name: desc.name.as_ref().map(|s| s.to_string()),
            raw: Md::new(Arc::new(raw)),
            device: Arc::clone(&device.raw),
        };

        if let Some(name) = &desc.name {
            device.raw.set_pipeline_layout_name(&s, name.as_ref())?;
        }

        device.raw.check_errors()?;

        Ok(s)
    }

    /// Get the name of self
    pub fn name(&self) -> Option<&str> {
        self.name.as_ref().map(|n| &**n)
    }
}

impl Drop for PipelineLayout {
    fn drop(&mut self) {
        unsafe {
            let raw = Md::take(&mut self.raw);
            if let Ok(raw) = Arc::try_unwrap(raw) {
                self.device.destroy_pipeline_layout(raw, None);
            }
        }
    }
}

#[derive(Debug)]
pub struct PipelineCacheDesc<'a> {
    pub name: Option<String>,
    pub initial_data: Option<&'a [u8]>,
}

impl<'a> std::default::Default for PipelineCacheDesc<'a> {
    fn default() -> Self {
        Self { name: Default::default(), initial_data: Default::default() }
    }
}

/// Object used for caching [`GraphicsPipeline`] or [`ComputePipeline`] compilation
/// 
/// If you are using multiple similar pipelines then creating each one with the same pipeline cache
/// can save time of subsequent pipeline creation as some compilation has already been done
/// <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkPipelineCache.html>
pub struct PipelineCache {
    pub(crate) raw: Md<Arc<vk::PipelineCache>>,
    pub(crate) device: Arc<crate::RawDevice>,
    pub(crate) name: Option<String>,
}

impl std::hash::Hash for PipelineCache {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (**self.raw).hash(state)
    }
}

impl PartialEq for PipelineCache {
    fn eq(&self, other: &PipelineCache) -> bool {
        **self.raw == **other.raw
    }
}

impl Eq for PipelineCache {}

impl Clone for PipelineCache {
    fn clone(&self) -> Self {
        Self {
            raw: Md::new(Arc::clone(&self.raw)),
            device: Arc::clone(&self.device),
            name: self.name.clone(),
        }
    }
}

impl std::fmt::Debug for PipelineCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PipelineCache {:?}, name: {:?}",
            **self.raw, self.name
        )
    }
}

impl PipelineCache {
    pub unsafe fn raw_cache(&self) -> vk::PipelineCache {
        **self.raw
    }
}

impl PipelineCache {
    pub fn new(device: &crate::Device, desc: &PipelineCacheDesc<'_>) -> Result<Self, crate::Error> {
        let create_info = vk::PipelineCacheCreateInfo {
            s_type: vk::StructureType::PIPELINE_CACHE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineCacheCreateFlags::empty(),
            initial_data_size: if let Some(data) = desc.initial_data {
                data.len() as _
            } else {
                0
            },
            p_initial_data: if let Some(data) = desc.initial_data {
                data.as_ptr() as *const _
            } else {
                ptr::null()
            },
            ..Default::default()
        };

        let result = unsafe {
            device.raw.create_pipeline_cache(&create_info, None)
        };

        let raw = match result {
            Ok(r) => r,
            Err(e) => return Err(e.into()),
        };

        let s = Self {
            raw: Md::new(Arc::new(raw)),
            device: Arc::clone(&device.raw),
            name: desc.name.clone(),
        };

        if let Some(name) = &desc.name {
            device.raw.set_pipeline_cache_name(&s, name)?;
        }

        device.raw.check_errors()?;

        Ok(s)
    }

    /// merge the other pipeline caches into self
    /// 
    /// None of the caches in others should be self
    pub fn merge<'a>(&self, others: impl IntoIterator<Item = &'a PipelineCache>) -> Result<(), crate::Error> {
        let src = others.into_iter().map(|p| **p.raw).collect::<Vec<_>>();

        let result = unsafe {
            self.device.merge_pipeline_caches(**self.raw, &src)
        };

        match result {
            Ok(_) => (),
            Err(e) => return Err(e.into()),
        }

        Ok(self.device.check_errors()?)
    }

    /// Get the cached data from self
    /// 
    /// Typical usage would be storing this to disk then loading on next run of the application into initial_data
    pub fn get_data(&self) -> Result<Vec<u8>, crate::Error> {
        let result = unsafe {
            self.device.get_pipeline_cache_data(**self.raw)
        };

        let data = match result {
            Ok(d) => d,
            Err(e) => return Err(e.into())
        };

        self.device.check_errors()?;

        Ok(data)
    }
}

impl Drop for PipelineCache {
    fn drop(&mut self) {
        let raw = unsafe { Md::take(&mut self.raw) };
        if let Ok(raw) = Arc::try_unwrap(raw) {
            unsafe {
                self.device.destroy_pipeline_cache(raw, None);
            }
        }
    }
}

/// Describes a GraphicsPipeline
#[derive(Debug)]
pub struct GraphicsPipelineDesc<'a> {
    /// the name of the graphics pipeline
    pub name: Option<String>,
    /// the layout of the pipeline
    pub layout: &'a PipelineLayout,
    /// the pass of the pipeline,
    pub pass: &'a crate::RenderPass,
    /// the vertex shader for the pipeline operates on each vertex input
    pub vertex: (&'a crate::ShaderModule, Option<crate::Specialization<'a>>),
    /// the tessellation options
    pub tessellation: Option<crate::Tesselation<'a>>,
    /// the geometry shader, not required
    pub geometry: Option<(&'a crate::ShaderModule, Option<crate::Specialization<'a>>)>,
    /// the fragment shader, not required
    pub fragment: Option<(&'a crate::ShaderModule, Option<crate::Specialization<'a>>)>,
    /// the rasterizer for this pipeline
    pub rasterizer: crate::Rasterizer,
    /// the vertex buffers that the pipeline takes
    pub vertex_states: &'a [crate::VertexState<'a>],
    /// how the color attachments are blended
    pub blend_states: &'a [crate::BlendState],
    /// how the depth testing should be performed
    pub depth_stencil: Option<crate::DepthStencilState>,
    /// what portion of the texture to render to
    pub viewports: &'a [crate::Viewport],
    /// cached pipeline creation data
    pub cache: Option<&'a PipelineCache>,
}

/// A GraphicsPipeline
///
/// Determins how draw command are executed, contains shaders modules to be run and other
/// infomation on how the rasterization and blending should be performed
/// <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkPipeline.html>
pub struct GraphicsPipeline {
    pub(crate) name: Option<String>,
    pub(crate) layout: PipelineLayout,
    pub(crate) pass: crate::RenderPass,
    pub(crate) raw: Md<Arc<vk::Pipeline>>,
    pub(crate) device: Arc<crate::RawDevice>,
}

impl std::hash::Hash for GraphicsPipeline {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (**self.raw).hash(state)
    }
}

impl PartialEq for GraphicsPipeline {
    fn eq(&self, other: &GraphicsPipeline) -> bool {
        **self.raw == **other.raw
    }
}

impl Eq for GraphicsPipeline {}

impl Clone for GraphicsPipeline {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            layout: self.layout.clone(),
            pass: self.pass.clone(),
            raw: Md::new(Arc::clone(&self.raw)),
            device: Arc::clone(&self.device),
        }
    }
}

impl std::fmt::Debug for GraphicsPipeline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "GraphicsPipeline {:?}, name: {:?}",
            **self.raw, self.name
        )
    }
}

impl GraphicsPipeline {
    pub unsafe fn raw_pipeline(&self) -> vk::Pipeline {
        **self.raw
    }
}

impl GraphicsPipeline {
    /// Create a new GraphicsPipeline
    ///
    /// # Safety
    ///
    /// The shaders must be compatible with the pipeline layout
    /// If in doubt enable the "VK_LAYER_KHRONOS_validation" validation layer
    /// And check for messages when the pipeline is created
    pub fn new<'a>(
        device: &crate::Device,
        desc: &GraphicsPipelineDesc<'a>,
    ) -> Result<Self, crate::Error> {
        let mut shaders = Vec::new();
        let mut spec_entries = Vec::new();
        let mut push_shader = |stage: crate::ShaderStages,
                               module: &crate::ShaderModule, spec: Option<crate::Specialization<'a>>|
         -> Result<(), crate::Error> {
            if let Some(entry) = module.map.iter().fold(None, |t, (s, e)| {
                if t.is_some() {
                    t
                } else if s.contains(stage) {
                    Some(e.clone())
                } else {
                    None
                }
            }) {
                if let Some(s) = spec {
                    let map_entries = s.entries.iter().map(|e| vk::SpecializationMapEntry {
                        constant_id: e.id,
                        offset: e.offset,
                        size: e.size,
                    }).collect::<Vec<_>>();
                    spec_entries.push((map_entries, s.data));

                    shaders.push((Arc::clone(&*module.raw), entry, stage, Some(spec_entries.len() - 1)));
                } else {
                    shaders.push((Arc::clone(&*module.raw), entry, stage, None));
                }
                
                Ok(())
            } else {
                panic!("ERROR: Attempt to use shader as stage {:?} with out declaring an entry point for stage", stage);
            }
        };

        push_shader(crate::ShaderStages::VERTEX, &desc.vertex.0, desc.vertex.1)?;

        let tessellation = if let Some(tessellation) = &desc.tessellation {
            push_shader(crate::ShaderStages::TESSELLATION_EVAL, tessellation.eval.0, tessellation.eval.1)?;
            if let Some(control) = tessellation.control {
                push_shader(crate::ShaderStages::TESSELLATION_CONTROL, control.0, control.1)?;
            }
            Some(vk::PipelineTessellationStateCreateInfo {
                s_type: vk::StructureType::PIPELINE_TESSELLATION_STATE_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::PipelineTessellationStateCreateFlags::empty(),
                patch_control_points: tessellation.patch_points.unwrap_or(1),
                ..Default::default()
            })
        } else {
            None
        };

        if let Some(geometry) = &desc.geometry {
            push_shader(crate::ShaderStages::GEOMETRY, geometry.0, geometry.1)?;
        }
        if let Some(fragment) = &desc.fragment {
            push_shader(crate::ShaderStages::FRAGMENT, fragment.0, fragment.1)?;
        }

        let spec_info = spec_entries.iter().map(|(e, d)| vk::SpecializationInfo {
            map_entry_count: e.len() as _,
            p_map_entries: e.as_ptr(),
            data_size: d.len(),
            p_data: d.as_ptr() as *const _,
            ..Default::default()
        }).collect::<Vec<_>>();

        let shader_stages = shaders
            .iter()
            .map(|s| vk::PipelineShaderStageCreateInfo {
                s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::PipelineShaderStageCreateFlags::empty(),
                module: *s.0,
                p_name: s.1.as_ptr(),
                stage: s.2.into(),
                p_specialization_info: if let Some(idx) = s.3 {
                    spec_info.get(idx).unwrap()
                } else {
                    ptr::null()
                },
                ..Default::default()
            })
            .collect::<Vec<_>>();

        let vertex_states = desc
            .vertex_states
            .iter()
            .enumerate()
            .map(|(i, s)| vk::VertexInputBindingDescription {
                binding: i as u32,
                stride: s.stride,
                input_rate: s.input_rate.into(),
            })
            .collect::<Vec<_>>();
        let vertex_attributes = desc
            .vertex_states
            .iter()
            .enumerate()
            .map(|(i, s)| {
                s.attributes
                    .iter()
                    .map(|a| vk::VertexInputAttributeDescription {
                        binding: i as u32,
                        location: a.location,
                        format: a.format.into(),
                        offset: a.offset,
                    })
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect::<Vec<_>>();

        let vertex_state = vk::PipelineVertexInputStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineVertexInputStateCreateFlags::empty(),
            vertex_attribute_description_count: vertex_attributes.len() as u32,
            p_vertex_attribute_descriptions: vertex_attributes.as_ptr(),
            vertex_binding_description_count: vertex_states.len() as u32,
            p_vertex_binding_descriptions: vertex_states.as_ptr(),
            ..Default::default()
        };

        let rasterization_state = desc.rasterizer.into();

        let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineInputAssemblyStateCreateFlags::empty(),
            primitive_restart_enable: vk::FALSE,
            topology: desc.rasterizer.primitive_topology.into(),
            ..Default::default()
        };

        let multisample_state = vk::PipelineMultisampleStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineMultisampleStateCreateFlags::empty(),
            rasterization_samples: desc.pass.samples.into(),
            ..Default::default()
        };

        let blend_states = desc
            .blend_states
            .iter()
            .map(|s| (*s).into())
            .collect::<Vec<vk::PipelineColorBlendAttachmentState>>();

        let color_blend_state = vk::PipelineColorBlendStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineColorBlendStateCreateFlags::empty(),
            logic_op_enable: vk::FALSE,
            logic_op: vk::LogicOp::COPY,
            attachment_count: desc.blend_states.len() as u32,
            p_attachments: blend_states.as_ptr(),
            blend_constants: [0.0; 4],
            ..Default::default()
        };

        let depth_state: Option<vk::PipelineDepthStencilStateCreateInfo> =
            desc.depth_stencil.map(|s| s.into());

        let scissors = desc
            .viewports
            .iter()
            .map(|v| vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: vk::Extent2D {
                    width: v.width as _,
                    height: v.height,
                },
            })
            .collect::<Vec<_>>();

        let viewports = desc
            .viewports
            .iter()
            .map(|v| (*v).into())
            .collect::<Vec<_>>();

        let viewport_state = vk::PipelineViewportStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineViewportStateCreateFlags::empty(),
            scissor_count: scissors.len() as _,
            p_scissors: scissors.as_ptr(),
            viewport_count: viewports.len() as _,
            p_viewports: viewports.as_ptr(),
            ..Default::default()
        };

        let create_info = vk::GraphicsPipelineCreateInfo {
            s_type: vk::StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineCreateFlags::empty(),
            stage_count: shader_stages.len() as _,
            p_stages: shader_stages.as_ptr(),
            p_vertex_input_state: &vertex_state,
            p_input_assembly_state: &input_assembly_state,
            p_tessellation_state: if let Some(t) = &tessellation {
                t
            } else {
                ptr::null()
            },
            p_viewport_state: &viewport_state,
            p_rasterization_state: &rasterization_state,
            p_multisample_state: &multisample_state,
            p_depth_stencil_state: if let Some(d) = &depth_state {
                d
            } else {
                ptr::null()
            },
            p_color_blend_state: &color_blend_state,
            p_dynamic_state: ptr::null(),
            layout: **desc.layout.raw,
            render_pass: **desc.pass.raw,
            subpass: 0,
            base_pipeline_handle: vk::Pipeline::null(),
            base_pipeline_index: 0,
            ..Default::default()
        };

        // let mut create_info = vk::GraphicsPipelineCreateInfo::builder()
        //     .layout(**desc.layout.raw)
        //     .viewport_state(&viewport_state)
        //     .multisample_state(&multisample_state)
        //     .color_blend_state(&color_blend_state)
        //     .stages(&shader_stages)
        //     .vertex_input_state(&vertex_state)
        //     .input_assembly_state(&input_assembly_state)
        //     .rasterization_state(&rasterization_state)
        //     .subpass(0)
        //     .render_pass(**desc.pass.raw);

        // if let Some(depth_stencil) = &depth_state {
        //     create_info = create_info.depth_stencil_state(depth_stencil);
        // }

        // if let Some(tessellation_state) = &tessellation {
        //     create_info = create_info.tessellation_state(tessellation_state);
        // }

        let raw_result = unsafe {
            device
                .raw
                .create_graphics_pipelines(vk::PipelineCache::null(), &[create_info], None)
        };

        let raw = match raw_result {
            Ok(r) => r[0],
            Err((_, e)) => return Err(e.into()),
        };

        let s = Self {
            name: desc.name.as_ref().map(|s| s.to_string()),
            layout: desc.layout.clone(),
            pass: desc.pass.clone(),
            raw: Md::new(Arc::new(raw)),
            device: Arc::clone(&device.raw),
        };

        if let Some(name) = &desc.name {
            device.raw.set_graphics_pipeline_name(&s, name.as_ref())?;
        }

        device.raw.check_errors()?;

        Ok(s)
    }

    /// Get a reference to the pipeline layout that self was created from
    pub fn layout<'a>(&'a self) -> &'a PipelineLayout {
        &self.layout
    }

    /// Get a reference to the render pass that self was created from
    pub fn pass<'a>(&'a self) -> &'a crate::RenderPass {
        &self.pass
    }

    /// Get the name of self
    pub fn name(&self) -> Option<&str> {
        self.name.as_ref().map(|n| &**n)
    }
}

impl Drop for GraphicsPipeline {
    fn drop(&mut self) {
        unsafe {
            let raw = Md::take(&mut self.raw);
            if let Ok(raw) = Arc::try_unwrap(raw) {
                self.device.destroy_pipeline(raw, None);
            }
        }
    }
}

/// Describes a ComputePipeline
#[derive(Debug)]
pub struct ComputePipelineDesc<'a> {
    /// The name of the compute pipeline
    pub name: Option<String>,
    /// The layout of the pipeline
    pub layout: &'a PipelineLayout,
    /// The shader module to be executed
    pub shader: (&'a crate::ShaderModule, Option<crate::Specialization<'a>>),
    /// Cached data to be reused
    pub cache: Option<&'a PipelineCache>,
}

/// A ComputePipeline
///
/// Describes how dispatch operations are performed
/// <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkPipeline.html>
pub struct ComputePipeline {
    pub(crate) name: Option<String>,
    pub(crate) layout: PipelineLayout,
    pub(crate) raw: Md<Arc<vk::Pipeline>>,
    pub(crate) device: Arc<crate::RawDevice>,
}

impl std::hash::Hash for ComputePipeline {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (**self.raw).hash(state)
    }
}

impl PartialEq for ComputePipeline {
    fn eq(&self, other: &ComputePipeline) -> bool {
        **self.raw == **other.raw
    }
}

impl Eq for ComputePipeline {}

impl Clone for ComputePipeline {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            layout: self.layout.clone(),
            raw: Md::new(Arc::clone(&self.raw)),
            device: Arc::clone(&self.device),
        }
    }
}

impl std::fmt::Debug for ComputePipeline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ComputePipeline id: {:?} name: {:?}",
            **self.raw, self.name
        )
    }
}

impl ComputePipeline {
    pub unsafe fn raw_pipeline(&self) -> vk::Pipeline {
        **self.raw
    }
}

impl ComputePipeline {
    /// Create a new ComputePipeline
    pub fn new(
        device: &crate::Device,
        desc: &ComputePipelineDesc<'_>,
    ) -> Result<Self, crate::Error> {
        let entry = if let Some(entry) = desc.shader.0.map.iter().fold(None, |t, (s, e)| {
            if t.is_some() {
                t
            } else if s.contains(crate::ShaderStages::COMPUTE) {
                Some(e.clone())
            } else {
                None
            }
        }) {
            entry
        } else {
            panic!(
                "ERROR: Attempt to use shader as type {:?} without declaring entry point",
                crate::ShaderStages::COMPUTE
            );
        };

        let spec_entries = desc.shader.1.as_ref().map(|s| {
            (
                s.entries.iter().map(|e| vk::SpecializationMapEntry {
                    constant_id: e.id,
                    offset: e.offset,
                    size: e.size,
                }).collect::<Vec<_>>(),
                s.data,
            )
        });

        let spec = spec_entries.as_ref().map(|(e, d)| vk::SpecializationInfo {
            map_entry_count: e.len() as _,
            p_map_entries: e.as_ptr(),
            data_size: d.len() as _,
            p_data: d.as_ptr() as *const _,
            ..Default::default()
        });

        let shader_stage = vk::PipelineShaderStageCreateInfo {
            s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineShaderStageCreateFlags::empty(),
            module: **desc.shader.0.raw,
            p_name: entry.as_ptr(),
            stage: vk::ShaderStageFlags::COMPUTE,
            p_specialization_info: if let Some(s) = spec {
                &s
            } else {
                ptr::null()
            },
            ..Default::default()
        };

        // let create_info = vk::ComputePipelineCreateInfo::builder()
        //     .stage(shader_stage)
        //     .layout(**desc.layout.raw);

        let create_info = vk::ComputePipelineCreateInfo {
            s_type: vk::StructureType::COMPUTE_PIPELINE_CREATE_INFO,
            stage: shader_stage,
            p_next: ptr::null(),
            flags: vk::PipelineCreateFlags::empty(),
            layout: **desc.layout.raw,
            base_pipeline_handle: vk::Pipeline::null(),
            base_pipeline_index: -1, // null
            ..Default::default()
        };

        let raw_result = unsafe {
            device
                .raw
                .create_compute_pipelines(vk::PipelineCache::null(), &[create_info], None)
        };
        let raw = match raw_result {
            Ok(r) => r[0],
            Err((_, e)) => return Err(e.into()),
        };

        let s = Self {
            name: desc.name.as_ref().map(|s| s.to_string()),
            layout: desc.layout.clone(),
            raw: Md::new(Arc::new(raw)),
            device: Arc::clone(&device.raw),
        };

        if let Some(name) = &desc.name {
            device.raw.set_compute_pipeline_name(&s, name.as_ref())?;
        }

        device.raw.check_errors()?;

        Ok(s)
    }

    /// Get a reference to the pipeline layout that self was created from
    pub fn layout<'a>(&'a self) -> &'a PipelineLayout {
        &self.layout
    }

    /// Get the name of self
    pub fn name(&self) -> Option<&str> {
        self.name.as_ref().map(|n| &**n)
    }
}

impl Drop for ComputePipeline {
    fn drop(&mut self) {
        unsafe {
            let raw = Md::take(&mut self.raw);
            if let Ok(raw) = Arc::try_unwrap(raw) {
                self.device.destroy_pipeline(raw, None);
            }
        }
    }
}
