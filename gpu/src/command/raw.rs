use std::borrow::Borrow;
use std::ptr;
use std::sync::Arc;

use ash::vk;

use parking_lot::Mutex;

pub(crate) fn pipeline_barrier(
    command_buffer: vk::CommandBuffer,
    device: &crate::RawDevice,
    src_stages: crate::PipelineStageFlags,
    dst_stages: crate::PipelineStageFlags,
    buffers: &[crate::BufferAccessInfo<'_>],
    textures: &[crate::TextureAccessInfo<'_>],
) -> Result<(), crate::Error> {
    #[cfg(feature = "logging")]
    log::trace!("GPU: cmd_pipeline_barrier");
    let image_barriers = textures
        .iter()
        .map(|info| vk::ImageMemoryBarrier {
            s_type: vk::StructureType::IMAGE_MEMORY_BARRIER,
            p_next: ptr::null(),
            src_access_mask: info.src_access.into(),
            dst_access_mask: info.dst_access.into(),
            old_layout: info.src_layout.into(),
            new_layout: info.dst_layout.into(),
            image: **info.texture.raw,
            src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: info.texture.format.aspects().into(),
                base_mip_level: info.base_mip_level,
                level_count: info.mip_levels,
                base_array_layer: info.base_array_layer,
                layer_count: info.array_layers,
            },
        })
        .collect::<Vec<_>>();

    let buffer_barriers = buffers
        .iter()
        .map(|info| vk::BufferMemoryBarrier {
            s_type: vk::StructureType::BUFFER_MEMORY_BARRIER,
            p_next: ptr::null(),
            src_access_mask: info.src_access.into(),
            dst_access_mask: info.dst_access.into(),
            src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            buffer: **info.buffer.buffer.raw,
            offset: info.buffer.offset,
            size: info.buffer.size,
        })
        .collect::<Vec<_>>();
    unsafe {
        device.cmd_pipeline_barrier(
            command_buffer,
            src_stages.into(),
            dst_stages.into(),
            vk::DependencyFlags::empty(),
            &[],
            &buffer_barriers,
            &image_barriers,
        )
    }

    Ok(device.check_errors()?)
}

pub(crate) fn update_buffer<B>(
    command_buffer: vk::CommandBuffer,
    device: &crate::RawDevice,
    buffer: B,
    offset: u64,
    data: &[u8],
    garbage: &mut super::Garbage,
) -> Result<(), crate::Error>
where
    B: Borrow<crate::Buffer>,
{
    garbage.buffers.push(Arc::clone(&*(buffer.borrow().raw)));
    garbage.memory.push(Arc::clone(&*buffer.borrow().memory));

    #[cfg(feature = "logging")]
    log::trace!("GPU: cmd_update_buffer");
    unsafe {
        device.cmd_update_buffer(command_buffer, **buffer.borrow().raw, offset, data);
    }
    Ok(device.check_errors()?)
}

pub(crate) fn clear_texture<'a, T1>(
    command_buffer: vk::CommandBuffer,
    device: &crate::RawDevice,
    texture: T1,
    layout: crate::TextureLayout,
    value: crate::ClearValue,
    garbage: &mut super::Garbage,
) -> Result<(), crate::Error>
where
    T1: Borrow<crate::TextureSlice<'a>>,
{
    if let Some(mem) = &texture.borrow().texture.memory {
        garbage
            .textures
            .push(Arc::clone(&*(texture.borrow().texture.raw)));
        garbage.memory.push(Arc::clone(mem));
    }

    #[cfg(feature = "logging")]
    log::trace!("GPU: cmd_clear_color_texture");
    if value.color() {
        unsafe {
            device.cmd_clear_color_image(
                command_buffer,
                **texture.borrow().texture.raw,
                layout.into(),
                &value.into(),
                &[vk::ImageSubresourceRange {
                    aspect_mask: texture.borrow().texture.format().aspects().into(),
                    base_mip_level: texture.borrow().base_mip_level,
                    level_count: texture.borrow().mip_levels,
                    base_array_layer: texture.borrow().base_array_layer,
                    layer_count: texture.borrow().array_layers,
                }],
            );
        }
    } else {
        unsafe {
            device.cmd_clear_depth_stencil_image(
                command_buffer,
                **texture.borrow().texture.raw,
                layout.into(),
                &value.into(),
                &[vk::ImageSubresourceRange {
                    aspect_mask: texture.borrow().texture.format().aspects().into(),
                    base_mip_level: texture.borrow().base_mip_level,
                    level_count: texture.borrow().mip_levels,
                    base_array_layer: texture.borrow().base_array_layer,
                    layer_count: texture.borrow().array_layers,
                }],
            );
        }
    }

    Ok(device.check_errors()?)
}

pub(crate) fn blit_textures<'a, T1, T2>(
    command_buffer: vk::CommandBuffer,
    device: &crate::RawDevice,
    src: T1,
    src_layout: crate::TextureLayout,
    dst: T2,
    dst_layout: crate::TextureLayout,
    filter: crate::FilterMode,
    garbage: &mut super::Garbage,
) -> Result<(), crate::Error>
where
    T1: Borrow<crate::TextureSlice<'a>>,
    T2: Borrow<crate::TextureSlice<'a>>,
{
    if let Some(mem) = &src.borrow().texture.memory {
        garbage
            .textures
            .push(Arc::clone(&*(src.borrow().texture.raw)));
        garbage.memory.push(Arc::clone(mem));
    }
    if let Some(mem) = &dst.borrow().texture.memory {
        garbage
            .textures
            .push(Arc::clone(&*(dst.borrow().texture.raw)));
        garbage.memory.push(Arc::clone(mem));
    }

    #[cfg(feature = "logging")]
    log::trace!("GPU: cmd_blit_textures src: {:?} in layout {:?}, dst: {:?} in layout {:?}, filter mode: {:?}", src.borrow(), dst.borrow(), src_layout, dst_layout, filter);
    #[cfg(feature = "logging")]
    if src.borrow().mip_levels != 1 {
        log::warn!(
            "blit textures with src \"{:?}\" with multiple mip levels: only the base mip level is used", src.borrow()
        )
    }
    #[cfg(feature = "logging")]
    if dst.borrow().mip_levels != 1 {
        log::warn!(
            "blit textures with dst \"{:?}\" with multiple mip levels: only the base mip level is used", dst.borrow()
        );
    }
    unsafe {
        device.cmd_blit_image(
            command_buffer,
            **src.borrow().texture.raw,
            src_layout.into(),
            **dst.borrow().texture.raw,
            dst_layout.into(),
            &[vk::ImageBlit {
                src_subresource: vk::ImageSubresourceLayers {
                    aspect_mask: src.borrow().texture.format.aspects().into(),
                    mip_level: src.borrow().base_mip_level,
                    base_array_layer: src.borrow().base_array_layer,
                    layer_count: src.borrow().array_layers,
                },
                src_offsets: [
                    src.borrow().offset.into(),
                    (src.borrow().offset + src.borrow().extent).into(),
                ],
                dst_subresource: vk::ImageSubresourceLayers {
                    aspect_mask: dst.borrow().texture.format.aspects().into(),
                    mip_level: dst.borrow().base_mip_level,
                    base_array_layer: dst.borrow().base_array_layer,
                    layer_count: dst.borrow().array_layers,
                },
                dst_offsets: [
                    dst.borrow().offset.into(),
                    (dst.borrow().offset + dst.borrow().extent).into(),
                ],
            }],
            filter.into(),
        )
    }
    Ok(device.check_errors()?)
}

pub(crate) fn copy_buffer_to_buffer<'a, B1, B2>(
    command_buffer: vk::CommandBuffer,
    device: &crate::RawDevice,
    src: B1,
    dst: B2,
    garbage: &mut super::Garbage,
) -> Result<(), crate::Error>
where
    B1: Borrow<crate::BufferSlice<'a>>,
    B2: Borrow<crate::BufferSlice<'a>>,
{
    garbage
        .buffers
        .push(Arc::clone(&*(src.borrow().buffer.raw)));
    garbage
        .memory
        .push(Arc::clone(&*src.borrow().buffer.memory));
    garbage
        .buffers
        .push(Arc::clone(&*(dst.borrow().buffer.raw)));
    garbage
        .memory
        .push(Arc::clone(&*dst.borrow().buffer.memory));

    #[cfg(feature = "logging")]
    log::trace!(
        "GPU: cmd_copy_buffer_to_buffer src: {:?}, dst: {:?}",
        src.borrow(),
        dst.borrow()
    );
    unsafe {
        device.cmd_copy_buffer(
            command_buffer,
            **src.borrow().buffer.raw,
            **dst.borrow().buffer.raw,
            &[vk::BufferCopy {
                src_offset: src.borrow().offset,
                dst_offset: dst.borrow().offset,
                size: src.borrow().size.min(dst.borrow().size),
            }],
        )
    }
    Ok(device.check_errors()?)
}

pub(crate) fn copy_texture_to_buffer<'a, B, T>(
    command_buffer: vk::CommandBuffer,
    device: &crate::RawDevice,
    src: T,
    src_layout: crate::TextureLayout,
    dst: B,
    garbage: &mut super::Garbage,
) -> Result<(), crate::Error>
where
    B: Borrow<crate::BufferSlice<'a>>,
    T: Borrow<crate::TextureSlice<'a>>,
{
    if let Some(mem) = &src.borrow().texture.memory {
        garbage
            .textures
            .push(Arc::clone(&*src.borrow().texture.raw));
        garbage.memory.push(Arc::clone(mem));
    }
    garbage.buffers.push(Arc::clone(&*dst.borrow().buffer.raw));
    garbage
        .memory
        .push(Arc::clone(&*dst.borrow().buffer.memory));

    #[cfg(feature = "logging")]
    log::trace!(
        "GPU: cmd_copy_texture_to_buffer src: {:?} in layout {:?} dst: {:?}",
        src.borrow(),
        src_layout,
        dst.borrow()
    );
    #[cfg(feature = "logging")]
    if src.borrow().mip_levels != 1 {
        log::warn!("GPU: copy texture to buffer with src: \"{:?}\" of multiple mip levels: only the base mip level is used", src.borrow())
    }
    unsafe {
        device.cmd_copy_image_to_buffer(
            command_buffer,
            **src.borrow().texture.raw,
            src_layout.into(),
            **dst.borrow().buffer.raw,
            &[vk::BufferImageCopy {
                buffer_offset: dst.borrow().offset,
                buffer_row_length: src.borrow().extent.width,
                buffer_image_height: src.borrow().extent.height,
                image_subresource: vk::ImageSubresourceLayers {
                    aspect_mask: src.borrow().texture.format.aspects().into(),
                    base_array_layer: src.borrow().base_array_layer,
                    layer_count: src.borrow().array_layers,
                    mip_level: src.borrow().base_mip_level,
                },
                image_offset: src.borrow().offset.into(),
                image_extent: src.borrow().extent.into(),
            }],
        )
    }
    Ok(device.check_errors()?)
}

pub(crate) fn copy_buffer_to_texture<'a, B, T>(
    command_buffer: vk::CommandBuffer,
    device: &crate::RawDevice,
    src: B,
    dst: T,
    dst_layout: crate::TextureLayout,
    garbage: &mut super::Garbage,
) -> Result<(), crate::Error>
where
    B: Borrow<crate::BufferSlice<'a>>,
    T: Borrow<crate::TextureSlice<'a>>,
{
    garbage.buffers.push(Arc::clone(&*src.borrow().buffer.raw));
    garbage
        .memory
        .push(Arc::clone(&*src.borrow().buffer.memory));
    if let Some(mem) = &dst.borrow().texture.memory {
        garbage
            .textures
            .push(Arc::clone(&*dst.borrow().texture.raw));
        garbage.memory.push(Arc::clone(mem));
    }

    #[cfg(feature = "logging")]
    log::trace!(
        "GPU: cmd_copy_buffer_to_texture src: {:?}, dst: {:?} in layout {:?}",
        src.borrow(),
        dst.borrow(),
        dst_layout
    );
    #[cfg(feature = "logging")]
    if dst.borrow().mip_levels != 1 {
        log::warn!("GPU: copy buffer to texture with dst: \"{:?}\" of multiple mip levels: only the base mip level is used", dst.borrow())
    }
    unsafe {
        device.cmd_copy_buffer_to_image(
            command_buffer,
            **src.borrow().buffer.raw,
            **dst.borrow().texture.raw,
            dst_layout.into(),
            &[vk::BufferImageCopy {
                buffer_offset: src.borrow().offset,
                buffer_row_length: dst.borrow().extent.width,
                buffer_image_height: dst.borrow().extent.height,
                image_subresource: vk::ImageSubresourceLayers {
                    aspect_mask: dst.borrow().texture.format.aspects().into(),
                    base_array_layer: dst.borrow().base_array_layer,
                    layer_count: dst.borrow().array_layers,
                    mip_level: dst.borrow().base_mip_level,
                },
                image_offset: dst.borrow().offset.into(),
                image_extent: dst.borrow().extent.into(),
            }],
        )
    }
    Ok(device.check_errors()?)
}

pub(crate) fn copy_texture_to_texture<'a, T1, T2>(
    command_buffer: vk::CommandBuffer,
    device: &crate::RawDevice,
    src: T1,
    src_layout: crate::TextureLayout,
    dst: T2,
    dst_layout: crate::TextureLayout,
    garbage: &mut super::Garbage,
) -> Result<(), crate::Error>
where
    T1: Borrow<crate::TextureSlice<'a>>,
    T2: Borrow<crate::TextureSlice<'a>>,
{
    if let Some(mem) = &src.borrow().texture.memory {
        garbage
            .textures
            .push(Arc::clone(&*src.borrow().texture.raw));
        garbage.memory.push(Arc::clone(mem));
    }
    if let Some(mem) = &dst.borrow().texture.memory {
        garbage
            .textures
            .push(Arc::clone(&*dst.borrow().texture.raw));
        garbage.memory.push(Arc::clone(mem));
    }

    #[cfg(feature = "logging")]
    log::trace!(
        "GPU: cmd_copy_texture_to_texture src: {:?} in layout {:?}, dst: {:?} in layout {:?}",
        src.borrow(),
        src_layout,
        dst.borrow(),
        dst_layout
    );
    #[cfg(feature = "logging")]
    if src.borrow().mip_levels != 1 {
        log::warn!("GPU: copy texture to texture with src \"{:?}\" of multiple mip levels: only the base mip level is used", src.borrow())
    }
    #[cfg(feature = "logging")]
    if dst.borrow().mip_levels != 1 {
        log::warn!("GPU: copy texture to texture with dst \"{:?}\" of multiple mip levels: only the base mip level is used", dst.borrow())
    }
    unsafe {
        device.cmd_copy_image(
            command_buffer,
            **src.borrow().texture.raw,
            src_layout.into(),
            **dst.borrow().texture.raw,
            dst_layout.into(),
            &[vk::ImageCopy {
                src_subresource: vk::ImageSubresourceLayers {
                    aspect_mask: src.borrow().texture.format.aspects().into(),
                    base_array_layer: src.borrow().base_array_layer,
                    layer_count: src.borrow().array_layers,
                    mip_level: src.borrow().base_mip_level,
                },
                src_offset: src.borrow().offset.into(),
                dst_subresource: vk::ImageSubresourceLayers {
                    aspect_mask: dst.borrow().texture.format.aspects().into(),
                    base_array_layer: dst.borrow().base_array_layer,
                    layer_count: dst.borrow().array_layers,
                    mip_level: dst.borrow().base_mip_level,
                },
                dst_offset: dst.borrow().offset.into(),
                extent: src.borrow().extent.into(),
            }],
        )
    }
    Ok(device.check_errors()?)
}

pub(crate) fn resolve_texture<'a, T1, T2>(
    command_buffer: vk::CommandBuffer,
    device: &crate::RawDevice,
    src: T1,
    src_layout: crate::TextureLayout,
    dst: T2,
    dst_layout: crate::TextureLayout,
    garbage: &mut super::Garbage,
) -> Result<(), crate::Error>
where
    T1: Borrow<crate::TextureSlice<'a>>,
    T2: Borrow<crate::TextureSlice<'a>>,
{
    if let Some(mem) = &src.borrow().texture.memory {
        garbage
            .textures
            .push(Arc::clone(&*src.borrow().texture.raw));
        garbage.memory.push(Arc::clone(mem));
    }
    if let Some(mem) = &dst.borrow().texture.memory {
        garbage
            .textures
            .push(Arc::clone(&*dst.borrow().texture.raw));
        garbage.memory.push(Arc::clone(mem));
    }

    #[cfg(feature = "logging")]
    log::trace!(
        "GPU: cmd_resolve_texture src: {:?} in layout {:?} dst: {:?} in layout {:?}",
        src.borrow(),
        src_layout,
        dst.borrow(),
        dst_layout
    );
    unsafe {
        device.cmd_resolve_image(
            command_buffer,
            **src.borrow().texture.raw,
            src_layout.into(),
            **dst.borrow().texture.raw,
            dst_layout.into(),
            &[vk::ImageResolve {
                src_subresource: vk::ImageSubresourceLayers {
                    aspect_mask: src.borrow().texture.format.aspects().into(),
                    base_array_layer: src.borrow().base_array_layer,
                    layer_count: src.borrow().array_layers,
                    mip_level: src.borrow().base_mip_level,
                },
                src_offset: src.borrow().offset.into(),
                dst_subresource: vk::ImageSubresourceLayers {
                    aspect_mask: dst.borrow().texture.format.aspects().into(),
                    base_array_layer: dst.borrow().base_array_layer,
                    layer_count: dst.borrow().array_layers,
                    mip_level: dst.borrow().base_mip_level,
                },
                dst_offset: dst.borrow().offset.into(),
                extent: src.borrow().extent.into(),
            }],
        )
    }
    Ok(device.check_errors()?)
}

pub(crate) fn begin_primary(
    command_buffer: vk::CommandBuffer,
    device: &crate::RawDevice,
    one_time_submit: bool,
) -> Result<(), crate::Error> {
    #[cfg(feature = "logging")]
    log::trace!("GPU: begin_command_buffer");
    let result = unsafe {
        device.begin_command_buffer(
            command_buffer,
            &vk::CommandBufferBeginInfo {
                s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
                p_next: ptr::null(),
                p_inheritance_info: ptr::null(),
                flags: if one_time_submit {
                    vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT
                } else {
                    vk::CommandBufferUsageFlags::empty()
                },
            },
        )
    };
    match result {
        Ok(_) => (),
        Err(e) => return Err(e.into()),
    }
    Ok(device.check_errors()?)
}

// pub(crate) fn begin_secondary(
//     command_buffer: vk::CommandBuffer,
//     device: &crate::RawDevice,
//     render_pass: Option<vk::RenderPass>,
// ) -> Result<(), crate::Error> {
//     #[cfg(feature = "logging")]
//     log::trace!("GPU: begin_command_buffer");
//     let t = vk::CommandBufferInheritanceInfo {
//         s_type: vk::StructureType::COMMAND_BUFFER_INHERITANCE_INFO,
//         p_next: ptr::null(),
//         render_pass: render_pass.unwrap_or(vk::RenderPass::null()),
//         subpass: 0,
//         framebuffer: vk::Framebuffer::null(),
//         query_flags: vk::QueryControlFlags::empty(),
//         occlusion_query_enable: vk::FALSE,
//         pipeline_statistics: vk::QueryPipelineStatisticFlags::empty(),
//     };
//     let result = unsafe {
//         device.begin_command_buffer(
//             command_buffer,
//             &vk::CommandBufferBeginInfo {
//                 s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
//                 p_next: ptr::null(),
//                 p_inheritance_info: &t,
//                 flags: vk::CommandBufferUsageFlags::empty(),
//             },
//         )
//     };
//     match result {
//         Ok(_) => (),
//         Err(e) => return Err(e.into()),
//     }
//     Ok(device.check_errors()?)
// }

pub(crate) fn end_recording(
    command_buffer: vk::CommandBuffer,
    device: &crate::RawDevice,
) -> Result<(), crate::Error> {
    #[cfg(feature = "logging")]
    log::trace!("GPU: cmd_end_recording");
    let result = unsafe { device.end_command_buffer(command_buffer) };
    match result {
        Ok(_) => (),
        Err(e) => return Err(e.into()),
    }
    Ok(device.check_errors()?)
}

pub(crate) fn begin_compute_pass(
    command_buffer: vk::CommandBuffer,
    device: &crate::RawDevice,
    pipeline: &crate::ComputePipeline,
    garbage: &mut super::Garbage,
) -> Result<(), crate::Error> {
    garbage.pipelines.push(Arc::clone(&pipeline.raw));

    #[cfg(feature = "logging")]
    log::trace!("GPU: cmd_begin_compute_pass pipeine: {:?}", pipeline);
    unsafe {
        device.cmd_bind_pipeline(
            command_buffer,
            vk::PipelineBindPoint::COMPUTE,
            **pipeline.raw,
        )
    }
    Ok(device.check_errors()?)
}

pub(crate) fn begin_graphics_pass<'a, B>(
    command_buffer: vk::CommandBuffer,
    device: &crate::RawDevice,
    color_attachments: &[B],
    resolve_attachments: &[B],
    depth_attachment: Option<B>,
    pipeline: &crate::GraphicsPipeline,
    garbage: &mut super::Garbage,
) -> Result<Option<(vk::Semaphore, vk::Semaphore)>, crate::Error>
where
    B: std::borrow::Borrow<crate::Attachment<'a>>,
{
    #[cfg(feature = "logging")]
    log::trace!("GPU: begin_graphics_pass pipeline: {:?}", pipeline);
    let swapchain = begin_render_pass(
        command_buffer,
        device,
        color_attachments,
        resolve_attachments,
        depth_attachment,
        &pipeline.pass,
        garbage,
    )?;

    garbage
        .pipeline_layouts
        .push(Arc::clone(&pipeline.layout.raw));
    garbage.pipelines.push(Arc::clone(&pipeline.raw));

    unsafe {
        device.cmd_bind_pipeline(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            **pipeline.raw,
        )
    };
    device.check_errors()?;
    Ok(swapchain)
}

pub(crate) fn begin_render_pass<'a, B>(
    command_buffer: vk::CommandBuffer,
    device: &crate::RawDevice,
    color_attachments: &[B],
    resolve_attachments: &[B],
    depth_attachment: Option<B>,
    pass: &crate::RenderPass,
    garbage: &mut super::Garbage,
) -> Result<Option<(vk::Semaphore, vk::Semaphore)>, crate::Error>
where
    B: std::borrow::Borrow<crate::Attachment<'a>>,
{
    garbage.render_passes.push(Arc::clone(&pass.raw));

    let (framebuffer_key, swapchain, extent) = framebuffer_key(
        color_attachments,
        resolve_attachments,
        depth_attachment.as_ref(),
        **pass.raw,
        garbage,
    );
    let framebuffer_cache = device.framebuffers.read();

    if let None = framebuffer_cache.get(&framebuffer_key) {
        drop(framebuffer_cache);
        let framebuffers = color_attachments
            .into_iter()
            .chain(resolve_attachments)
            .chain(depth_attachment.as_ref())
            .map(|v| &**v.borrow().view().framebuffers)
            .collect::<Vec<_>>();
        raw_framebuffer(device, &framebuffer_key, extent, &framebuffers)?;
    }

    let c = device.framebuffers.read();
    let framebuffer = c.get(&framebuffer_key).unwrap();

    garbage.framebuffers.push(Arc::clone(&framebuffer));

    let clear_values = color_attachments
        .into_iter()
        .chain(resolve_attachments)
        .chain(depth_attachment.as_ref())
        .map(|v| v.borrow().clear_value().into())
        .collect::<Vec<_>>();

    unsafe {
        device.cmd_begin_render_pass(
            command_buffer,
            &vk::RenderPassBeginInfo {
                s_type: vk::StructureType::RENDER_PASS_BEGIN_INFO,
                p_next: ptr::null(),
                render_pass: **pass.raw,
                framebuffer: **framebuffer,
                render_area: vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: vk::Extent2D {
                        width: extent.width,
                        height: extent.height,
                    },
                },
                clear_value_count: clear_values.len() as u32,
                p_clear_values: clear_values.as_ptr(),
            },
            vk::SubpassContents::INLINE,
        );
    }

    device.check_errors()?;

    Ok(swapchain)
}

pub(crate) fn framebuffer_key<'a, B>(
    color_attachments: &[B],
    resolve_attachments: &[B],
    depth_attachment: Option<&B>,
    pass: vk::RenderPass,
    garbage: &mut super::Garbage,
) -> (
    crate::FramebufferKey,
    Option<(vk::Semaphore, vk::Semaphore)>,
    crate::Extent2D,
)
where
    B: std::borrow::Borrow<crate::Attachment<'a>>,
{
    let mut swapchain = None;
    let mut extent = crate::Extent2D {
        width: 0,
        height: 0,
    };

    let attachments = color_attachments
        .iter()
        .chain(resolve_attachments)
        .chain(depth_attachment.borrow().iter().map(|a| *a))
        .map(|v| {
            match v.borrow() {
                crate::Attachment::Swapchain(s, _) => {
                    let wait_semaphore = **s
                        .inner
                        .acquire_complete_semaphores
                        .get(s.wait_semaphore)
                        .unwrap();
                    let signal_semaphore = **s
                        .inner
                        .rendering_complete_semaphores
                        .get(s.signal_semaphore)
                        .unwrap();
                    swapchain = Some((wait_semaphore, signal_semaphore));
                    s.drawn.set(true);
                    extent.width = s.view.extent.width;
                    extent.height = s.view.extent.height;

                    garbage.swapchains.push(s.inner.clone());
                    garbage.views.push(Arc::clone(&*s.view.raw));
                    // since it's swapchain know there's no memory
                    // don't cache texture as from swapchain
                }
                crate::Attachment::View(v, _) => {
                    extent.width = v.extent.width;
                    extent.height = v.extent.height;

                    garbage.views.push(Arc::clone(&*v.raw));
                    if let Some(mem) = &v.texture.memory {
                        // somethings gone wrong if v.texture.memory is None
                        garbage.textures.push(Arc::clone(&*v.texture.raw));
                        garbage.memory.push(Arc::clone(mem));
                    }
                }
            }
            **v.borrow().view().raw
        })
        .collect::<Vec<_>>();

    (
        crate::FramebufferKey {
            attachments,
            render_pass: pass,
        },
        swapchain,
        extent,
    )
}

pub(crate) fn raw_framebuffer(
    device: &crate::RawDevice,
    framebuffer_key: &crate::FramebufferKey,
    extent: crate::Extent2D,
    caches: &[&Mutex<Vec<crate::FramebufferKey>>],
) -> Result<vk::Framebuffer, crate::Error> {
    let framebuffer_create_info = vk::FramebufferCreateInfo {
        s_type: vk::StructureType::FRAMEBUFFER_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::FramebufferCreateFlags::empty(),
        render_pass: framebuffer_key.render_pass,
        attachment_count: framebuffer_key.attachments.len() as u32,
        p_attachments: framebuffer_key.attachments.as_ptr(),
        width: extent.width,
        height: extent.height,
        layers: 1,
    };

    let framebuffer_result = unsafe { device.create_framebuffer(&framebuffer_create_info, None) };

    let f = match framebuffer_result {
        Ok(f) => {
            let framebuffer = Arc::new(f);
            device
                .framebuffers
                .write()
                .insert(framebuffer_key.clone(), framebuffer);
            for framebuffers in caches {
                framebuffers.lock().push(framebuffer_key.clone());
            }
            f
        }
        Err(e) => return Err(e.into()),
    };

    device.check_errors()?;
    Ok(f)
}

pub(crate) fn end_render_pass(
    command_buffer: vk::CommandBuffer,
    device: &crate::RawDevice,
) -> Result<(), crate::Error> {
    #[cfg(feature = "logging")]
    log::trace!("GPU: cmd_end_graphics_pass");
    unsafe { device.cmd_end_render_pass(command_buffer) }
    Ok(device.check_errors()?)
}

pub(crate) fn draw_indirect(
    command_buffer: vk::CommandBuffer,
    device: &crate::RawDevice,
    buffer: &crate::Buffer,
    offset: u64,
    draw_count: u32,
    stride: u32,
    garbage: &mut super::Garbage,
) -> Result<(), crate::Error> {
    garbage.buffers.push(Arc::clone(&buffer.raw));
    garbage.memory.push(Arc::clone(&buffer.memory));

    unsafe {
        device.cmd_draw_indirect(
            command_buffer, 
            **buffer.raw, 
            offset, 
            draw_count, 
            stride
        )
    }

    Ok(device.check_errors()?)
}

pub(crate) fn draw_indexed_indirect(
    command_buffer: vk::CommandBuffer,
    device: &crate::RawDevice,
    buffer: &crate::Buffer,
    offset: u64,
    draw_count: u32,
    stride: u32,
    garbage: &mut super::Garbage,
) -> Result<(), crate::Error> {
    garbage.buffers.push(Arc::clone(&buffer.raw));
    garbage.memory.push(Arc::clone(&buffer.memory));

    unsafe {
        device.cmd_draw_indexed_indirect(
            command_buffer, 
            **buffer.raw, 
            offset, 
            draw_count, 
            stride
        )
    }

    Ok(device.check_errors()?)
}

pub(crate) fn draw(
    command_buffer: vk::CommandBuffer,
    device: &crate::RawDevice,
    first_vertex: u32,
    vertex_count: u32,
    first_instance: u32,
    instance_count: u32,
) -> Result<(), crate::Error> {
    #[cfg(feature = "logging")]
    log::trace!(
        "GPU: cmd_draw first vertex {}, vertex count {}, first_instance {}, instance_count {}",
        first_vertex,
        vertex_count,
        first_instance,
        instance_count
    );
    unsafe {
        device.cmd_draw(
            command_buffer,
            vertex_count,
            instance_count,
            first_vertex,
            first_instance,
        );
    }
    Ok(device.check_errors()?)
}

pub(crate) fn draw_indexed(
    command_buffer: vk::CommandBuffer,
    device: &crate::RawDevice,
    first_index: u32,
    index_count: u32,
    first_instance: u32,
    instance_count: u32,
    vertex_offset: i32,
) -> Result<(), crate::Error> {
    #[cfg(feature = "logging")]
    log::trace!("GPU: cmd_draw_indexed first_index: {}, index_count: {}, first_instance: {}, instance_count: {}, vertex_offset: {}", first_index, index_count, first_instance, instance_count, vertex_offset);
    unsafe {
        device.cmd_draw_indexed(
            command_buffer,
            index_count,
            instance_count,
            first_index,
            vertex_offset,
            first_instance,
        )
    }
    Ok(device.check_errors()?)
}

pub(crate) fn dispatch(
    command_buffer: vk::CommandBuffer,
    device: &crate::RawDevice,
    x: u32,
    y: u32,
    z: u32,
) -> Result<(), crate::Error> {
    #[cfg(feature = "logging")]
    log::trace!("GPU: cmd_dispatch x: {}, y: {}, z: {}", x, y, z);
    unsafe { device.cmd_dispatch(command_buffer, x, y, z) };
    Ok(device.check_errors()?)
}

pub(crate) fn bind_vertex_buffers<'a, B>(
    command_buffer: vk::CommandBuffer,
    device: &crate::RawDevice,
    buffers: &[B],
    first_binding: u32,
    garbage: &mut super::Garbage,
) -> Result<(), crate::Error>
where
    B: Borrow<crate::BufferSlice<'a>>,
{
    #[cfg(feature = "logging")]
    log::trace!("GPU: cmd_bind_vertex_buffers");
    let raw_buffers = buffers
        .iter()
        .map(|b| {
            let slice: &'_ crate::BufferSlice = b.borrow();
            garbage.buffers.push(Arc::clone(&*slice.buffer.raw));
            garbage.memory.push(Arc::clone(&*slice.buffer.memory));
            **slice.buffer.raw
        })
        .collect::<Vec<_>>();
    let offsets = buffers
        .iter()
        .map(|b| b.borrow().offset)
        .collect::<Vec<_>>();
    unsafe {
        device.cmd_bind_vertex_buffers(command_buffer, first_binding, &raw_buffers, &offsets)
    };
    Ok(device.check_errors()?)
}

pub(crate) fn bind_index_buffer<'a, B>(
    command_buffer: vk::CommandBuffer,
    device: &crate::RawDevice,
    buffer: B,
    ty: crate::IndexType,
    garbage: &mut super::Garbage,
) -> Result<(), crate::Error>
where
    B: Borrow<crate::BufferSlice<'a>>,
{
    garbage
        .buffers
        .push(Arc::clone(&*buffer.borrow().buffer.raw));
    garbage
        .memory
        .push(Arc::clone(&*buffer.borrow().buffer.memory));

    #[cfg(feature = "logging")]
    log::trace!("GPU: cmd_bind_index_buffer {:?}", buffer.borrow());
    unsafe {
        device.cmd_bind_index_buffer(
            command_buffer,
            **buffer.borrow().buffer.raw,
            buffer.borrow().offset,
            ty.into(),
        )
    };
    Ok(device.check_errors()?)
}

// pub(crate) fn execute_secondary(
//     command_buffer: vk::CommandBuffer,
//     device: &crate::RawDevice,
//     secondary_buffer: vk::CommandBuffer,
// ) -> Result<(), crate::Error> {
//     #[cfg(feature = "logging")]
//     log::trace!("GPU: cmd_execute_secondary {:?}", secondary_buffer);
//     unsafe { device.cmd_execute_commands(command_buffer, &[secondary_buffer]) };
//     Ok(device.check_errors()?)
// }

pub(crate) fn bind_descriptors<G>(
    command_buffer: vk::CommandBuffer,
    device: &crate::RawDevice,
    first_location: u32,
    groups: &[G],
    bind_point: crate::PipelineBindPoint,
    layout: &crate::PipelineLayout,
    garbage: &mut super::Garbage,
) -> Result<(), crate::Error>
where
    G: Borrow<crate::DescriptorSet>,
{
    #[cfg(feature = "logging")]
    log::trace!("GPU: cmd_set_descriptors");
    let descriptor_sets = groups
        .iter()
        .map(|g| {
            let set: &'_ crate::DescriptorSet = g.borrow();
            for buffer in &*set.buffers {
                garbage.buffers.push(Arc::clone(&*buffer.buffer.raw));
                garbage.memory.push(Arc::clone(&*buffer.buffer.memory));
            }
            for texture in &*set.textures {
                garbage.textures.push(Arc::clone(&*texture.0.texture.raw));
                garbage.views.push(Arc::clone(&*texture.0.raw));
                if let Some(mem) = &texture.0.texture.memory {
                    garbage.memory.push(Arc::clone(mem));
                }
            }
            for sampler in &*set.samplers {
                garbage.samplers.push(Arc::clone(&*sampler.raw));
            }
            garbage
                .descriptor_layouts
                .push(Arc::clone(&*g.borrow().layout));
            garbage.descriptor_pools.push(Arc::clone(&*g.borrow().pool));
            **g.borrow().set
        })
        .collect::<Vec<_>>();
    unsafe {
        device.cmd_bind_descriptor_sets(
            command_buffer,
            bind_point.into(),
            **layout.raw,
            first_location,
            &descriptor_sets,
            &[],
        )
    };
    Ok(device.check_errors()?)
}

pub(crate) fn push_constants(
    command_buffer: vk::CommandBuffer,
    device: &crate::RawDevice,
    offset: u32,
    constants: &[u8],
    stages: crate::ShaderStages,
    layout: &crate::PipelineLayout,
) -> Result<(), crate::Error> {
    #[cfg(feature = "logging")]
    log::trace!("GPU: cmd_push_constants");
    unsafe {
        device.cmd_push_constants(
            command_buffer,
            **layout.raw,
            stages.into(),
            offset,
            constants,
        )
    }
    Ok(device.check_errors()?)
}

pub(crate) fn write_timestamp(
    command_buffer: vk::CommandBuffer,
    device: &crate::RawDevice,
    query: &crate::TimeQuery,
    pipeline_stage: crate::PipelineStage,
    index: u32,
    garbage: &mut super::Garbage,
) -> Result<(), crate::Error> {
    garbage.queries.push(Arc::clone(&query.raw));
    unsafe { device.cmd_write_timestamp(command_buffer, pipeline_stage.into(), **query.raw, index) }

    Ok(device.check_errors()?)
}

pub(crate) fn reset_time_query(
    command_buffer: vk::CommandBuffer,
    device: &crate::RawDevice,
    query: &crate::TimeQuery,
    first_query: u32,
    query_count: u32,
    garbage: &mut super::Garbage,
) -> Result<(), crate::Error> {
    garbage.queries.push(Arc::clone(&query.raw));
    unsafe { device.cmd_reset_query_pool(command_buffer, **query.raw, first_query, query_count) }
    Ok(device.check_errors()?)
}

pub(crate) fn submit(
    device: &crate::RawDevice,
    queue: vk::Queue,
    command_buffer: vk::CommandBuffer,
    semaphore: &Arc<vk::Semaphore>,
    swapchain_sync: Option<(vk::Semaphore, vk::Semaphore)>,
    fence: vk::Fence,
    garbage: &mut super::Garbage,
) -> Result<(), crate::Error> {
    #[cfg(feature = "logging")]
    log::trace!("GPU: cmd_submit");
    let reset_result = unsafe { device.reset_fences(&[fence]) };
    match reset_result {
        Ok(_) => (),
        Err(e) => return Err(e.into()),
    }
    // get the semaphore of the last command to have been submitted and use it to wait on
    let mut semaphores = device.semaphores.lock();

    let mut wait_semaphores = Vec::new();
    let mut signal_semaphores = Vec::new();

    if let Some(s) = semaphores.get(&std::thread::current().id()) {
        garbage.prev_semaphore = Some(Arc::clone(s));
        wait_semaphores.push(**s);
    }
    signal_semaphores.push(**semaphore);
    if let Some((wait, signal)) = swapchain_sync {
        wait_semaphores.push(wait);
        signal_semaphores.push(signal);
    }
    semaphores.insert(std::thread::current().id(), Arc::clone(semaphore));

    let wait_dst_stage_mask = if wait_semaphores.len() == 0 {
        [vk::PipelineStageFlags::empty(); 2]
    } else if wait_semaphores.len() == 1 {
        [
            vk::PipelineStageFlags::BOTTOM_OF_PIPE,
            vk::PipelineStageFlags::empty(),
        ]
    } else {
        [
            vk::PipelineStageFlags::BOTTOM_OF_PIPE,
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        ]
    };

    let submit_info = vk::SubmitInfo {
        s_type: vk::StructureType::SUBMIT_INFO,
        p_next: ptr::null(),
        wait_semaphore_count: wait_semaphores.len() as _,
        p_wait_semaphores: wait_semaphores.as_ptr(),
        p_wait_dst_stage_mask: &wait_dst_stage_mask[0],
        signal_semaphore_count: signal_semaphores.len() as _,
        p_signal_semaphores: signal_semaphores.as_ptr(),
        command_buffer_count: 1,
        p_command_buffers: &command_buffer,
    };

    // This crashes only on release builds
    // maybe bug in compiler?? idk it's 3am rn
    // validation layers complain about having invalid p_wait_dst_stage_mask
    // including bits that arn't valid and even not recognized members

    // let submit_info = vk::SubmitInfo {
    //     s_type: vk::StructureType::SUBMIT_INFO,
    //     p_next: ptr::null(),
    //     wait_semaphore_count: wait_semaphores.len() as _,
    //     p_wait_semaphores: wait_semaphores.as_ptr(),
    //     p_wait_dst_stage_mask: if wait_semaphores.len() == 0 {
    //          &vk::PipelineStageFlags::empty()
    //     } else {
    //         &vk::PipelineStageFlags::BOTTOM_OF_PIPE
    //     },
    //     signal_semaphore_count: signal_semaphores.len() as _,
    //     p_signal_semaphores: signal_semaphores.as_ptr(),
    //     command_buffer_count: 1,
    //     p_command_buffers: &command_buffer,
    // };

    // println!("{:?}", unsafe { *submit_info.p_wait_dst_stage_mask });

    //      [2022-04-15T01:08:53Z ERROR gpu::ffi] GPU VALIDATION "Validation Error: [ UNASSIGNED-GeneralParameterError-UnrecognizedValue ] Object 0: handle = 0x55d82e546570, type = VK_OBJECT_TYPE_DEVICE; | MessageID = 0xbe6eff91 | vkQueueSubmit: value of pSubmits[0].pWaitDstStageMask[1] contains flag bits that are not recognized members of VkPipelineStageFlagBits"
    //      [Debug]ERROR[Validation]"Validation Error: [ UNASSIGNED-GeneralParameterError-UnrecognizedValue ] Object 0: handle = 0x55d82e546570, type = VK_OBJECT_TYPE_DEVICE; | MessageID = 0xbe6eff91 | vkQueueSubmit: value of pSubmits[0].pWaitDstStageMask[1] contains flag bits that are not recognized members of VkPipelineStageFlagBits"
    //      [2022-04-15T01:08:53Z ERROR gpu::ffi] GPU VALIDATION "Validation Error: [ VUID-vkQueueSubmit-pWaitDstStageMask-00066 ] Object 0: handle = 0x967dd1000000000e, type = VK_OBJECT_TYPE_SEMAPHORE; Object 1: handle = 0x55d82e4a6c00, type = VK_OBJECT_TYPE_QUEUE; | MessageID = 0x48e8bcee | vkQueueSubmit(): pSubmits[0].pWaitDstStageMask[1] flag Unhandled VkPipelineStageFlagBits is not compatible with the queue family properties (VK_QUEUE_GRAPHICS_BIT|VK_QUEUE_COMPUTE_BIT|VK_QUEUE_TRANSFER_BIT) of this command buffer. The Vulkan spec states: Any stage flag included in any element of the pWaitDstStageMask member of any element of pSubmits must be a pipeline stage supported by one of the capabilities of queue, as specified in the table of supported pipeline stages (https://vulkan.lunarg.com/doc/view/1.2.189.0/linux/1.2-extensions/vkspec.html#VUID-vkQueueSubmit-pWaitDstStageMask-00066)"
    //      [Debug]ERROR[Validation]"Validation Error: [ VUID-vkQueueSubmit-pWaitDstStageMask-00066 ] Object 0: handle = 0x967dd1000000000e, type = VK_OBJECT_TYPE_SEMAPHORE; Object 1: handle = 0x55d82e4a6c00, type = VK_OBJECT_TYPE_QUEUE; | MessageID = 0x48e8bcee | vkQueueSubmit(): pSubmits[0].pWaitDstStageMask[1] flag Unhandled VkPipelineStageFlagBits is not compatible with the queue family properties (VK_QUEUE_GRAPHICS_BIT|VK_QUEUE_COMPUTE_BIT|VK_QUEUE_TRANSFER_BIT) of this command buffer. The Vulkan spec states: Any stage flag included in any element of the pWaitDstStageMask member of any element of pSubmits must be a pipeline stage supported by one of the capabilities of queue, as specified in the table of supported pipeline stages (https://vulkan.lunarg.com/doc/view/1.2.189.0/linux/1.2-extensions/vkspec.html#VUID-vkQueueSubmit-pWaitDstStageMask-00066)"
    //      [2022-04-15T01:08:53Z ERROR gpu::ffi] GPU VALIDATION "Validation Error: [ VUID-vkQueueSubmit-pWaitDstStageMask-00066 ] Object 0: handle = 0x967dd1000000000e, type = VK_OBJECT_TYPE_SEMAPHORE; Object 1: handle = 0x55d82e4a6c00, type = VK_OBJECT_TYPE_QUEUE; | MessageID = 0x48e8bcee | vkQueueSubmit(): pSubmits[0].pWaitDstStageMask[1] flag Unhandled VkPipelineStageFlagBits is not compatible with the queue family properties (VK_QUEUE_GRAPHICS_BIT|VK_QUEUE_COMPUTE_BIT|VK_QUEUE_TRANSFER_BIT) of this command buffer. The Vulkan spec states: Any stage flag included in any element of the pWaitDstStageMask member of any element of pSubmits must be a pipeline stage supported by one of the capabilities of queue, as specified in the table of supported pipeline stages (https://vulkan.lunarg.com/doc/view/1.2.189.0/linux/1.2-extensions/vkspec.html#VUID-vkQueueSubmit-pWaitDstStageMask-00066)"
    //      [Debug]ERROR[Validation]"Validation Error: [ VUID-vkQueueSubmit-pWaitDstStageMask-00066 ] Object 0: handle = 0x967dd1000000000e, type = VK_OBJECT_TYPE_SEMAPHORE; Object 1: handle = 0x55d82e4a6c00, type = VK_OBJECT_TYPE_QUEUE; | MessageID = 0x48e8bcee | vkQueueSubmit(): pSubmits[0].pWaitDstStageMask[1] flag Unhandled VkPipelineStageFlagBits is not compatible with the queue family properties (VK_QUEUE_GRAPHICS_BIT|VK_QUEUE_COMPUTE_BIT|VK_QUEUE_TRANSFER_BIT) of this command buffer. The Vulkan spec states: Any stage flag included in any element of the pWaitDstStageMask member of any element of pSubmits must be a pipeline stage supported by one of the capabilities of queue, as specified in the table of supported pipeline stages (https://vulkan.lunarg.com/doc/view/1.2.189.0/linux/1.2-extensions/vkspec.html#VUID-vkQueueSubmit-pWaitDstStageMask-00066)"
    //      [2022-04-15T01:08:53Z ERROR gpu::ffi] GPU VALIDATION "Validation Error: [ VUID-vkQueueSubmit-pWaitDstStageMask-00066 ] Object 0: handle = 0x967dd1000000000e, type = VK_OBJECT_TYPE_SEMAPHORE; Object 1: handle = 0x55d82e4a6c00, type = VK_OBJECT_TYPE_QUEUE; | MessageID = 0x48e8bcee | vkQueueSubmit(): pSubmits[0].pWaitDstStageMask[1] flag Unhandled VkPipelineStageFlagBits is not compatible with the queue family properties (VK_QUEUE_GRAPHICS_BIT|VK_QUEUE_COMPUTE_BIT|VK_QUEUE_TRANSFER_BIT) of this command buffer. The Vulkan spec states: Any stage flag included in any element of the pWaitDstStageMask member of any element of pSubmits must be a pipeline stage supported by one of the capabilities of queue, as specified in the table of supported pipeline stages (https://vulkan.lunarg.com/doc/view/1.2.189.0/linux/1.2-extensions/vkspec.html#VUID-vkQueueSubmit-pWaitDstStageMask-00066)"
    //      [Debug]ERROR[Validation]"Validation Error: [ VUID-vkQueueSubmit-pWaitDstStageMask-00066 ] Object 0: handle = 0x967dd1000000000e, type = VK_OBJECT_TYPE_SEMAPHORE; Object 1: handle = 0x55d82e4a6c00, type = VK_OBJECT_TYPE_QUEUE; | MessageID = 0x48e8bcee | vkQueueSubmit(): pSubmits[0].pWaitDstStageMask[1] flag Unhandled VkPipelineStageFlagBits is not compatible with the queue family properties (VK_QUEUE_GRAPHICS_BIT|VK_QUEUE_COMPUTE_BIT|VK_QUEUE_TRANSFER_BIT) of this command buffer. The Vulkan spec states: Any stage flag included in any element of the pWaitDstStageMask member of any element of pSubmits must be a pipeline stage supported by one of the capabilities of queue, as specified in the table of supported pipeline stages (https://vulkan.lunarg.com/doc/view/1.2.189.0/linux/1.2-extensions/vkspec.html#VUID-vkQueueSubmit-pWaitDstStageMask-00066)"
    //      [2022-04-15T01:08:53Z ERROR gpu::ffi] GPU VALIDATION "Validation Error: [ VUID-vkQueueSubmit-pWaitDstStageMask-00066 ] Object 0: handle = 0x967dd1000000000e, type = VK_OBJECT_TYPE_SEMAPHORE; Object 1: handle = 0x55d82e4a6c00, type = VK_OBJECT_TYPE_QUEUE; | MessageID = 0x48e8bcee | vkQueueSubmit(): pSubmits[0].pWaitDstStageMask[1] flag Unhandled VkPipelineStageFlagBits is not compatible with the queue family properties (VK_QUEUE_GRAPHICS_BIT|VK_QUEUE_COMPUTE_BIT|VK_QUEUE_TRANSFER_BIT) of this command buffer. The Vulkan spec states: Any stage flag included in any element of the pWaitDstStageMask member of any element of pSubmits must be a pipeline stage supported by one of the capabilities of queue, as specified in the table of supported pipeline stages (https://vulkan.lunarg.com/doc/view/1.2.189.0/linux/1.2-extensions/vkspec.html#VUID-vkQueueSubmit-pWaitDstStageMask-00066)"
    //      [Debug]ERROR[Validation]"Validation Error: [ VUID-vkQueueSubmit-pWaitDstStageMask-00066 ] Object 0: handle = 0x967dd1000000000e, type = VK_OBJECT_TYPE_SEMAPHORE; Object 1: handle = 0x55d82e4a6c00, type = VK_OBJECT_TYPE_QUEUE; | MessageID = 0x48e8bcee | vkQueueSubmit(): pSubmits[0].pWaitDstStageMask[1] flag Unhandled VkPipelineStageFlagBits is not compatible with the queue family properties (VK_QUEUE_GRAPHICS_BIT|VK_QUEUE_COMPUTE_BIT|VK_QUEUE_TRANSFER_BIT) of this command buffer. The Vulkan spec states: Any stage flag included in any element of the pWaitDstStageMask member of any element of pSubmits must be a pipeline stage supported by one of the capabilities of queue, as specified in the table of supported pipeline stages (https://vulkan.lunarg.com/doc/view/1.2.189.0/linux/1.2-extensions/vkspec.html#VUID-vkQueueSubmit-pWaitDstStageMask-00066)"
    //      [2022-04-15T01:08:53Z ERROR gpu::ffi] GPU VALIDATION "Validation Error: [ VUID-VkSubmitInfo-pWaitDstStageMask-00076 ] Object 0: handle = 0x967dd1000000000e, type = VK_OBJECT_TYPE_SEMAPHORE; Object 1: handle = 0x55d82e4a6c00, type = VK_OBJECT_TYPE_QUEUE; | MessageID = 0x974ac677 | vkQueueSubmit(): pSubmits[0].pWaitDstStageMask[1] includes VK_PIPELINE_STAGE_GEOMETRY_SHADER_BIT when the device does not have geometryShader feature enabled. The Vulkan spec states: If the geometry shaders feature is not enabled, each element of pWaitDstStageMask must not contain VK_PIPELINE_STAGE_GEOMETRY_SHADER_BIT (https://vulkan.lunarg.com/doc/view/1.2.189.0/linux/1.2-extensions/vkspec.html#VUID-VkSubmitInfo-pWaitDstStageMask-00076)"
    //      [Debug]ERROR[Validation]"Validation Error: [ VUID-VkSubmitInfo-pWaitDstStageMask-00076 ] Object 0: handle = 0x967dd1000000000e, type = VK_OBJECT_TYPE_SEMAPHORE; Object 1: handle = 0x55d82e4a6c00, type = VK_OBJECT_TYPE_QUEUE; | MessageID = 0x974ac677 | vkQueueSubmit(): pSubmits[0].pWaitDstStageMask[1] includes VK_PIPELINE_STAGE_GEOMETRY_SHADER_BIT when the device does not have geometryShader feature enabled. The Vulkan spec states: If the geometry shaders feature is not enabled, each element of pWaitDstStageMask must not contain VK_PIPELINE_STAGE_GEOMETRY_SHADER_BIT (https://vulkan.lunarg.com/doc/view/1.2.189.0/linux/1.2-extensions/vkspec.html#VUID-VkSubmitInfo-pWaitDstStageMask-00076)"
    //      [2022-04-15T01:08:53Z ERROR gpu::ffi] GPU VALIDATION "Validation Error: [ UNASSIGNED-CoreChecks-VkSubmitInfo-pWaitDstStageMask-conditionalRendering ] Object 0: handle = 0x967dd1000000000e, type = VK_OBJECT_TYPE_SEMAPHORE; Object 1: handle = 0x55d82e4a6c00, type = VK_OBJECT_TYPE_QUEUE; | MessageID = 0x76a3c53 | vkQueueSubmit(): pSubmits[0].pWaitDstStageMask[1] includes VK_PIPELINE_STAGE_CONDITIONAL_RENDERING_BIT_EXT when the device does not have conditionalRendering feature enabled."
    //      [Debug]ERROR[Validation]"Validation Error: [ UNASSIGNED-CoreChecks-VkSubmitInfo-pWaitDstStageMask-conditionalRendering ] Object 0: handle = 0x967dd1000000000e, type = VK_OBJECT_TYPE_SEMAPHORE; Object 1: handle = 0x55d82e4a6c00, type = VK_OBJECT_TYPE_QUEUE; | MessageID = 0x76a3c53 | vkQueueSubmit(): pSubmits[0].pWaitDstStageMask[1] includes VK_PIPELINE_STAGE_CONDITIONAL_RENDERING_BIT_EXT when the device does not have conditionalRendering feature enabled."
    //      [2022-04-15T01:08:53Z ERROR gpu::ffi] GPU VALIDATION "Validation Error: [ VUID-VkSubmitInfo-pWaitDstStageMask-02090 ] Object 0: handle = 0x967dd1000000000e, type = VK_OBJECT_TYPE_SEMAPHORE; Object 1: handle = 0x55d82e4a6c00, type = VK_OBJECT_TYPE_QUEUE; | MessageID = 0x3702e1cc | vkQueueSubmit(): pSubmits[0].pWaitDstStageMask[1] includes VK_PIPELINE_STAGE_TASK_SHADER_BIT_NV when the device does not have taskShader feature enabled. The Vulkan spec states: If the task shaders feature is not enabled, each element of pWaitDstStageMask must not contain VK_PIPELINE_STAGE_TASK_SHADER_BIT_NV (https://vulkan.lunarg.com/doc/view/1.2.189.0/linux/1.2-extensions/vkspec.html#VUID-VkSubmitInfo-pWaitDstStageMask-02090)"
    //      [Debug]ERROR[Validation]"Validation Error: [ VUID-VkSubmitInfo-pWaitDstStageMask-02090 ] Object 0: handle = 0x967dd1000000000e, type = VK_OBJECT_TYPE_SEMAPHORE; Object 1: handle = 0x55d82e4a6c00, type = VK_OBJECT_TYPE_QUEUE; | MessageID = 0x3702e1cc | vkQueueSubmit(): pSubmits[0].pWaitDstStageMask[1] includes VK_PIPELINE_STAGE_TASK_SHADER_BIT_NV when the device does not have taskShader feature enabled. The Vulkan spec states: If the task shaders feature is not enabled, each element of pWaitDstStageMask must not contain VK_PIPELINE_STAGE_TASK_SHADER_BIT_NV (https://vulkan.lunarg.com/doc/view/1.2.189.0/linux/1.2-extensions/vkspec.html#VUID-VkSubmitInfo-pWaitDstStageMask-02090)"
    //      [2022-04-15T01:08:53Z ERROR gpu::ffi] GPU VALIDATION "Validation Error: [ UNASSIGNED-CoreChecks-VkSubmitInfo-pWaitDstStageMask-shadingRate ] Object 0: handle = 0x967dd1000000000e, type = VK_OBJECT_TYPE_SEMAPHORE; Object 1: handle = 0x55d82e4a6c00, type = VK_OBJECT_TYPE_QUEUE; | MessageID = 0xb718b4cd | vkQueueSubmit(): pSubmits[0].pWaitDstStageMask[1] includes VK_PIPELINE_STAGE_FRAGMENT_SHADING_RATE_ATTACHMENT_BIT_KHR when the device does not have shadingRate feature enabled."
    //      [Debug]ERROR[Validation]"Validation Error: [ UNASSIGNED-CoreChecks-VkSubmitInfo-pWaitDstStageMask-shadingRate ] Object 0: handle = 0x967dd1000000000e, type = VK_OBJECT_TYPE_SEMAPHORE; Object 1: handle = 0x55d82e4a6c00, type = VK_OBJECT_TYPE_QUEUE; | MessageID = 0xb718b4cd | vkQueueSubmit(): pSubmits[0].pWaitDstStageMask[1] includes VK_PIPELINE_STAGE_FRAGMENT_SHADING_RATE_ATTACHMENT_BIT_KHR when the device does not have shadingRate feature enabled."
    //      [2022-04-15T01:08:53Z ERROR gpu::ffi] GPU VALIDATION "Validation Error: [ VUID-VkSubmitInfo-pWaitDstStageMask-00078 ] Object 0: handle = 0x55d82e546570, type = VK_OBJECT_TYPE_DEVICE; | MessageID = 0xc17cd9ae | vkQueueSubmit(): pSubmits[0].pWaitDstStageMask[1] stage mask must not include VK_PIPELINE_STAGE_HOST_BIT as the stage can't be invoked inside a command buffer. The Vulkan spec states: Each element of pWaitDstStageMask must not include VK_PIPELINE_STAGE_HOST_BIT (https://vulkan.lunarg.com/doc/view/1.2.189.0/linux/1.2-extensions/vkspec.html#VUID-VkSubmitInfo-pWaitDstStageMask-00078)"

    let submit_result = unsafe { device.queue_submit(queue, &[submit_info], fence) };

    match submit_result {
        Ok(_) => (),
        Err(e) => return Err(e.into()),
    }

    Ok(device.check_errors()?)
}
