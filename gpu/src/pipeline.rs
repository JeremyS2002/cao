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
    pub vertex: &'a crate::ShaderModule,
    /// the tessellation options
    pub tessellation: Option<crate::Tesselation<'a>>,
    /// the geometry shader, not required
    pub geometry: Option<&'a crate::ShaderModule>,
    /// the fragment shader, not required
    pub fragment: Option<&'a crate::ShaderModule>,
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
    pub fn new(
        device: &crate::Device,
        desc: &GraphicsPipelineDesc<'_>,
    ) -> Result<Self, crate::Error> {
        let mut shaders = Vec::new();
        let mut push_shader = |stage: crate::ShaderStages,
                               module: &crate::ShaderModule|
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
                shaders.push((Arc::clone(&*module.raw), entry, stage));
                Ok(())
            } else {
                panic!("ERROR: Attempt to use shader as stage {:?} with out declaring an entry point for stage", stage);
            }
        };

        push_shader(crate::ShaderStages::VERTEX, desc.vertex)?;

        let tessellation = if let Some(tessellation) = &desc.tessellation {
            push_shader(crate::ShaderStages::TESSELLATION_EVAL, tessellation.eval)?;
            if let Some(control) = tessellation.control {
                push_shader(crate::ShaderStages::TESSELLATION_CONTROL, control)?;
            }
            Some(vk::PipelineTessellationStateCreateInfo {
                s_type: vk::StructureType::PIPELINE_TESSELLATION_STATE_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::PipelineTessellationStateCreateFlags::empty(),
                patch_control_points: tessellation.patch_points.unwrap_or(1),
            })
        } else {
            None
        };

        if let Some(geometry) = &desc.geometry {
            push_shader(crate::ShaderStages::GEOMETRY, geometry)?;
        }
        if let Some(fragment) = &desc.fragment {
            push_shader(crate::ShaderStages::FRAGMENT, fragment)?;
        }

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
        };

        let rasterization_state = desc.rasterizer.into();

        let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineInputAssemblyStateCreateFlags::empty(),
            primitive_restart_enable: vk::FALSE,
            topology: desc.rasterizer.primitive_topology.into(),
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
        };

        let depth_state: Option<vk::PipelineDepthStencilStateCreateInfo> =
            desc.depth_stencil.map(|s| s.into());


        let scissors = desc.viewports.iter().map(|v| vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: vk::Extent2D {
                width: v.width as _,
                height: v.height,
            },
        }).collect::<Vec<_>>();

        let viewports = desc.viewports.iter().map(|v| (*v).into()).collect::<Vec<_>>();

        let viewport_state = vk::PipelineViewportStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineViewportStateCreateFlags::empty(),
            scissor_count: scissors.len() as _,
            p_scissors: scissors.as_ptr(),
            viewport_count: viewports.len() as _,
            p_viewports: viewports.as_ptr(),
        };

        let shader_stages = shaders
            .iter()
            .map(|s| vk::PipelineShaderStageCreateInfo {
                s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::PipelineShaderStageCreateFlags::empty(),
                module: *s.0,
                p_name: s.1.as_ptr(),
                p_specialization_info: ptr::null(),
                stage: s.2.into(),
            })
            .collect::<Vec<_>>();

        let mut create_info = vk::GraphicsPipelineCreateInfo::builder()
            .layout(**desc.layout.raw)
            .viewport_state(&viewport_state)
            .multisample_state(&multisample_state)
            .color_blend_state(&color_blend_state)
            .stages(&shader_stages)
            .vertex_input_state(&vertex_state)
            .input_assembly_state(&input_assembly_state)
            .rasterization_state(&rasterization_state)
            .subpass(0)
            .render_pass(**desc.pass.raw);

        if let Some(depth_stencil) = &depth_state {
            create_info = create_info.depth_stencil_state(depth_stencil);
        }

        if let Some(tessellation_state) = &tessellation {
            create_info = create_info.tessellation_state(tessellation_state);
        }

        let raw_result = unsafe {
            device
                .raw
                .create_graphics_pipelines(vk::PipelineCache::null(), &[*create_info], None)
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
    pub shader: &'a crate::ShaderModule,
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
        let entry = if let Some(entry) = desc.shader.map.iter().fold(None, |t, (s, e)| {
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

        let create_info = vk::ComputePipelineCreateInfo::builder()
            .stage(
                *vk::PipelineShaderStageCreateInfo::builder()
                    .stage(vk::ShaderStageFlags::COMPUTE)
                    .name(&entry)
                    .module(**desc.shader.raw),
            )
            .layout(**desc.layout.raw);

        let raw_result = unsafe {
            device
                .raw
                .create_compute_pipelines(vk::PipelineCache::null(), &[*create_info], None)
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
