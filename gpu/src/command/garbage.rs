
use std::sync::Arc;

use ash::vk;

pub(crate) struct Garbage {
    pub memory: Vec<Arc<vk::DeviceMemory>>,
    pub textures: Vec<Arc<vk::Image>>,
    pub views: Vec<Arc<vk::ImageView>>,
    pub buffers: Vec<Arc<vk::Buffer>>,
    pub samplers: Vec<Arc<vk::Sampler>>,
    pub descriptor_layouts: Vec<Arc<vk::DescriptorSetLayout>>,
    pub descriptor_pools: Vec<Arc<vk::DescriptorPool>>,
    pub pipeline_layouts: Vec<Arc<vk::PipelineLayout>>,
    pub render_passes: Vec<Arc<vk::RenderPass>>,
    pub pipelines: Vec<Arc<vk::Pipeline>>,
    pub framebuffers: Vec<Arc<vk::Framebuffer>>,
    pub swapchains: Vec<crate::SwapchainInner>,
    pub queries: Vec<Arc<vk::QueryPool>>,
    pub prev_semaphore: Option<Arc<vk::Semaphore>>,
}

impl std::default::Default for Garbage {
    fn default() -> Self {
        Self { 
            memory: Vec::new(),
            textures: Vec::new(), 
            views: Vec::new(), 
            buffers: Vec::new(), 
            samplers: Vec::new(),
            descriptor_layouts: Vec::new(),
            descriptor_pools: Vec::new(), 
            pipeline_layouts: Vec::new(), 
            render_passes: Vec::new(), 
            pipelines: Vec::new(),
            framebuffers: Vec::new(),
            swapchains: Vec::new(),
            queries: Vec::new(),
            prev_semaphore: None,
        }
    }
}

impl Garbage {
    pub unsafe fn clean(&mut self, device: &crate::RawDevice) {
        for mem in self.memory.drain(..) {
            if let Ok(mem) = Arc::try_unwrap(mem) {
                device.free_memory(mem, None);
            }
        }

        for tex in self.textures.drain(..) {
            if let Ok(tex) = Arc::try_unwrap(tex) {
                device.destroy_image(tex, None);
            }
        }

        for view in self.views.drain(..) {
            if let Ok(view) = Arc::try_unwrap(view) {
                device.destroy_image_view(view, None);
            }
        }

        for buffer in self.buffers.drain(..) {
            if let Ok(buffer) = Arc::try_unwrap(buffer) {
                device.destroy_buffer(buffer, None);
            }
        }

        for sampler in self.samplers.drain(..) {
            if let Ok(sampler) = Arc::try_unwrap(sampler) {
                device.destroy_sampler(sampler, None);
            }
        }

        for descriptor_layout in self.descriptor_layouts.drain(..) {
            if let Ok(descriptor_layout) = Arc::try_unwrap(descriptor_layout) {
                device.destroy_descriptor_set_layout(descriptor_layout, None);
            }
        }

        for descriptor_pool in self.descriptor_pools.drain(..) {
            if let Ok(descriptor_pool) = Arc::try_unwrap(descriptor_pool) {
                device.destroy_descriptor_pool(descriptor_pool, None);
            }
        }

        for pipeline_layout in self.pipeline_layouts.drain(..) {
            if let Ok(pipeline_layout) = Arc::try_unwrap(pipeline_layout) {
                device.destroy_pipeline_layout(pipeline_layout, None);
            }
        }

        for render_pass in self.render_passes.drain(..) {
            if let Ok(render_pass) = Arc::try_unwrap(render_pass) {
                device.destroy_render_pass(render_pass, None);
            }
        }

        for pipeline in self.pipelines.drain(..) {
            if let Ok(pipeline) = Arc::try_unwrap(pipeline) {
                device.destroy_pipeline(pipeline, None);
            }
        }

        for framebuffer in self.framebuffers.drain(..) {
            if let Ok(framebuffer) = Arc::try_unwrap(framebuffer) {
                device.destroy_framebuffer(framebuffer, None);
            }
        }

        for swapchain in self.swapchains.drain(..) {
            drop(swapchain);
        }

        for pool in self.queries.drain(..) {
            if let Ok(pool) = Arc::try_unwrap(pool) {
                device.destroy_query_pool(pool, None);
            }
        }

        if let Some(prev_semaphore) = self.prev_semaphore.take() {
            if let Ok(semaphore) = Arc::try_unwrap(prev_semaphore) {
                device.destroy_semaphore(semaphore, None);
            }
        }
    }
}