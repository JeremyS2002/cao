//! Utility for recording into a [`gpu::CommandBuffer`]
//!
//! [`CommandEncoder`]'s store a list of commands that they then record into a CommandBuffer.
//!
//! CommmandEncoders main utility comes from managing texture layouts automatically
//! They also manage pipeline barriers and allow for reflected pipelines to be much more useful
//!
//! [`CommandEncoder::record`] formats the encoders commands then begins the command buffer, records commands and ends the buffer
//! [`CommandEncoder::submit`] does the same as record but submits the command buffer afterwards

#[cfg(feature = "reflect")]
use std::any::TypeId;
use std::borrow::Cow;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::Hash;
use std::mem::ManuallyDrop as Md;

pub mod command;

pub use command::Command;

pub struct CommandEncoder<'a> {
    pub(crate) formatted: bool,
    pub(crate) commands: Vec<Command<'a>>,
}

impl<'a> CommandEncoder<'a> {
    /// Create a new CommandEncoder
    pub fn new() -> Self {
        Self {
            // device,
            formatted: false,
            commands: Vec::new(),
        }
    }

    /// Record the commands into a command buffer
    pub fn record(
        &mut self,
        command_buffer: &mut gpu::CommandBuffer,
        one_time_submit: bool,
    ) -> Result<(), gpu::Error> {
        unsafe {
            if !self.formatted {
                self.format();
            }

            command_buffer.begin(one_time_submit)?;
            for command in &self.commands {
                command.execute(command_buffer)?;
            }

            command_buffer.end()?;
        }

        Ok(())
    }

    /// Record the commands into a command buffer and then submit
    pub fn submit(
        &mut self,
        command_buffer: &mut gpu::CommandBuffer,
        one_time_submit: bool,
    ) -> Result<(), gpu::Error> {
        let proof = self.record(command_buffer, one_time_submit)?;
        command_buffer.submit()?;
        Ok(proof)
    }

    /// Push a command to the end of self
    pub fn push_command(&mut self, command: Command<'a>) {
        let textures = command
            .textures()
            .into_iter()
            .map(|((t, mip, array), l)| {
                // transition the texture to the desired layout leaving other details of the barrier to be filled in later
                gpu::TextureAccessInfo {
                    src_access: gpu::AccessFlags::empty(),
                    dst_access: gpu::AccessFlags::empty(),
                    src_layout: t.initial_layout(),
                    dst_layout: l,
                    base_array_layer: array,
                    array_layers: 1,
                    base_mip_level: mip,
                    mip_levels: 1,
                    texture: Cow::Owned(t),
                }
            })
            .collect::<Vec<_>>();
        let buffers = command
            .buffers()
            .into_iter()
            .map(|b| gpu::BufferAccessInfo {
                buffer: b,
                src_access: gpu::AccessFlags::empty(),
                dst_access: gpu::AccessFlags::empty(),
            })
            .collect::<Vec<_>>();
        if textures.len() != 0 || buffers.len() != 0 {
            self.commands.push(Command::PipelineBarrier {
                src_stage: gpu::PipelineStageFlags::TOP_OF_PIPE,
                dst_stage: gpu::PipelineStageFlags::BOTTOM_OF_PIPE,
                textures,
                buffers,
            })
        }
        self.commands.push(command);
        self.formatted = false;
    }

    /// Execute a secondary command buffer
    // pub fn execute_secondary(&mut self, secondary: &'a gpu::SecondaryCommandBuffer) {
    //     self.push_command(Command::ExecuteSecondary(secondary))
    // }

    /// Update the buffer by reference
    pub fn update_buffer_ref(&mut self, buffer: &'a gpu::Buffer, offset: u64, data: &'a [u8]) {
        self.push_command(Command::UpdateBuffer {
            buffer: Cow::Borrowed(buffer),
            offset,
            data: Cow::Borrowed(data),
        })
    }

    /// Update the buffer by ownership
    pub fn update_buffer_owned(&mut self, buffer: gpu::Buffer, offset: u64, data: Vec<u8>) {
        self.push_command(Command::UpdateBuffer {
            buffer: Cow::Owned(buffer),
            offset,
            data: Cow::Owned(data),
        })
    }

    /// Clear the texture owning it
    pub fn clear_texture(&mut self, texture: gpu::TextureSlice<'a>, value: gpu::ClearValue) {
        self.push_command(Command::ClearTexture {
            texture,
            layout: gpu::TextureLayout::General,
            value,
        })
    }

    /// blit the src to the dst
    pub fn blit_textures(
        &mut self,
        src: gpu::TextureSlice<'a>,
        dst: gpu::TextureSlice<'a>,
        filter: gpu::FilterMode,
    ) {
        self.push_command(Command::BlitTextures {
            src,
            src_layout: gpu::TextureLayout::CopySrcOptimal,
            dst,
            dst_layout: gpu::TextureLayout::CopyDstOptimal,
            filter,
        })
    }

    /// copy the src buffer to the dst buffer taking ownership of the buffers
    pub fn copy_buffer_to_buffer(&mut self, src: gpu::BufferSlice<'a>, dst: gpu::BufferSlice<'a>) {
        self.push_command(Command::CopyBufferToBuffer { src, dst })
    }

    /// copy the src Texture to the dst buffer
    pub fn copy_texture_to_buffer(
        &mut self,
        src: gpu::TextureSlice<'a>,
        dst: gpu::BufferSlice<'a>,
    ) {
        self.push_command(Command::CopyTextureToBuffer {
            src,
            src_layout: gpu::TextureLayout::CopySrcOptimal,
            dst,
        })
    }

    /// copy the src buffer to the dst Texture
    pub fn copy_buffer_to_texture(
        &mut self,
        src: gpu::BufferSlice<'a>,
        dst: gpu::TextureSlice<'a>,
    ) {
        self.push_command(Command::CopyBufferToTexture {
            src,
            dst,
            dst_layout: gpu::TextureLayout::CopyDstOptimal,
        })
    }

    /// copy the src Texture to the dst Texture
    pub fn copy_texture_to_texture(
        &mut self,
        src: gpu::TextureSlice<'a>,
        dst: gpu::TextureSlice<'a>,
    ) {
        self.push_command(Command::CopyTextureToTexture {
            src,
            src_layout: gpu::TextureLayout::CopySrcOptimal,
            dst,
            dst_layout: gpu::TextureLayout::CopyDstOptimal,
        })
    }

    /// resolve the src texture into the dst texture by reference
    pub fn resolve_texture(&mut self, src: gpu::TextureSlice<'a>, dst: gpu::TextureSlice<'a>) {
        self.push_command(Command::ResolveTextures {
            src,
            src_layout: gpu::TextureLayout::CopySrcOptimal,
            dst,
            dst_layout: gpu::TextureLayout::CopyDstOptimal,
        })
    }

    /// begin a graphics pass
    pub fn graphics_pass_ref<'b>(
        &'b mut self,
        color_attachments: &'a [gpu::Attachment<'a>],
        resolve_attachments: &'a [gpu::Attachment<'a>],
        depth_attachment: Option<gpu::Attachment<'a>>,
        pipeline: &'a gpu::GraphicsPipeline,
    ) -> Result<crate::pass::BasicGraphicsPass<'a, 'b>, gpu::Error> {
        Ok(crate::pass::BasicGraphicsPass {
            color_attachments: Cow::Borrowed(color_attachments),
            resolve_attachments: Cow::Borrowed(resolve_attachments),
            depth_attachment,
            pipeline: Md::new(Cow::Borrowed(pipeline)),
            commands: Vec::new(),
            encoder: self,
        })
    }

    /// begin a graphics pass
    pub fn graphics_pass_owned<'b>(
        &'b mut self,
        color_attachments: &[gpu::Attachment<'a>],
        resolve_attachments: &[gpu::Attachment<'a>],
        depth_attachment: Option<gpu::Attachment<'a>>,
        pipeline: gpu::GraphicsPipeline,
    ) -> Result<crate::pass::BasicGraphicsPass<'a, 'b>, gpu::Error> {
        Ok(crate::pass::BasicGraphicsPass {
            color_attachments: Cow::Owned(Vec::from(color_attachments)),
            resolve_attachments: Cow::Owned(Vec::from(resolve_attachments)),
            depth_attachment,
            pipeline: Md::new(Cow::Owned(pipeline)),
            commands: Vec::new(),
            encoder: self,
        })
    }

    /// Begin a reflected graphics pass owning the data
    #[cfg(any(feature = "reflect", feature = "spirv"))]
    pub fn graphics_pass_reflected<'b, V: crate::Vertex>(
        &'b mut self,
        device: &gpu::Device,
        colors: &[crate::Attachment<'a>],
        resolves: &[crate::Attachment<'a>],
        depth: Option<crate::Attachment<'a>>,
        graphics: &crate::reflect::ReflectedGraphics,
    ) -> Result<crate::pass::ReflectedGraphicsPass<'a, 'b, V>, gpu::Error> {
        use std::hash::Hasher;

        if colors.len() > graphics.pipeline_data.blend_states.len() {
            panic!("Graphics Pipeline {:?} doesn't have enough blend states to begin pass with {} color attachments", graphics, colors.len());
        }

        let extent = if colors.len() != 0 {
            colors[0].raw.view().extent()
        } else if let Some(d) = depth.as_ref() {
            d.raw.view().extent()
        } else {
            panic!("Cannot begin graphics pass with no color or depth attachments");
        };

        let samples = if colors.len() != 0 {
            colors[0].raw.view().samples()
        } else if let Some(d) = depth.as_ref() {
            d.raw.view().samples()
        } else {
            panic!("Cannot begin graphics pass with no color or depth attachments");
        };

        let colors_desc = colors
            .iter()
            .map(|a| gpu::ColorAttachmentDesc {
                format: a.raw.view().format(),
                load: a.load,
                store: a.store,
                initial_layout: gpu::TextureLayout::ColorAttachmentOptimal,
                // for normal textures will just return General but for swapchain will return SwapchainPresent
                final_layout: a.raw.view().texture().initial_layout(),
            })
            .collect::<Vec<_>>();

        let resolves_desc = resolves
            .iter()
            .map(|a| gpu::ResolveAttachmentDesc {
                load: a.load,
                store: a.store,
                initial_layout: gpu::TextureLayout::ColorAttachmentOptimal,
                final_layout: a.raw.view().texture().initial_layout(),
            })
            .collect::<Vec<_>>();

        let depth_desc = depth.as_ref().map(|a| gpu::DepthAttachmentDesc {
            format: a.raw.view().format(),
            load: a.load,
            store: a.store,
            initial_layout: gpu::TextureLayout::DepthStencilAttachmentOptimal,
            final_layout: a.raw.view().texture().initial_layout(),
        });

        let mut hasher = DefaultHasher::new();
        colors_desc.hash(&mut hasher);
        resolves_desc.hash(&mut hasher);
        depth_desc.hash(&mut hasher);
        let pass_hash = hasher.finish();

        let viewport = gpu::Viewport {
            x: 0,
            y: 0,
            width: extent.width as _,
            height: extent.height as _,
            ..Default::default()
        };

        let c = graphics.pipeline_map.read();

        let key = crate::reflect::graphics::GraphicsPipelineKey {
            pass_hash,
            vertex_ty: TypeId::of::<V>(),
            viewport,
        };

        if let None = c.get(&key) {
            drop(c);
            let pass_name = graphics
                .pipeline_data
                .name
                .as_ref()
                .map(|n| format!("{}_pass_{}", n, pass_hash));

            let pass = device.create_render_pass(&gpu::RenderPassDesc {
                name: pass_name,
                colors: &colors_desc,
                resolves: &resolves_desc,
                depth: depth_desc,
                samples,
            })?;

            let vertex_state = gpu::VertexState {
                stride: std::mem::size_of::<V>() as u32,
                input_rate: gpu::VertexInputRate::Vertex,
                attributes: &graphics.vertex_attributes::<V>(),
            };

            let vertex_states = &[vertex_state];

            let pipeline_name = graphics
                .pipeline_data
                .name
                .as_ref()
                .map(|n| format!("{}_pipeline", n));

            let mut desc = gpu::GraphicsPipelineDesc {
                name: pipeline_name,
                layout: &graphics.pipeline_data.layout,
                pass: &pass,
                vertex: (&graphics.pipeline_data.vertex, None),
                tessellation: None,
                geometry: graphics.pipeline_data.geometry.as_ref().map(|s| (s, None)),
                fragment: graphics.pipeline_data.fragment.as_ref().map(|s| (s, None)),
                rasterizer: graphics.pipeline_data.rasterizer,
                vertex_states,
                blend_states: &graphics.pipeline_data.blend_states[..colors.len()],
                depth_stencil: graphics.pipeline_data.depth_stencil,
                viewports: &[viewport],
                cache: None,
            };

            if std::mem::size_of::<V>() == 0 {
                desc.vertex_states = &[];
            }

            let pipeline = device.create_graphics_pipeline(&desc)?;
            graphics.pipeline_map.write().insert(key, pipeline);
        }

        let pipeline_map = graphics.pipeline_map.read();
        let pipeline = pipeline_map.get(&key).unwrap();

        Ok(crate::pass::ReflectedGraphicsPass {
            parent_id: graphics.id,
            bundle_needed: graphics.bundle_needed(),
            push_constant_names: graphics.reflect_data.push_constant_names.clone(),
            color_attachments: colors.to_vec(),
            resolve_attachments: resolves.to_vec(),
            depth_attachment: depth,
            pipeline: Md::new(Cow::Owned(pipeline.clone())),
            commands: Vec::new(),
            encoder: self,
            marker: std::marker::PhantomData,
        })
    }

    /// begin a compute pass owning the pipeline
    pub fn compute_pass_ref<'b>(
        &'b mut self,
        pipeline: &'a gpu::ComputePipeline,
    ) -> Result<crate::pass::BasicComputePass<'a, 'b>, gpu::Error> {
        Ok(crate::pass::BasicComputePass {
            pipeline: Md::new(Cow::Borrowed(pipeline)),
            commands: Vec::new(),
            encoder: self,
        })
    }

    /// begin a compute pass owning the pipeline
    pub fn compute_pass_owned<'b>(
        &'b mut self,
        pipeline: gpu::ComputePipeline,
    ) -> Result<crate::pass::BasicComputePass<'a, 'b>, gpu::Error> {
        Ok(crate::pass::BasicComputePass {
            pipeline: Md::new(Cow::Owned(pipeline)),
            commands: Vec::new(),
            encoder: self,
        })
    }

    /// Begin a reflected compute pass without borrowning the ReflectedCompute
    #[cfg(any(feature = "reflect", feature = "spirv"))]
    pub fn compute_pass_reflected<'b>(
        &'b mut self,
        device: &gpu::Device,
        compute: &crate::reflect::ReflectedCompute,
    ) -> Result<crate::pass::ReflectedComputePass<'a, 'b>, gpu::Error> {
        let pipeline_map = compute.pipeline_map.read().unwrap();

        let key = crate::reflect::compute::ComputePipelineKey {
            specialization: None,
        };

        if pipeline_map.get(&key).is_none() {
            drop(pipeline_map);

            let pipeline = device.create_compute_pipeline(&gpu::ComputePipelineDesc {
                name: compute.pipeline_data.name.clone(),
                layout: &compute.pipeline_data.layout,
                shader: (&compute.pipeline_data.shader, None),
                cache: Some(&compute.pipeline_data.cache),
            })?;

            compute.pipeline_map.write().unwrap().insert(key, pipeline);
        }

        let pipeline = compute.pipeline_map.read().unwrap().get(&key).unwrap().clone();

        Ok(crate::pass::ReflectedComputePass {
            parent_id: compute.id,
            bundle_needed: compute.bundle_needed(),
            push_constant_names: Cow::Owned(compute.reflect_data.push_constant_names.clone()),
            pipeline: Md::new(Cow::Owned(pipeline)),
            commands: Vec::new(),
            encoder: self,
        })
    }

    /// Write a timestamp to the query index when all previous commands in the pipeline stage have completed
    pub fn write_timestamp_ref(
        &mut self,
        query: &'a gpu::TimeQuery,
        index: u32,
        pipeline_stage: gpu::PipelineStage,
    ) {
        self.push_command(Command::WriteTimeStamp {
            query: Cow::Borrowed(query),
            index,
            pipeline_stage,
        })
    }

    /// Write a timestamp to the query index when all previous commands in the pipeline stage have completed
    pub fn write_timestamp_owned(
        &mut self,
        query: gpu::TimeQuery,
        index: u32,
        pipeline_stage: gpu::PipelineStage,
    ) {
        self.push_command(Command::WriteTimeStamp {
            query: Cow::Owned(query),
            index,
            pipeline_stage,
        })
    }

    /// Reset the queries from first_query to first_query + query_count (not inclusive)
    pub fn reset_time_query_ref(
        &mut self,
        query: &'a gpu::TimeQuery,
        first_query: u32,
        query_count: u32,
    ) {
        self.push_command(Command::ResetTimeQuery {
            query: Cow::Borrowed(query),
            first_query,
            query_count,
        })
    }

    /// Reset the queries from first_query to first_query + query_count (not inclusive)
    pub fn reset_time_query_owned(
        &mut self,
        query: gpu::TimeQuery,
        first_query: u32,
        query_count: u32,
    ) {
        self.push_command(Command::ResetTimeQuery {
            query: Cow::Owned(query),
            first_query,
            query_count,
        })
    }

    /// fill in any pipeline barriers to contain the correct src and dst flags
    /// TODO different layers of array textures are allowed to be in different formats
    /// at the moment this will not work as it doesn't know that so will report error saying that
    /// texture is trying to be in multiple layouts at the same time which is wrong
    pub fn format(&mut self) {
        self.formatted = true;

        let mut i = 0;
        let mut j = self.commands.len() - 1;

        let mut forward_buffer = HashMap::new();
        let mut forward_texture = HashMap::new();

        let mut back_buffer = HashMap::new();
        let mut back_texture = HashMap::new();

        let commands_len = self.commands.len();

        loop {
            let forward_command = self.commands.get_mut(i).unwrap();

            if let Command::PipelineBarrier {
                src_stage,
                buffers,
                textures,
                ..
            } = forward_command
            {
                for buffer in buffers {
                    if let Some((a, s)) = forward_buffer.get_mut(&buffer.buffer) {
                        *src_stage |= *s;
                        buffer.src_access = *a;
                        *a = gpu::AccessFlags::empty();
                        *s = gpu::PipelineStageFlags::empty();
                    }
                }

                for texture in textures {
                    for i in texture.base_mip_level..(texture.base_mip_level + texture.mip_levels) {
                        for j in texture.base_array_layer
                            ..(texture.base_array_layer + texture.array_layers)
                        {
                            let key = ((*texture.texture).clone(), i, j);
                            if let Some((a, s, l)) = forward_texture.get_mut(&key) {
                                *src_stage |= *s;
                                texture.src_access = *a;
                                texture.src_layout = *l;
                                *l = texture.dst_layout;
                                *a = gpu::AccessFlags::empty();
                                *s = gpu::PipelineStageFlags::empty();
                            }
                        }
                    }
                }
            } else {
                let stage = forward_command.stage();

                let buffer_access = forward_command.buffer_access();
                let texture_access = forward_command.texture_access();

                for buffer in forward_command.buffers() {
                    if let Some((a, s)) = forward_buffer.get_mut(&buffer) {
                        *a = buffer_access;
                        *s = stage;
                    } else {
                        forward_buffer.insert(buffer, (buffer_access, stage));
                    }
                }

                for (texture, layout) in forward_command.textures() {
                    if let Some((a, s, l)) = forward_texture.get_mut(&texture) {
                        *a = texture_access;
                        *s = stage;
                        *l = layout;
                    } else {
                        forward_texture.insert(texture, (texture_access, stage, layout));
                    }
                }

                for (texture, mip, array, layout) in forward_command.layout_changes() {
                    let (_, _, l) = forward_texture.get_mut(&(texture, mip, array)).unwrap();
                    *l = layout;
                }
            }

            let back_command = self.commands.get_mut(j).unwrap();

            if let Command::PipelineBarrier {
                dst_stage,
                buffers,
                textures,
                ..
            } = back_command
            {
                for buffer in buffers {
                    if let Some((a, s)) = back_buffer.get_mut(&buffer.buffer) {
                        *dst_stage |= *s;
                        buffer.dst_access = *a;
                        *a = gpu::AccessFlags::empty();
                        *s = gpu::PipelineStageFlags::empty();
                    }
                }

                for texture in textures {
                    for i in texture.base_mip_level..(texture.base_mip_level + texture.mip_levels) {
                        for j in texture.base_array_layer
                            ..(texture.base_array_layer + texture.array_layers)
                        {
                            let key = ((*texture.texture).clone(), i, j);
                            if let Some((a, s, l)) = back_texture.get_mut(&key) {
                                *dst_stage |= *s;
                                texture.dst_access = *a;
                                *l = texture.src_layout;
                                *a = gpu::AccessFlags::empty();
                                *s = gpu::PipelineStageFlags::empty();
                            }
                        }
                    }
                }
            } else {
                let stage = back_command.stage();

                let buffer_access = back_command.buffer_access();
                let texture_access = back_command.texture_access();

                for buffer in back_command.buffers() {
                    if let Some((a, s)) = back_buffer.get_mut(&buffer) {
                        *a = buffer_access;
                        *s = stage;
                    } else {
                        back_buffer.insert(buffer, (buffer_access, stage));
                    }
                }

                for (texture, layout) in back_command.textures() {
                    if let Some((a, s, l)) = back_texture.get_mut(&texture) {
                        *a = texture_access;
                        *s = stage;
                        *l = layout;
                    } else {
                        back_texture.insert(texture, (texture_access, stage, layout));
                    }
                }
            }

            i += 1;
            if i == commands_len {
                break;
            }
            j -= 1;
        }

        if !forward_texture.is_empty() {
            let src_stages = forward_texture
                .iter()
                .fold(gpu::PipelineStageFlags::empty(), |a, (_, (_, s, _))| a | *s);

            let textures = forward_texture
                .into_iter()
                .filter_map(|((t, mip, array), (access, _, layout))| {
                    if layout != t.initial_layout() {
                        Some(gpu::TextureAccessInfo {
                            src_layout: layout,
                            dst_layout: t.initial_layout(),
                            src_access: access,
                            dst_access: gpu::AccessFlags::empty(),
                            base_array_layer: array,
                            array_layers: 1,
                            base_mip_level: mip,
                            mip_levels: 1,
                            texture: Cow::Owned(t),
                        })
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            self.commands.push(Command::PipelineBarrier {
                src_stage: src_stages,
                dst_stage: gpu::PipelineStageFlags::BOTTOM_OF_PIPE,
                buffers: Vec::new(),
                textures,
            })
        }
    }
}
