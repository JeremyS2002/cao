
use std::mem::ManuallyDrop as Md;
use std::sync::Arc;
use std::ptr;

use ash::vk;

#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct FramebufferKey {
    pub attachments: Vec<vk::ImageView>,
    pub render_pass: vk::RenderPass,
}

/// Describes a RenderPass
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RenderPassDesc<'a> {
    /// name of the render pass
    pub name: Option<String>,
    /// Color attachment descriptions
    pub colors: &'a [crate::ColorAttachmentDesc],
    /// Resolve attachment descriptions
    pub resolves: &'a [crate::ResolveAttachmentDesc],
    /// Depth attachment description
    pub depth: Option<crate::DepthAttachmentDesc>,
    /// number of samples in the renderpass
    pub samples: crate::Samples,
}

/// RenderPass
///
/// <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkRenderPass.html>
pub struct RenderPass {
    pub(crate) raw: Md<Arc<vk::RenderPass>>,
    pub(crate) name: Option<String>,
    pub(crate) device: Arc<crate::RawDevice>,
    pub(crate) samples: crate::Samples,
    pub(crate) colors: Arc<[crate::ColorAttachmentDesc]>,
    pub(crate) resolves: Arc<[crate::ResolveAttachmentDesc]>,
    pub(crate) depth: Option<crate::DepthAttachmentDesc>,
}

impl std::hash::Hash for RenderPass {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (**self.raw).hash(state)
    }
}

impl PartialEq for RenderPass {
    fn eq(&self, other: &RenderPass) -> bool {
        **self.raw == **other.raw
    }
}

impl Eq for RenderPass {}

impl std::clone::Clone for RenderPass {
    fn clone(&self) -> Self {
        Self {
            raw: Md::new(Arc::clone(&self.raw)),
            name: self.name.clone(),
            device: Arc::clone(&self.device),
            samples: self.samples,
            colors: Arc::clone(&self.colors),
            resolves: Arc::clone(&self.resolves),
            depth: self.depth.clone(),
        }
    }
}

impl Drop for RenderPass {
    fn drop(&mut self) {
        unsafe {
            let raw = Md::take(&mut self.raw);
            if let Ok(raw) = Arc::try_unwrap(raw) {
                self.device.destroy_render_pass(raw, None);
            }
        }
    }
}

impl std::fmt::Debug for RenderPass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RenderPass {:?}, name: {:?}", **self.raw, self.name)
    }
}

impl RenderPass {
    pub fn raw_render_pass(&self) -> vk::RenderPass {
        **self.raw
    }
}

impl RenderPass {
    /// Create a new RenderPass
    pub fn new(device: &crate::Device, desc: &RenderPassDesc<'_>) -> Result<Self, crate::Error> {
        let mut attachments = desc
            .colors
            .iter()
            .map(|a| vk::AttachmentDescription {
                flags: vk::AttachmentDescriptionFlags::empty(),
                format: a.format.into(),
                samples: desc.samples.into(),
                load_op: a.load.into(),
                store_op: a.store.into(),
                stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
                stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
                initial_layout: a.initial_layout.into(),
                final_layout: a.final_layout.into(),
            })
            .collect::<Vec<_>>();
        attachments.append(
            &mut desc
                .resolves
                .as_ref()
                .iter()
                .zip(desc.colors.iter())
                .map(|(r, c)| vk::AttachmentDescription {
                    flags: vk::AttachmentDescriptionFlags::empty(),
                    format: c.format.into(),
                    samples: vk::SampleCountFlags::TYPE_1,
                    load_op: r.load.into(),
                    store_op: r.store.into(),
                    stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
                    stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
                    initial_layout: r.initial_layout.into(),
                    final_layout: r.final_layout.into(),
                })
                .collect::<Vec<_>>(),
        );

        if let Some(d) = desc.depth {
            attachments.push(vk::AttachmentDescription {
                flags: vk::AttachmentDescriptionFlags::empty(),
                format: d.format.into(),
                samples: desc.samples.into(),
                load_op: d.load.into(),
                store_op: d.store.into(),
                stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
                stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
                initial_layout: d.initial_layout.into(),
                final_layout: d.final_layout.into(),
            })
        }

        let color_refs = desc
            .colors
            .iter()
            .enumerate()
            .map(|(i, _)| vk::AttachmentReference {
                attachment: i as u32,
                layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            })
            .collect::<Vec<_>>();
        let resolve_start = color_refs.len() as u32;
        let resolve_refs = desc
            .resolves
            .iter()
            .enumerate()
            .map(|(i, _)| vk::AttachmentReference {
                attachment: resolve_start + i as u32,
                layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            })
            .collect::<Vec<_>>();
        let depth_index = resolve_start + resolve_refs.len() as u32;
        let depth_ref = desc.depth.as_ref().map(|_| vk::AttachmentReference {
            attachment: depth_index,
            layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        });
        let p_depth_stencil_attachment = if let Some(d) = &depth_ref {
            d
        } else {
            ptr::null()
        };

        let subpass = vk::SubpassDescription {
            flags: vk::SubpassDescriptionFlags::empty(),
            pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
            input_attachment_count: 0,
            p_input_attachments: ptr::null(),
            color_attachment_count: color_refs.len() as u32,
            p_color_attachments: color_refs.as_ptr(),
            p_resolve_attachments: if resolve_refs.len() != 0 {
                resolve_refs.as_ptr()
            } else {
                ptr::null()
            },
            p_depth_stencil_attachment,
            preserve_attachment_count: 0,
            p_preserve_attachments: ptr::null(),
        };

        let dependency = vk::SubpassDependency {
            src_subpass: vk::SUBPASS_EXTERNAL,
            dst_subpass: 0,
            src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            src_access_mask: vk::AccessFlags::empty(),
            dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE
                | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            dependency_flags: vk::DependencyFlags::empty(),
        };

        let create_info = vk::RenderPassCreateInfo {
            s_type: vk::StructureType::RENDER_PASS_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::RenderPassCreateFlags::empty(),
            attachment_count: attachments.len() as u32,
            p_attachments: attachments.as_ptr(),
            subpass_count: 1,
            p_subpasses: &subpass,
            dependency_count: 1,
            p_dependencies: &dependency,
        };

        let pass_result = unsafe { device.raw.create_render_pass(&create_info, None) };

        let p = match pass_result {
            Ok(p) => p,
            Err(e) => return Err(crate::ExplicitError(e).into()),
        };

        let s = Self {
            raw: Md::new(Arc::new(p)),
            device: Arc::clone(&device.raw),
            name: desc.name.as_ref().map(|n| n.to_string()),
            samples: desc.samples,
            colors: desc.colors.to_vec().into(),
            resolves: desc.resolves.to_vec().into(),
            depth: desc.depth.clone(),
        };

        if let Some(name) = &desc.name {
            device.raw.set_render_pass_name(&s, name.as_ref())?;
        }

        device.raw.check_errors()?;

        Ok(s)
    }

    /// Get the number of samples in the pass
    pub fn samples(&self) -> crate::Samples {
        self.samples
    }

    /// Get the color attachments
    pub fn colors<'a>(&'a self) -> &'a [crate::ColorAttachmentDesc] {
        &self.colors
    }

    /// Get the resolve attachments
    pub fn resolves<'a>(&'a self) -> &'a [crate::ResolveAttachmentDesc] {
        &self.resolves
    }

    /// Get the depth attachment if any
    pub fn depth(&self) -> Option<crate::DepthAttachmentDesc> {
        self.depth.clone()
    }
}