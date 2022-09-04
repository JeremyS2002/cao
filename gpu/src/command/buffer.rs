use std::borrow::Borrow;
use std::mem::ManuallyDrop as Md;
use std::ptr;
use std::sync::Arc;

use super::raw;

use ash::vk;

pub struct CommandBuffer {
    pub(crate) name: Option<String>,

    pub(crate) pool: vk::CommandPool,
    pub(crate) buffer: vk::CommandBuffer,

    pub(crate) semaphore: Md<Arc<vk::Semaphore>>,
    pub(crate) fence: vk::Fence,

    pub(crate) queue: vk::Queue,
    pub(crate) device: Arc<crate::RawDevice>,
    /// version shouldn't overflow
    ///
    /// assume 120fps and submit command buffer 1000 times per frame
    /// (i don't think anyone would submit 1000 times per frame but why not)
    /// ((1/120)*u64::max / 1000) / (60 * 60 * 24 * 365) ~= 5million years
    pub(crate) version: u64,

    pub(crate) swapchain: Option<(vk::Semaphore, vk::Semaphore)>,
    pub(crate) garbage: super::Garbage,
}

impl std::fmt::Debug for CommandBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CommandBuffer id: {:?} name: {:?}", self.pool, self.name)
    }
}

impl CommandBuffer {
    pub unsafe fn raw_pool(&self) -> vk::CommandPool {
        self.pool
    }

    pub unsafe fn raw_command_buffer(&self) -> vk::CommandBuffer {
        self.buffer
    }

    pub unsafe fn raw_semaphore(&self) -> vk::Semaphore {
        **self.semaphore
    }

    pub unsafe fn raw_fence(&self) -> vk::Fence {
        self.fence
    }

    pub unsafe fn raw_queue(&self) -> vk::Queue {
        self.queue
    }
}

impl CommandBuffer {
    pub fn new(device: &crate::Device, name: Option<String>) -> Result<Self, crate::Error> {
        let pool_create_info = vk::CommandPoolCreateInfo {
            s_type: vk::StructureType::COMMAND_POOL_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
            queue_family_index: device.queue_family,
        };

        let pool_result = unsafe { device.raw.create_command_pool(&pool_create_info, None) };

        let pool = match pool_result {
            Ok(p) => p,
            Err(e) => return Err(e.into()),
        };

        let buffer_alloc_info = vk::CommandBufferAllocateInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            p_next: ptr::null(),
            command_buffer_count: 1,
            command_pool: pool,
            level: vk::CommandBufferLevel::PRIMARY,
        };

        let buffer_result = unsafe { device.raw.allocate_command_buffers(&buffer_alloc_info) };

        let buffer = match buffer_result {
            Ok(b) => b[0],
            Err(e) => return Err(e.into()),
        };

        let fence_create_info = vk::FenceCreateInfo {
            s_type: vk::StructureType::FENCE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::FenceCreateFlags::SIGNALED,
        };

        let fence_result = unsafe { device.raw.create_fence(&fence_create_info, None) };

        let fence = match fence_result {
            Ok(f) => f,
            Err(e) => return Err(e.into()),
        };

        let semaphore_create_info = vk::SemaphoreCreateInfo {
            s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::SemaphoreCreateFlags::empty(),
        };

        let semaphore_result = unsafe { device.raw.create_semaphore(&semaphore_create_info, None) };

        let semaphore = match semaphore_result {
            Ok(s) => s,
            Err(e) => return Err(e.into()),
        };

        let s = Self {
            name: name,
            pool,
            buffer,
            fence,
            semaphore: Md::new(Arc::new(semaphore)),
            queue: device.queue,
            device: Arc::clone(&device.raw),
            version: 0,
            swapchain: None,
            garbage: super::Garbage::default(),
        };

        if let Some(name) = &s.name {
            device.raw.set_command_buffer_name(&s, name)?;
        }

        device.raw.check_errors()?;

        Ok(s)
    }

    /// <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkQueueSubmit.html>
    pub fn submit(&mut self) -> Result<(), crate::Error> {
        self.wait(!0)?;
        raw::submit(
            &self.device,
            self.queue,
            self.buffer,
            &self.semaphore,
            self.swapchain,
            self.fence,
            &mut self.garbage,
        )
    }

    /// <https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/vkWaitForFences.html>
    pub fn wait(&mut self, timeout: u64) -> Result<(), crate::Error> {
        let wait_result = unsafe { self.device.wait_for_fences(&[self.fence], true, timeout) };

        match wait_result {
            Ok(_) => Ok(()),
            Err(e) => return Err(e.into()),
        }
    }

    /// <https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/vkResetCommandPool.html>
    pub fn reset(&mut self) -> Result<(), crate::Error> {
        self.wait(!0)?;
        unsafe {
            self.garbage.clean(&self.device);
        }

        self.version += 1;
        let result = unsafe {
            self.device
                .reset_command_pool(self.pool, vk::CommandPoolResetFlags::empty())
        };

        match result {
            Ok(_) => Ok(()),
            Err(e) => return Err(e.into()),
        }
    }

    /// Get a unique id of the command bufer
    /// equivalent to buffer.id() == mem::transmute(buffer.raw_pool())
    pub fn id(&self) -> u64 {
        unsafe { std::mem::transmute(self.raw_pool()) }
    }

    /// Command buffers keep track of how many times they have been recorded to
    pub fn version(&self) -> u64 {
        self.version
    }

    /// <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkBeginCommandBuffer.html>
    pub fn begin(&mut self, one_time_submit: bool) -> Result<(), crate::Error> {
        // wait for previous submission to complete if any
        if self.version != 0 {
            self.wait(!0)?;
            unsafe {
                self.garbage.clean(&self.device);
            }
        }

        self.version += 1;
        raw::begin_primary(self.buffer, &self.device, one_time_submit)
    }

    /// <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkCmdPipelineBarrier.html>
    pub fn end(&mut self) -> Result<(), crate::Error> {
        raw::end_recording(self.buffer, &self.device)
    }

    /// <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkCmdPipelineBarrier.html>
    pub fn pipeline_barrier(
        &mut self,
        src_stages: crate::PipelineStageFlags,
        dst_stages: crate::PipelineStageFlags,
        buffers: &[crate::BufferAccessInfo<'_>],
        textures: &[crate::TextureAccessInfo<'_>],
    ) -> Result<(), crate::Error> {
        raw::pipeline_barrier(
            self.buffer,
            &self.device,
            src_stages,
            dst_stages,
            buffers,
            textures,
        )
    }

    /// <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkCmdUpdateBuffer.html>
    pub fn update_buffer<B>(
        &mut self,
        buffer: B,
        offset: u64,
        data: &[u8],
    ) -> Result<(), crate::Error>
    where
        B: Borrow<crate::Buffer>,
    {
        raw::update_buffer(
            self.buffer,
            &self.device,
            buffer,
            offset,
            data,
            &mut self.garbage,
        )
    }

    /// <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkCmdClearColorImage.html>
    /// <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkCmdClearDepthStencilImage.html>
    pub fn clear_texture<'a, T>(
        &mut self,
        texture: T,
        layout: crate::TextureLayout,
        value: crate::ClearValue,
    ) -> Result<(), crate::Error>
    where
        T: Borrow<crate::TextureSlice<'a>>,
    {
        raw::clear_texture(
            self.buffer,
            &self.device,
            texture,
            layout,
            value,
            &mut self.garbage,
        )
    }

    /// Only the base mip level of the slices will be used for the blit
    /// <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkCmdBlitImage.html>
    pub fn blit_textures<'a, T1, T2>(
        &mut self,
        src: T1,
        src_layout: crate::TextureLayout,
        dst: T2,
        dst_layout: crate::TextureLayout,
        filter: crate::FilterMode,
    ) -> Result<(), crate::Error>
    where
        T1: Borrow<crate::TextureSlice<'a>>,
        T2: Borrow<crate::TextureSlice<'a>>,
    {
        raw::blit_textures(
            self.buffer,
            &self.device,
            src.borrow(),
            src_layout,
            dst.borrow(),
            dst_layout,
            filter,
            &mut self.garbage,
        )
    }

    /// <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkBufferImageCopy.html>
    pub fn copy_buffer_to_buffer<'a, B1, B2>(
        &mut self,
        src: B1,
        dst: B2,
    ) -> Result<(), crate::Error>
    where
        B1: Borrow<crate::BufferSlice<'a>>,
        B2: Borrow<crate::BufferSlice<'a>>,
    {
        raw::copy_buffer_to_buffer(self.buffer, &self.device, src, dst, &mut self.garbage)
    }

    /// <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkBufferImageCopy.html>
    pub fn copy_texture_to_buffer<'a, T, B>(
        &mut self,
        src: T,
        src_layout: crate::TextureLayout,
        dst: B,
    ) -> Result<(), crate::Error>
    where
        T: Borrow<crate::TextureSlice<'a>>,
        B: Borrow<crate::BufferSlice<'a>>,
    {
        raw::copy_texture_to_buffer(
            self.buffer,
            &self.device,
            src,
            src_layout,
            dst,
            &mut self.garbage,
        )
    }

    /// <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkBufferImageCopy.html>
    pub fn copy_buffer_to_texture<'a, T, B>(
        &mut self,
        src: B,
        dst: T,
        dst_layout: crate::TextureLayout,
    ) -> Result<(), crate::Error>
    where
        B: Borrow<crate::BufferSlice<'a>>,
        T: Borrow<crate::TextureSlice<'a>>,
    {
        raw::copy_buffer_to_texture(
            self.buffer,
            &self.device,
            src,
            dst,
            dst_layout,
            &mut self.garbage,
        )
    }

    /// <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkCmdCopyImage.html>
    pub fn copy_texture_to_texture<'a, T1, T2>(
        &mut self,
        src: T1,
        src_layout: crate::TextureLayout,
        dst: T2,
        dst_layout: crate::TextureLayout,
    ) -> Result<(), crate::Error>
    where
        T1: Borrow<crate::TextureSlice<'a>>,
        T2: Borrow<crate::TextureSlice<'a>>,
    {
        raw::copy_texture_to_texture(
            self.buffer,
            &self.device,
            src,
            src_layout,
            dst,
            dst_layout,
            &mut self.garbage,
        )
    }

    /// <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkCmdResolveImage.html>
    pub fn resolve_texture<'a, T1, T2>(
        &mut self,
        src: T1,
        src_layout: crate::TextureLayout,
        dst: T2,
        dst_layout: crate::TextureLayout,
    ) -> Result<(), crate::Error>
    where
        T1: Borrow<crate::TextureSlice<'a>>,
        T2: Borrow<crate::TextureSlice<'a>>,
    {
        raw::resolve_texture(
            self.buffer,
            &self.device,
            src,
            src_layout,
            dst,
            dst_layout,
            &mut self.garbage,
        )
    }

    /// Begin and end a render pass without doing anything in the pass, to draw use a graphics pass and pipeline
    /// <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkCmdBeginRenderPass.html>
    pub fn empty_pass<'a, B>(
        &mut self,
        color_attachments: &[B],
        resolve_attachments: &[B],
        depth_attachment: Option<B>,
        render_pass: &crate::RenderPass,
    ) -> Result<(), crate::Error>
    where
        B: std::borrow::Borrow<crate::Attachment<'a>>,
    {
        if let Some(swapchain) = raw::begin_render_pass(
            self.buffer,
            &self.device,
            color_attachments,
            resolve_attachments,
            depth_attachment,
            render_pass,
            &mut self.garbage,
        )? {
            self.swapchain = Some(swapchain)
        }

        raw::end_render_pass(self.buffer, &self.device)
    }

    /// <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkCmdBeginRenderPass.html>
    /// <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkCmdBindPipeline.html>
    pub fn begin_graphics_pass<'a, B>(
        &mut self,
        color_attachments: &[B],
        resolve_attachments: &[B],
        depth_attachment: Option<B>,
        pipeline: &crate::GraphicsPipeline,
    ) -> Result<(), crate::Error>
    where
        B: std::borrow::Borrow<crate::Attachment<'a>>,
    {
        if let Some(swapchain) = raw::begin_graphics_pass(
            self.buffer,
            &self.device,
            color_attachments,
            resolve_attachments,
            depth_attachment,
            pipeline,
            &mut self.garbage,
        )? {
            self.swapchain = Some(swapchain)
        }

        Ok(())
    }

    /// <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkCmdEndRenderPass.html>
    pub fn end_graphics_pass(&mut self) -> Result<(), crate::Error> {
        raw::end_render_pass(self.buffer, &self.device)
    }

    /// <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkCmdDraw.html>
    pub fn draw(
        &mut self,
        first_vertex: u32,
        vertex_count: u32,
        first_instance: u32,
        instance_count: u32,
    ) -> Result<(), crate::Error> {
        raw::draw(
            self.buffer,
            &self.device,
            first_vertex,
            vertex_count,
            first_instance,
            instance_count,
        )
    }

    /// <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkCmdDrawIndexed.html>
    pub fn draw_indexed(
        &mut self,
        first_index: u32,
        index_count: u32,
        first_instance: u32,
        instance_count: u32,
        vertex_offset: i32,
    ) -> Result<(), crate::Error> {
        raw::draw_indexed(
            self.buffer,
            &self.device,
            first_index,
            index_count,
            first_instance,
            instance_count,
            vertex_offset,
        )
    }

    /// <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkCmdBindIndexBuffer.html>
    pub fn bind_index_buffer<'a, B>(
        &mut self,
        buffer: B,
        ty: crate::IndexType,
    ) -> Result<(), crate::Error>
    where
        B: Borrow<crate::BufferSlice<'a>>,
    {
        raw::bind_index_buffer(self.buffer, &self.device, buffer, ty, &mut self.garbage)
    }

    /// <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkCmdBindVertexBuffers.html>
    pub fn bind_vertex_buffer<'a, B>(&mut self, buffer: B, binding: u32) -> Result<(), crate::Error>
    where
        B: Borrow<crate::BufferSlice<'a>>,
    {
        raw::bind_vertex_buffers(
            self.buffer,
            &self.device,
            &[buffer],
            binding,
            &mut self.garbage,
        )
    }

    /// <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkCmdBindVertexBuffers.html>
    pub fn bind_vertex_buffers<'a, B>(
        &mut self,
        buffers: &[B],
        first_binding: u32,
    ) -> Result<(), crate::Error>
    where
        B: Borrow<crate::BufferSlice<'a>>,
    {
        raw::bind_vertex_buffers(
            self.buffer,
            &self.device,
            buffers,
            first_binding,
            &mut self.garbage,
        )
    }

    /// <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkCmdBindDescriptorSets.html>
    pub fn bind_descriptor<G>(
        &mut self,
        location: u32,
        group: G,
        bind_point: crate::PipelineBindPoint,
        layout: &crate::PipelineLayout,
    ) -> Result<(), crate::Error>
    where
        G: Borrow<crate::DescriptorSet>,
    {
        raw::bind_descriptors(
            self.buffer,
            &self.device,
            location,
            &[group],
            bind_point,
            layout,
            &mut self.garbage,
        )
    }

    /// <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkCmdBindDescriptorSets.html>
    pub fn bind_descriptors<G>(
        &mut self,
        first_location: u32,
        groups: &[G],
        bind_point: crate::PipelineBindPoint,
        layout: &crate::PipelineLayout,
    ) -> Result<(), crate::Error>
    where
        G: Borrow<crate::DescriptorSet>,
    {
        raw::bind_descriptors(
            self.buffer,
            &self.device,
            first_location,
            groups,
            bind_point,
            layout,
            &mut self.garbage,
        )
    }

    /// <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkCmdPushConstants.html>
    pub fn push_constants(
        &mut self,
        offset: u32,
        constants: &[u8],
        stages: crate::ShaderStages,
        layout: &crate::PipelineLayout,
    ) -> Result<(), crate::Error> {
        raw::push_constants(self.buffer, &self.device, offset, constants, stages, layout)
    }

    /// <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkCmdBindPipeline.html>
    pub fn begin_compute_pass(
        &mut self,
        pipeline: &crate::ComputePipeline,
    ) -> Result<(), crate::Error> {
        raw::begin_compute_pass(self.buffer, &self.device, pipeline, &mut self.garbage)
    }

    /// <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkCmdBindPipeline.html>
    pub fn dispatch(&mut self, x: u32, y: u32, z: u32) -> Result<(), crate::Error> {
        raw::dispatch(self.buffer, &self.device, x, y, z)
    }

    /// <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkCmdWriteTimestamp.html>
    pub fn write_timestamp(
        &mut self,
        query: &crate::TimeQuery,
        pipeline_stage: crate::PipelineStage,
        index: u32,
    ) -> Result<(), crate::Error> {
        raw::write_timestamp(
            self.buffer,
            &self.device,
            query,
            pipeline_stage,
            index,
            &mut self.garbage,
        )
    }

    /// <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkResetQueryPool.html>
    pub fn reset_time_query(
        &mut self,
        query: &crate::TimeQuery,
        first_query: u32,
        query_count: u32,
    ) -> Result<(), crate::Error> {
        raw::reset_time_query(
            self.buffer,
            &self.device,
            query,
            first_query,
            query_count,
            &mut self.garbage,
        )
    }
}

impl Drop for CommandBuffer {
    fn drop(&mut self) {
        unsafe {
            self.wait(!0).unwrap();
            self.garbage.clean(&self.device);

            self.device.destroy_command_pool(self.pool, None);
            let semaphore = Md::take(&mut self.semaphore);
            if let Ok(semaphore) = Arc::try_unwrap(semaphore) {
                self.device.destroy_semaphore(semaphore, None);
            }
            self.device.destroy_fence(self.fence, None);
        }
    }
}
