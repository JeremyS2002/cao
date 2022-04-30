use std::borrow::Cow;
use std::collections::HashMap;
use std::collections::HashSet;

/// Represents a valid command or sequence of commands that can be submitted on a command recorder
#[derive(Debug)]
#[allow(missing_docs)]
pub enum Command<'a> {
    PipelineBarrier {
        src_stage: gpu::PipelineStageFlags,
        dst_stage: gpu::PipelineStageFlags,
        buffers: Vec<gpu::BufferAccessInfo<'a>>,
        textures: Vec<gpu::TextureAccessInfo<'a>>,
    },
    UpdateBuffer {
        buffer: Cow<'a, gpu::Buffer>,
        offset: u64,
        data: Cow<'a, [u8]>,
    },
    ClearTexture {
        texture: gpu::TextureSlice<'a>,
        layout: gpu::TextureLayout,
        value: gpu::ClearValue,
    },
    BlitTextures {
        src: gpu::TextureSlice<'a>,
        src_layout: gpu::TextureLayout,
        dst: gpu::TextureSlice<'a>,
        dst_layout: gpu::TextureLayout,
        filter: gpu::FilterMode,
    },
    ResolveTextures {
        src: gpu::TextureSlice<'a>,
        src_layout: gpu::TextureLayout,
        dst: gpu::TextureSlice<'a>,
        dst_layout: gpu::TextureLayout,
    },
    CopyBufferToBuffer {
        src: gpu::BufferSlice<'a>,
        dst: gpu::BufferSlice<'a>,
    },
    CopyTextureToBuffer {
        src: gpu::TextureSlice<'a>,
        src_layout: gpu::TextureLayout,
        dst: gpu::BufferSlice<'a>,
    },
    CopyBufferToTexture {
        src: gpu::BufferSlice<'a>,
        dst: gpu::TextureSlice<'a>,
        dst_layout: gpu::TextureLayout,
    },
    CopyTextureToTexture {
        src: gpu::TextureSlice<'a>,
        src_layout: gpu::TextureLayout,
        dst: gpu::TextureSlice<'a>,
        dst_layout: gpu::TextureLayout,
    },
    //ExecuteSecondary(&'a gpu::SecondaryCommandBuffer),
    GraphicsPass {
        color_attachments: Cow<'a, [gpu::Attachment<'a>]>,
        resolve_attachments: Cow<'a, [gpu::Attachment<'a>]>,
        depth_attachment: Option<gpu::Attachment<'a>>,
        pipeline: Cow<'a, gpu::GraphicsPipeline>,
        commands: Vec<crate::pass::GraphicsPassCommand<'a>>,
    },
    ComputePass {
        pipeline: Cow<'a, gpu::ComputePipeline>,
        commands: Vec<crate::pass::ComputePassCommand<'a>>,
    },
}

impl<'a> Command<'a> {
    pub(crate) unsafe fn execute(
        &self,
        command_buffer: &mut gpu::CommandBuffer,
    ) -> Result<(), gpu::Error> {
        match self {
            // Command::ExecuteSecondary(s) => command_buffer.execute_secondary(s)?,
            Command::ClearTexture {
                texture,
                layout,
                value,
            } => command_buffer.clear_texture(texture, *layout, *value)?,
            Command::UpdateBuffer {
                buffer,
                offset,
                data,
            } => command_buffer.update_buffer(buffer.as_ref(), *offset, data)?,
            Command::ResolveTextures {
                src,
                src_layout,
                dst,
                dst_layout,
            } => command_buffer.resolve_texture(src, *src_layout, dst, *dst_layout)?,
            Command::BlitTextures {
                src,
                src_layout,
                dst,
                dst_layout,
                filter,
            } => command_buffer.blit_textures(src, *src_layout, dst, *dst_layout, *filter)?,
            Command::PipelineBarrier {
                buffers,
                textures,
                src_stage,
                dst_stage,
            } => command_buffer.pipeline_barrier(*src_stage, *dst_stage, buffers, textures)?,
            Command::CopyBufferToBuffer { src, dst } => {
                command_buffer.copy_buffer_to_buffer(src, dst)?
            }
            Command::CopyBufferToTexture {
                src,
                dst,
                dst_layout,
            } => command_buffer.copy_buffer_to_texture(src, dst, *dst_layout)?,
            Command::CopyTextureToBuffer {
                src,
                src_layout,
                dst,
            } => command_buffer.copy_texture_to_buffer(src, *src_layout, dst)?,
            Command::CopyTextureToTexture {
                src,
                src_layout,
                dst,
                dst_layout,
            } => command_buffer.copy_texture_to_texture(src, *src_layout, dst, *dst_layout)?,
            Command::ComputePass { commands, pipeline } => {
                command_buffer.begin_compute_pass(pipeline)?;
                for command in commands {
                    command.execute(command_buffer, pipeline.layout())?;
                }
            }
            Command::GraphicsPass {
                color_attachments,
                resolve_attachments,
                depth_attachment,
                commands,
                pipeline,
            } => {
                command_buffer.begin_graphics_pass(
                    color_attachments,
                    resolve_attachments,
                    *depth_attachment,
                    pipeline,
                )?;
                for command in commands {
                    command.execute(command_buffer, &pipeline.layout())?;
                }
                command_buffer.end_graphics_pass()?;
            }
        }
        Ok(())
    }

    /// produces textures and what layout they are in after the call
    pub fn layout_changes(&self) -> Vec<(Cow<'a, gpu::Texture>, u32, u32, gpu::TextureLayout)> {
        let mut result = Vec::new();
        match self {
            Command::GraphicsPass {
                color_attachments,
                resolve_attachments,
                depth_attachment,
                pipeline,
                ..
            } => {
                for (i, a) in color_attachments.as_ref().iter().enumerate() {
                    let view = a.view();
                    let c = pipeline.pass().colors()[i];

                    for i in view.base_mip_level()..(view.base_mip_level() + view.mip_levels()) {
                        for j in
                            view.base_array_layer()..(view.base_array_layer() + view.array_layers())
                        {
                            result.push((Cow::Borrowed(view.texture()), i, j, c.final_layout));
                        }
                    }
                }

                for (i, a) in resolve_attachments.as_ref().iter().enumerate() {
                    let r = pipeline.pass().resolves()[i];

                    let view = a.view();
                    for i in view.base_mip_level()..(view.base_mip_level() + view.mip_levels()) {
                        for j in
                            view.base_array_layer()..(view.base_array_layer() + view.array_layers())
                        {
                            result.push((Cow::Borrowed(view.texture()), i, j, r.final_layout));
                        }
                    }
                }

                if let Some(a) = depth_attachment {
                    let view = a.view();

                    for i in view.base_mip_level()..(view.base_mip_level() + view.mip_levels()) {
                        for j in
                            view.base_array_layer()..(view.base_array_layer() + view.array_layers())
                        {
                            result.push((
                                Cow::Borrowed(view.texture()),
                                i,
                                j,
                                pipeline.pass().depth().unwrap().final_layout,
                            ));
                        }
                    }
                }
            }
            _ => (),
        }

        result
    }

    /// Get all the textures referenced by the command represented by self and the layout they should be in
    pub fn textures(&self) -> HashMap<(Cow<'a, gpu::Texture>, u32, u32), gpu::TextureLayout> {
        let mut result = HashMap::new();
        match self {
            Command::ClearTexture {
                texture, layout, ..
            } => {
                for i in texture.base_mip_level()..(texture.base_mip_level() + texture.mip_levels())
                {
                    for j in texture.base_array_layer()
                        ..(texture.base_array_layer() + texture.array_layers())
                    {
                        match texture.cow_texture() {
                            Cow::Borrowed(t) => {
                                result.insert((Cow::Borrowed(*t), i, j), *layout);
                            }
                            Cow::Owned(t) => {
                                result.insert((Cow::Owned(t.clone()), i, j), *layout);
                            }
                        }
                    }
                }
            }
            Command::ResolveTextures {
                src,
                src_layout,
                dst,
                dst_layout,
            } => {
                for i in src.base_mip_level()..(src.base_mip_level() + src.mip_levels()) {
                    for j in src.base_array_layer()..(src.base_array_layer() + src.array_layers()) {
                        match src.cow_texture() {
                            Cow::Borrowed(t) => {
                                result.insert((Cow::Borrowed(*t), i, j), *src_layout);
                            }
                            Cow::Owned(t) => {
                                result.insert((Cow::Owned(t.clone()), i, j), *src_layout);
                            }
                        }
                    }
                }
                for i in dst.base_mip_level()..(dst.base_mip_level() + dst.mip_levels()) {
                    for j in dst.base_array_layer()..(dst.base_array_layer() + dst.array_layers()) {
                        match dst.cow_texture() {
                            Cow::Borrowed(t) => {
                                result.insert((Cow::Borrowed(*t), i, j), *dst_layout);
                            }
                            Cow::Owned(t) => {
                                result.insert((Cow::Owned(t.clone()), i, j), *dst_layout);
                            }
                        }
                    }
                }
            }
            Command::BlitTextures {
                src,
                src_layout,
                dst,
                dst_layout,
                ..
            } => {
                // only need to track base mip level as only base will be used for this command
                for i in src.base_array_layer()..(src.base_array_layer() + src.array_layers()) {
                    match src.cow_texture() {
                        Cow::Borrowed(t) => {
                            result
                                .insert((Cow::Borrowed(*t), src.base_mip_level(), i), *src_layout);
                        }
                        Cow::Owned(t) => {
                            result.insert(
                                (Cow::Owned(t.clone()), src.base_mip_level(), i),
                                *src_layout,
                            );
                        }
                    }
                }
                for i in dst.base_array_layer()..(dst.base_array_layer() + dst.array_layers()) {
                    match dst.cow_texture() {
                        Cow::Borrowed(t) => {
                            result
                                .insert((Cow::Borrowed(*t), dst.base_mip_level(), i), *dst_layout);
                        }
                        Cow::Owned(t) => {
                            result.insert(
                                (Cow::Owned(t.clone()), dst.base_mip_level(), i),
                                *dst_layout,
                            );
                        }
                    }
                }
            }
            Command::CopyTextureToTexture {
                src,
                src_layout,
                dst,
                dst_layout,
            } => {
                // only need to track base mip level as only base will be used for this command
                for i in src.base_array_layer()..(src.base_array_layer() + src.array_layers()) {
                    match src.cow_texture() {
                        Cow::Borrowed(t) => {
                            result
                                .insert((Cow::Borrowed(*t), src.base_mip_level(), i), *src_layout);
                        }
                        Cow::Owned(t) => {
                            result.insert(
                                (Cow::Owned(t.clone()), src.base_mip_level(), i),
                                *src_layout,
                            );
                        }
                    }
                }
                for i in dst.base_array_layer()..(dst.base_array_layer() + dst.array_layers()) {
                    match dst.cow_texture() {
                        Cow::Borrowed(t) => {
                            result
                                .insert((Cow::Borrowed(*t), dst.base_mip_level(), i), *dst_layout);
                        }
                        Cow::Owned(t) => {
                            result.insert(
                                (Cow::Owned(t.clone()), dst.base_mip_level(), i),
                                *dst_layout,
                            );
                        }
                    }
                }
            }
            Command::CopyBufferToTexture {
                dst, dst_layout, ..
            } => {
                // only need to track base mip level as only base will be used for this command
                for i in dst.base_array_layer()..(dst.base_array_layer() + dst.array_layers()) {
                    match dst.cow_texture() {
                        Cow::Borrowed(t) => {
                            result
                                .insert((Cow::Borrowed(*t), dst.base_mip_level(), i), *dst_layout);
                        }
                        Cow::Owned(t) => {
                            result.insert(
                                (Cow::Owned(t.clone()), dst.base_mip_level(), i),
                                *dst_layout,
                            );
                        }
                    }
                }
            }
            Command::CopyTextureToBuffer {
                src, src_layout, ..
            } => {
                // only need to track base mip level as only base will be used for this command
                for i in src.base_array_layer()..(src.base_array_layer() + src.array_layers()) {
                    match src.cow_texture() {
                        Cow::Borrowed(t) => {
                            result
                                .insert((Cow::Borrowed(*t), src.base_mip_level(), i), *src_layout);
                        }
                        Cow::Owned(t) => {
                            result.insert(
                                (Cow::Owned(t.clone()), src.base_mip_level(), i),
                                *src_layout,
                            );
                        }
                    }
                }
            }
            Command::ComputePass { commands, .. } => {
                for command in commands {
                    for (texture, layout) in command.textures() {
                        if let Some(l) = result.insert(texture, layout) {
                            if layout != l {
                                panic!("ERROR: ComputePass uses texture with different layouts {:?} and {:?}", layout, l);
                            }
                        }
                    }
                }
            }
            Command::GraphicsPass {
                color_attachments,
                resolve_attachments,
                depth_attachment,
                commands,
                pipeline,
                ..
            } => {
                for (index, a) in color_attachments.as_ref().iter().enumerate() {
                    let view = a.view();
                    let c = pipeline.pass().colors()[index];
                    for i in view.base_mip_level()..(view.base_mip_level() + view.mip_levels()) {
                        for j in
                            view.base_array_layer()..(view.base_array_layer() + view.array_layers())
                        {
                            if let Some(_) = result
                                .insert((Cow::Borrowed(view.texture()), i, j), c.initial_layout)
                            {
                                panic!(
                                    "ERROR: GraphicsPass uses texture {:?} as multiple attachments",
                                    view.texture()
                                );
                            }
                        }
                    }
                }
                for (index, a) in resolve_attachments.as_ref().iter().enumerate() {
                    let view = a.view();
                    let c = pipeline.pass().resolves()[index];
                    for i in view.base_mip_level()..(view.base_mip_level() + view.mip_levels()) {
                        for j in
                            view.base_array_layer()..(view.base_array_layer() + view.array_layers())
                        {
                            if let Some(_) = result
                                .insert((Cow::Borrowed(view.texture()), i, j), c.initial_layout)
                            {
                                panic!(
                                    "ERROR: GraphicsPass uses texture {:?} as multiple attachments",
                                    view.texture()
                                );
                            }
                        }
                    }
                }
                if let Some(a) = depth_attachment {
                    let view = a.view();
                    let d = pipeline.pass().depth()
                        .expect("Attempt to use render pass with no depth component description with depth attachment");
                    for i in view.base_mip_level()..(view.base_mip_level() + view.mip_levels()) {
                        for j in
                            view.base_array_layer()..(view.base_array_layer() + view.array_layers())
                        {
                            if let Some(_) = result
                                .insert((Cow::Borrowed(view.texture()), i, j), d.initial_layout)
                            {
                                panic!(
                                    "ERROR: GraphicsPass uses texture {:?} as multiple attachments",
                                    view.texture()
                                );
                            }
                        }
                    }
                }
                let mut command_map = HashMap::new();
                for command in commands {
                    for (texture, layout) in command.textures() {
                        if let Some(l) = command_map.insert(texture, layout) {
                            if layout != l {
                                panic!("ERROR: GraphicsPass uses texture with different layouts {:?} and {:?}", layout, l);
                            }
                        }
                    }
                }
                for (texture, layout) in command_map {
                    if let Some(l) = result.insert(texture, layout) {
                        panic!(
                            "ERROR: GraphicsPass uses texture with different layouts {:?} and {:?}",
                            layout, l
                        );
                    }
                }
            }
            _ => (),
        }
        result
    }

    /// Get all the buffers referenced by the command represented by self
    pub fn buffers(&self) -> HashSet<gpu::BufferSlice<'a>> {
        let mut result = HashSet::new();
        match self {
            Command::UpdateBuffer {
                buffer,
                offset,
                data,
            } => match buffer {
                Cow::Borrowed(b) => {
                    result.insert(b.slice_ref((*offset)..(data.len() as _)));
                }
                Cow::Owned(b) => {
                    result.insert(b.slice_owned((*offset)..(data.len() as _)));
                }
            },
            Command::CopyBufferToBuffer { src, dst } => {
                result.insert(src.clone());
                result.insert(dst.clone());
            }
            Command::CopyBufferToTexture { src, .. } => {
                result.insert(src.clone());
            }
            Command::CopyTextureToBuffer { dst, .. } => {
                result.insert(dst.clone());
            }
            Command::ComputePass { commands, .. } => {
                for command in commands {
                    for buffer in command.buffers() {
                        result.insert(buffer);
                    }
                }
            }
            Command::GraphicsPass { commands, .. } => {
                for command in commands {
                    for buffer in command.buffers() {
                        result.insert(buffer);
                    }
                }
            }
            _ => (),
        }
        result
    }

    /// Get all the samplers referenced by the command represented by self
    pub fn samplers<'b>(&'b self) -> Vec<&'b gpu::Sampler> {
        let mut samplers = Vec::new();
        match self {
            Self::GraphicsPass { commands, .. } => {
                for command in commands {
                    match command {
                        crate::pass::GraphicsPassCommand::BindDescriptorSets {
                            descriptors,
                            ..
                        } => {
                            for descriptor in descriptors.as_ref() {
                                samplers.extend(descriptor.samplers())
                            }
                        }
                        crate::pass::GraphicsPassCommand::BindDescriptorSet {
                            descriptor, ..
                        } => samplers.extend(descriptor.samplers()),
                        _ => (),
                    }
                }
            }
            Self::ComputePass { commands, .. } => {
                for command in commands {
                    match command {
                        crate::pass::ComputePassCommand::BindDescriptorSets {
                            descriptors, ..
                        } => {
                            for descriptor in descriptors.as_ref() {
                                samplers.extend(descriptor.samplers())
                            }
                        }
                        crate::pass::ComputePassCommand::BindDescriptorSet {
                            descriptor, ..
                        } => samplers.extend(descriptor.samplers()),
                        _ => (),
                    }
                }
            }
            _ => (),
        }
        samplers
    }

    /// returns access flags for this command
    pub(crate) fn texture_access(&self) -> gpu::AccessFlags {
        match self {
            // Command::ExecuteSecondary(_) => gpu::AccessFlags::empty(),
            Command::ClearTexture { .. } => gpu::AccessFlags::COPY_WRITE,
            Command::UpdateBuffer { .. } => gpu::AccessFlags::COPY_WRITE,
            Command::BlitTextures { .. } => {
                gpu::AccessFlags::COPY_READ | gpu::AccessFlags::COPY_WRITE
            }
            Command::CopyBufferToBuffer { .. } => {
                gpu::AccessFlags::COPY_READ | gpu::AccessFlags::COPY_WRITE
            }
            Command::CopyBufferToTexture { .. } => {
                gpu::AccessFlags::COPY_READ | gpu::AccessFlags::COPY_WRITE
            }
            Command::CopyTextureToBuffer { .. } => {
                gpu::AccessFlags::COPY_READ | gpu::AccessFlags::COPY_WRITE
            }
            Command::CopyTextureToTexture { .. } => {
                gpu::AccessFlags::COPY_READ | gpu::AccessFlags::COPY_WRITE
            }
            Command::ResolveTextures { .. } => {
                gpu::AccessFlags::COPY_READ | gpu::AccessFlags::COPY_WRITE
            }
            Command::GraphicsPass { .. } => gpu::AccessFlags::MEMORY_READ,
            Command::PipelineBarrier { .. } => gpu::AccessFlags::empty(),
            Command::ComputePass { .. } => gpu::AccessFlags::empty(),
        }
    }

    pub(crate) fn buffer_access(&self) -> gpu::AccessFlags {
        match self {
            // Command::ExecuteSecondary(_) => gpu::AccessFlags::empty(),
            Command::ClearTexture { .. } => gpu::AccessFlags::empty(),
            Command::BlitTextures { .. } => gpu::AccessFlags::empty(),
            Command::UpdateBuffer { .. } => gpu::AccessFlags::COPY_WRITE,
            Command::CopyBufferToBuffer { .. } => {
                gpu::AccessFlags::COPY_READ | gpu::AccessFlags::COPY_WRITE
            }
            Command::CopyBufferToTexture { .. } => {
                gpu::AccessFlags::COPY_READ | gpu::AccessFlags::COPY_WRITE
            }
            Command::CopyTextureToBuffer { .. } => {
                gpu::AccessFlags::COPY_READ | gpu::AccessFlags::COPY_WRITE
            }
            Command::CopyTextureToTexture { .. } => {
                gpu::AccessFlags::COPY_READ | gpu::AccessFlags::COPY_WRITE
            }
            Command::ResolveTextures { .. } => gpu::AccessFlags::empty(),
            Command::GraphicsPass { .. } => gpu::AccessFlags::MEMORY_READ,
            Command::PipelineBarrier { .. } => gpu::AccessFlags::empty(),
            Command::ComputePass { .. } => gpu::AccessFlags::empty(),
        }
    }

    /// returns the stage that the command is in
    pub(crate) fn stage(&self) -> gpu::PipelineStageFlags {
        match self {
            // Command::ExecuteSecondary(_) => {
            //     gpu::PipelineStageFlags::TOP_OF_PIPE | gpu::PipelineStageFlags::BOTTOM_OF_PIPE
            // }
            Command::UpdateBuffer { .. } => gpu::PipelineStageFlags::COPY,
            Command::ClearTexture { .. } => gpu::PipelineStageFlags::COPY,
            Command::BlitTextures { .. } => gpu::PipelineStageFlags::COPY,
            Command::CopyBufferToBuffer { .. } => gpu::PipelineStageFlags::COPY,
            Command::CopyTextureToBuffer { .. } => gpu::PipelineStageFlags::COPY,
            Command::CopyBufferToTexture { .. } => gpu::PipelineStageFlags::COPY,
            Command::CopyTextureToTexture { .. } => gpu::PipelineStageFlags::COPY,
            Command::ResolveTextures { .. } => gpu::PipelineStageFlags::COPY,
            Command::GraphicsPass { .. } => {
                gpu::PipelineStageFlags::FRAGMENT
                    | gpu::PipelineStageFlags::DEPTH_STENCIL_EARLY
                    | gpu::PipelineStageFlags::DEPTH_STENCIL_LATE
            }
            Command::PipelineBarrier { .. } => gpu::PipelineStageFlags::empty(),
            Command::ComputePass { .. } => gpu::PipelineStageFlags::COMPUTE,
        }
    }
}
