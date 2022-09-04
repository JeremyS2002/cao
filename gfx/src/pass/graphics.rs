//! GraphicsPass + Implementors

use std::borrow::Cow;
use std::collections::HashMap;
use std::collections::HashSet;
use std::mem::ManuallyDrop as Md;

#[cfg(feature = "reflect")]
use crate::reflect::Bundle;
#[cfg(feature = "reflect")]
use std::any::TypeId;
#[cfg(feature = "reflect")]
use std::marker::PhantomData;

// #[cfg(feature = "reflect")]
// use crate::prelude::*;

/// Represents valid commands to perform while in a graphics pass
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum GraphicsPassCommand<'a> {
    Draw {
        first_vertex: u32,
        vertex_count: u32,
        first_instance: u32,
        instance_count: u32,
    },
    DrawIndexed {
        first_index: u32,
        index_count: u32,
        first_instance: u32,
        instance_count: u32,
        vertex_offset: i32,
    },
    BindVertexBuffer {
        buffer: gpu::BufferSlice<'a>,
        binding: u32,
    },
    BindVertexBuffers {
        buffers: Cow<'a, [gpu::BufferSlice<'a>]>,
        first_binding: u32,
    },
    BindIndexBuffer {
        buffer: gpu::BufferSlice<'a>,
        ty: gpu::IndexType,
    },
    BindDescriptorSets {
        descriptors: Cow<'a, [Cow<'a, gpu::DescriptorSet>]>,
        first_location: u32,
    },
    BindDescriptorSet {
        descriptor: Cow<'a, gpu::DescriptorSet>,
        location: u32,
    },
    PushConstants {
        offset: u32,
        constants: Vec<u8>,
        stages: gpu::ShaderStages,
    },
}

impl<'a> GraphicsPassCommand<'a> {
    /// execute the command represented by self a command recorder
    pub fn execute(
        &self,
        command_buffer: &mut gpu::CommandBuffer,
        layout: &gpu::PipelineLayout,
    ) -> Result<(), gpu::Error> {
        match self {
            GraphicsPassCommand::Draw {
                first_instance,
                first_vertex,
                instance_count,
                vertex_count,
            } => command_buffer.draw(
                *first_vertex,
                *vertex_count,
                *first_instance,
                *instance_count,
            ),
            GraphicsPassCommand::DrawIndexed {
                first_instance,
                first_index,
                instance_count,
                index_count,
                vertex_offset,
            } => command_buffer.draw_indexed(
                *first_index,
                *index_count,
                *first_instance,
                *instance_count,
                *vertex_offset,
            ),
            GraphicsPassCommand::BindVertexBuffers {
                buffers,
                first_binding,
            } => command_buffer.bind_vertex_buffers(buffers, *first_binding),
            GraphicsPassCommand::BindVertexBuffer { buffer, binding } => {
                command_buffer.bind_vertex_buffer(buffer, *binding)
            }
            GraphicsPassCommand::BindIndexBuffer { buffer, ty } => {
                command_buffer.bind_index_buffer(buffer, *ty)
            }
            GraphicsPassCommand::BindDescriptorSets {
                descriptors,
                first_location,
            } => command_buffer.bind_descriptors(
                *first_location,
                descriptors,
                gpu::PipelineBindPoint::Graphics,
                layout,
            ),
            GraphicsPassCommand::BindDescriptorSet {
                descriptor,
                location,
            } => command_buffer.bind_descriptor(
                *location,
                descriptor.as_ref(),
                gpu::PipelineBindPoint::Graphics,
                layout,
            ),
            GraphicsPassCommand::PushConstants {
                offset,
                constants,
                stages,
            } => command_buffer.push_constants(*offset, constants, *stages, layout),
        }
    }

    /// Get all the buffers referenced by the command represented by self
    pub fn buffers(&self) -> HashSet<gpu::BufferSlice<'a>> {
        let mut result = HashSet::new();
        match self {
            GraphicsPassCommand::BindDescriptorSet { descriptor, .. } => {
                for buffer in descriptor.buffers() {
                    result.insert(buffer.clone());
                }
            }
            GraphicsPassCommand::BindDescriptorSets { descriptors, .. } => {
                for descriptor in descriptors.as_ref() {
                    for buffer in descriptor.buffers() {
                        result.insert(buffer.clone());
                    }
                }
            }
            GraphicsPassCommand::BindIndexBuffer { buffer, .. } => {
                result.insert(buffer.clone());
            }
            GraphicsPassCommand::BindVertexBuffer { buffer, .. } => {
                result.insert(buffer.clone());
            }
            GraphicsPassCommand::BindVertexBuffers { buffers, .. } => {
                for buffer in buffers.as_ref() {
                    result.insert(buffer.clone());
                }
            }
            _ => (),
        }
        result
    }

    /// Get all the textures referenced by the command represented by self and the layout they should be in
    pub fn textures(&self) -> HashMap<(gpu::Texture, u32, u32), gpu::TextureLayout> {
        let mut result = HashMap::new();
        match self {
            GraphicsPassCommand::BindDescriptorSet { descriptor, .. } => {
                for (texture, layout) in descriptor.textures() {
                    for i in
                        texture.base_mip_level()..(texture.base_mip_level() + texture.mip_levels())
                    {
                        for j in texture.base_array_layer()
                            ..(texture.base_array_layer() + texture.array_layers())
                        {
                            if let Some(l) =
                                result.insert((texture.texture().clone(), i, j), *layout)
                            {
                                if *layout != l {
                                    panic!("ERROR: GraphicsPassCommand::BindDescriptorSet uses descriptor with texture using different layouts {:?}, {:?}", *layout, l);
                                }
                            }
                        }
                    }
                }
            }
            GraphicsPassCommand::BindDescriptorSets { descriptors, .. } => {
                for descriptor in descriptors.as_ref() {
                    for (texture, layout) in descriptor.textures() {
                        for i in texture.base_mip_level()
                            ..(texture.base_mip_level() + texture.mip_levels())
                        {
                            for j in texture.base_array_layer()
                                ..(texture.base_array_layer() + texture.array_layers())
                            {
                                if let Some(l) =
                                    result.insert((texture.texture().clone(), i, j), *layout)
                                {
                                    if *layout != l {
                                        panic!("ERROR: GraphicsPassCommand::BindDescriptorSet uses descriptor with texture using different layouts {:?}, {:?}", *layout, l);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            _ => (),
        }
        result
    }
}

/// represents and object that can record graphics pass commands
pub trait GraphicsPass<'a> {
    /// push a command to the queue
    fn push_command(&mut self, command: GraphicsPassCommand<'a>);

    /// draw from a vertex buffer
    ///
    /// # Safety
    ///
    /// The draw indices must be in range of the vertex buffer size
    /// and the bind descriptors used by the pipeline must be set
    fn draw(
        &mut self,
        first_vertex: u32,
        vertex_count: u32,
        first_instance: u32,
        instance_count: u32,
    ) {
        self.push_command(GraphicsPassCommand::Draw {
            first_vertex,
            vertex_count,
            first_instance,
            instance_count,
        })
    }

    /// draw from a vertex buffer
    ///
    /// # Safety
    ///
    /// The draw indices must be in range of the vertex buffer size
    /// and the bind descriptors used by the pipeline must be set
    fn draw_indexed(
        &mut self,
        first_index: u32,
        index_count: u32,
        first_instance: u32,
        instance_count: u32,
        vertex_offset: i32,
    ) {
        self.push_command(GraphicsPassCommand::DrawIndexed {
            first_index,
            index_count,
            first_instance,
            instance_count,
            vertex_offset,
        })
    }

    /// bind an index buffer
    fn bind_index_buffer(&mut self, buffer: gpu::BufferSlice<'a>, ty: gpu::IndexType) {
        if !buffer.buffer().usage().contains(gpu::BufferUsage::INDEX) {
            panic!("ERROR: Buffer {:?} missing usage INDEX", buffer.buffer())
        }
        self.push_command(GraphicsPassCommand::BindIndexBuffer { buffer, ty })
    }

    /// bind a single vertex buffer
    fn bind_vertex_buffer(&mut self, buffer: gpu::BufferSlice<'a>, binding: u32) {
        if !buffer.buffer().usage().contains(gpu::BufferUsage::VERTEX) {
            panic!("ERROR: Buffer {:?} Missing usage VERTEX", buffer.buffer())
        }
        self.push_command(GraphicsPassCommand::BindVertexBuffer { buffer, binding })
    }

    /// bind the buffers to vertex usage
    fn bind_vertex_buffers_ref(&mut self, buffers: &'a [gpu::BufferSlice<'a>], first_binding: u32) {
        for buffer in buffers {
            if !buffer.buffer().usage().contains(gpu::BufferUsage::VERTEX) {
                panic!("ERROR: Buffer {:?} Missing usage VERTEX", buffer.buffer())
            }
        }

        self.push_command(GraphicsPassCommand::BindVertexBuffers {
            buffers: Cow::Borrowed(buffers),
            first_binding,
        })
    }

    /// bind the buffers to vertex usage
    fn bind_vertex_buffers_owned(
        &mut self,
        buffers: Vec<gpu::BufferSlice<'a>>,
        first_binding: u32,
    ) {
        for buffer in &buffers {
            if !buffer.buffer().usage().contains(gpu::BufferUsage::VERTEX) {
                panic!("ERROR: Buffer {:?} Missing usage VERTEX", buffer.buffer())
            }
        }

        self.push_command(GraphicsPassCommand::BindVertexBuffers {
            buffers: Cow::from(buffers),
            first_binding,
        })
    }

    /// set a single bind descriptor
    ///
    /// # Safety
    ///
    /// The bind descriptor being set must match the pipeline
    fn bind_descriptor_ref(&mut self, location: u32, descriptor: &'a gpu::DescriptorSet) {
        self.push_command(GraphicsPassCommand::BindDescriptorSet {
            location,
            descriptor: Cow::Borrowed(descriptor),
        })
    }

    /// set a single bind descriptor
    ///
    /// # Safety
    ///
    /// The bind descriptor being set must match the pipeline
    fn bind_descriptor_owned(&mut self, location: u32, descriptor: gpu::DescriptorSet) {
        self.push_command(GraphicsPassCommand::BindDescriptorSet {
            location,
            descriptor: Cow::Owned(descriptor),
        })
    }

    /// set the bind descriptors
    ///
    /// # Safety
    ///
    /// The bind descriptor being set must match the pipeline
    fn bind_descriptors_ref(
        &mut self,
        first_location: u32,
        descriptors: &[&'a gpu::DescriptorSet],
    ) {
        let descriptors = descriptors
            .iter()
            .map(|&g| Cow::Borrowed(g))
            .collect::<Vec<_>>();
        self.push_command(GraphicsPassCommand::BindDescriptorSets {
            first_location,
            descriptors: Cow::from(descriptors),
        })
    }

    /// set the bind descriptors
    ///
    /// # Safety
    ///
    /// The bind descriptor being set must match the pipeline
    fn bind_descriptors_owned(
        &mut self,
        first_location: u32,
        descriptors: Vec<gpu::DescriptorSet>,
    ) {
        let descriptors = descriptors
            .into_iter()
            .map(|g| Cow::Owned(g))
            .collect::<Vec<_>>();
        self.push_command(GraphicsPassCommand::BindDescriptorSets {
            first_location,
            descriptors: Cow::from(descriptors),
        })
    }

    /// push constants
    fn push_constants(&mut self, offset: u32, constants: &[u8], stages: gpu::ShaderStages) {
        self.push_command(GraphicsPassCommand::PushConstants {
            offset,
            constants: Vec::from(constants),
            stages,
        })
    }
}

/// A GraphicsPass
///
/// Load the attachments and then call functions to operate of the attachments
/// <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkRenderPass.html>
/// <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkFramebuffer.html>
pub struct BasicGraphicsPass<'a, 'b> {
    pub(crate) pipeline: Md<Cow<'a, gpu::GraphicsPipeline>>,
    pub(crate) color_attachments: Cow<'a, [gpu::Attachment<'a>]>,
    pub(crate) resolve_attachments: Cow<'a, [gpu::Attachment<'a>]>,
    pub(crate) depth_attachment: Option<gpu::Attachment<'a>>,
    pub(crate) commands: Vec<GraphicsPassCommand<'a>>,
    /// The encoder that the graphics pass will be recorded into
    pub encoder: &'b mut crate::CommandEncoder<'a>,
}

impl std::fmt::Debug for BasicGraphicsPass<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "BasicGraphicsPass")
    }
}

impl<'a, 'b> GraphicsPass<'a> for BasicGraphicsPass<'a, 'b> {
    fn push_command(&mut self, command: GraphicsPassCommand<'a>) {
        self.commands.push(command)
    }
}

impl<'a> BasicGraphicsPass<'a, '_> {
    /// End the graphics pass by dropping it and allowing the encoder to be used again
    pub fn finish(self) {}
}

impl<'a, 'b> Drop for BasicGraphicsPass<'a, 'b> {
    fn drop(&mut self) {
        self.encoder
            .push_command(crate::encoder::Command::GraphicsPass {
                pipeline: unsafe { Md::take(&mut self.pipeline) },
                color_attachments: self.color_attachments.clone(),
                resolve_attachments: self.resolve_attachments.clone(),
                depth_attachment: self.depth_attachment.take(),
                commands: self.commands.drain(..).collect(),
            })
    }
}

/// A GraphicsPass with a marked vertex to allow for inference of types
///
/// Load the attachments and then call functions to operate of the attachments
/// <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkRenderPass.html>
/// <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkFramebuffer.html>
#[cfg(feature = "reflect")]
pub struct ReflectedGraphicsPass<'a, 'b, V: crate::Vertex> {
    pub(crate) parent_id: u64,
    pub(crate) bundle_needed: bool,
    /// Pipeline contained inside a manually drop so that it can be taken an moved into the encoder
    pub(crate) pipeline: Md<Cow<'a, gpu::GraphicsPipeline>>,
    pub(crate) color_attachments: Vec<crate::Attachment<'a>>,
    pub(crate) resolve_attachments: Vec<crate::Attachment<'a>>,
    pub(crate) depth_attachment: Option<crate::Attachment<'a>>,
    pub(crate) push_constant_names: Option<HashMap<String, (u32, gpu::ShaderStages, TypeId)>>,
    pub(crate) commands: Vec<GraphicsPassCommand<'a>>,
    /// The encoder that the graphics pass will be recorded into
    pub encoder: &'b mut crate::CommandEncoder<'a>,
    pub(crate) marker: PhantomData<V>,
}

#[cfg(feature = "reflect")]
impl<V: crate::Vertex> std::fmt::Debug for ReflectedGraphicsPass<'_, '_, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "ReflectedGraphicsPass parent id {}", self.parent_id)
    }
}

#[cfg(feature = "reflect")]
impl<'a, 'b, V: crate::Vertex> GraphicsPass<'a> for ReflectedGraphicsPass<'a, 'b, V> {
    fn push_command(&mut self, command: GraphicsPassCommand<'a>) {
        self.commands.push(command)
    }
}

#[cfg(feature = "reflect")]
impl<V: crate::Vertex> ReflectedGraphicsPass<'_, '_, V> {
    /// End the graphics pass by dropping it and allowing the encoder to be used again
    pub fn finish(self) {}
}

#[cfg(feature = "reflect")]
impl<'a, 'b, V: crate::Vertex> Drop for ReflectedGraphicsPass<'a, 'b, V> {
    fn drop(&mut self) {
        self.encoder
            .push_command(crate::encoder::Command::GraphicsPass {
                pipeline: unsafe { Md::take(&mut self.pipeline) },
                color_attachments: self.color_attachments.drain(..).map(|a| a.raw).collect(),
                resolve_attachments: self.resolve_attachments.drain(..).map(|a| a.raw).collect(),
                depth_attachment: self.depth_attachment.take().map(|a| a.raw),
                commands: self.commands.drain(..).collect(),
            });
    }
}

#[cfg(feature = "reflect")]
impl<'a, 'b, V: crate::Vertex> ReflectedGraphicsPass<'a, 'b, V> {
    /// Returns if the pass needs Descriptors to function
    pub fn bundle_needed(&self) -> bool {
        self.bundle_needed
    }

    /// Draw a mesh referencing the mesh's buffers
    pub fn draw_mesh_ref(&mut self, mesh: &'a crate::Mesh<V>) {
        mesh.draw_ref(self);
    }

    /// Draw a mesh cloning the mesh's buffers
    pub fn draw_mesh_owned(&mut self, mesh: crate::Mesh<V>) {
        mesh.draw_owned(self);
    }

    /// Draw a mesh referencing the mesh's buffers
    pub fn draw_instanced_mesh_ref(
        &mut self,
        mesh: &'a crate::Mesh<V>,
        first_instance: u32,
        instance_count: u32,
    ) {
        mesh.draw_instanced_ref(self, first_instance, instance_count);
    }

    /// Draw a mesh cloning the mesh's buffers
    pub fn draw_instanced_mesh_owned(
        &mut self,
        mesh: crate::Mesh<V>,
        first_instance: u32,
        instance_count: u32,
    ) {
        mesh.draw_instanced_owned(self, first_instance, instance_count);
    }

    /// Set a bundle referencing the bundle
    pub fn set_bundle_ref(&mut self, bundle: &'a Bundle) {
        #[cfg(feature = "logging")]
        if !self.bundle_needed {
            log::warn!(
                "GFX: Attempt to set bundle {:?} on pass without bundle needed",
                bundle
            )
        }
        #[cfg(feature = "logging")]
        if self.parent_id != bundle.parent_id {
            log::warn!(
                "GFX: Attempt to set bundle {:?} of different parent id than pass",
                bundle
            )
        }
        self.bind_descriptors_ref(0, &bundle.descriptor_sets.iter().collect::<Vec<_>>());
    }

    /// Set a bundle cloning the bundle data
    pub fn set_bundle_owned(&mut self, bundle: Bundle) {
        #[cfg(feature = "logging")]
        if !self.bundle_needed {
            log::warn!(
                "GFX: Attempt to set bundle {:?} on pass without bundle needed",
                bundle
            )
        }
        #[cfg(feature = "logging")]
        if self.parent_id != bundle.parent_id {
            log::warn!(
                "GFX: Attempt to set bundle {:?} of different parent id than pass",
                bundle
            )
        }
        self.bind_descriptors_owned(
            0,
            bundle.descriptor_sets.iter().cloned().collect::<Vec<_>>(),
        );
    }

    /// Push a single constant by variable name
    /// If there are no constants by the name no action will be taken
    /// If the type supplied is different to the type expected this will panic
    #[inline(always)]
    pub fn push_constant<T: bytemuck::Pod + std::fmt::Debug>(&mut self, name: &str, constant: T) {
        if let Some(map) = self.push_constant_names.as_ref() {
            if let Some(&(offset, stages, ty)) = map.get(name) {
                assert_eq!(
                    ty,
                    TypeId::of::<T>(),
                    "ERROR: Call to push_constant with different type of constant than in spirv"
                );
                self.push_constants(offset, bytemuck::bytes_of(&constant), stages)
            } else {
                #[cfg(feature = "logging")]
                log::error!("Call to push_constant at {} with value {:?}, with no rust type found for field, No action taken", name, constant);
            }
        } else {
            #[cfg(feature = "logging")]
            log::error!("Call to push_constant with at {} with value {:?}, when there are no push constants, No action taken", name, constant);
        }
    }
}

macro_rules! push {
    ($(($f:tt, $ty:tt),)*) => {
        impl<'a, 'b, V: crate::Vertex> ReflectedGraphicsPass<'a, 'b, V> {
            $(
                #[allow(missing_docs)]
                #[inline(always)]
                pub fn $f(&mut self, name: &str, constant: $ty) {
                    self.push_constant(name, constant);
                }
            )*
        }
    };
}

push!(
    (push_u8, u8),
    (push_u16, u16),
    (push_u32, u32),
    (push_u64, u64),
    (push_uvec2, [u32; 2]),
    (push_uvec3, [u32; 3]),
    (push_uvec4, [u32; 4]),
    (push_i8, i8),
    (push_i16, i16),
    (push_i32, i32),
    (push_i64, i64),
    (push_svec2, [i32; 2]),
    (push_svec3, [i32; 3]),
    (push_svec4, [i32; 4]),
    (push_f32, f32),
    (push_f64, f64),
    (push_vec2, [f32; 2]),
    (push_vec3, [f32; 3]),
    (push_vec4, [f32; 4]),
    (push_mat2, [f32; 4]),
    (push_mat3, [f32; 9]),
    (push_mat4, [f32; 16]),
    (push_dvec2, [f64; 2]),
    (push_dvec3, [f64; 3]),
    (push_dvec4, [f64; 4]),
    (push_dmat2, [f64; 4]),
    (push_dmat3, [f64; 9]),
    (push_dmat4, [f64; 16]),
);
