use std::borrow::Cow;
use std::collections::HashMap;
use std::collections::HashSet;
use std::mem::ManuallyDrop as Md;

use smallvec::SmallVec;

#[cfg(feature = "reflect")]
use crate::reflect::Bundle;
#[cfg(feature = "reflect")]
use std::any::TypeId;

/// Represents valid operations to perform during a compute pass
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum ComputePassCommand<'a> {
    BindDescriptorSet {
        descriptor: Cow<'a, gpu::DescriptorSet>,
        location: u32,
    },
    BindDescriptorSets {
        descriptors: Cow<'a, [Cow<'a, gpu::DescriptorSet>]>,
        first_location: u32,
    },
    Dispatch {
        x: u32,
        y: u32,
        z: u32,
    },
    PushConstants {
        offset: u32,
        constants: SmallVec<[u8; 64]>,
        stages: gpu::ShaderStages,
    },
}

impl<'a> ComputePassCommand<'a> {
    /// Execute the command on a CommandRecorder
    #[inline]
    pub fn execute(
        &self,
        command_buffer: &mut gpu::CommandBuffer,
        layout: &gpu::PipelineLayout,
    ) -> Result<(), gpu::Error> {
        match self {
            ComputePassCommand::BindDescriptorSet {
                descriptor,
                location,
            } => command_buffer.bind_descriptor(
                *location,
                descriptor.as_ref(),
                gpu::PipelineBindPoint::Compute,
                layout,
            ),
            ComputePassCommand::BindDescriptorSets {
                descriptors,
                first_location,
            } => command_buffer.bind_descriptors(
                *first_location,
                descriptors,
                gpu::PipelineBindPoint::Compute,
                layout,
            ),
            ComputePassCommand::Dispatch { x, y, z } => command_buffer.dispatch(*x, *y, *z),
            ComputePassCommand::PushConstants {
                offset,
                constants,
                stages,
            } => command_buffer.push_constants(*offset, constants, *stages, layout),
        }
    }

    /// Get all the buffers referenced by the command represented by self
    #[inline]
    pub fn buffers(&self) -> HashSet<gpu::BufferSlice<'a>> {
        let mut result = HashSet::new();
        match self {
            ComputePassCommand::BindDescriptorSet { descriptor, .. } => {
                for buffer in descriptor.buffers() {
                    result.insert(buffer.clone());
                }
            }
            ComputePassCommand::BindDescriptorSets { descriptors, .. } => {
                for descriptor in descriptors.as_ref() {
                    for buffer in descriptor.buffers() {
                        result.insert(buffer.clone());
                    }
                }
            }
            _ => (),
        }
        result
    }

    /// Get all the textures referenced by the command represented by self and the layout they should be in
    #[inline]
    pub fn textures(&self) -> HashMap<(Cow<'a, gpu::Texture>, u32, u32), gpu::TextureLayout> {
        let mut result = HashMap::new();
        match self {
            ComputePassCommand::BindDescriptorSet { descriptor, .. } => match descriptor {
                Cow::Borrowed(d) => {
                    for (texture, layout) in d.textures() {
                        for i in texture.base_mip_level()
                            ..(texture.base_mip_level() + texture.mip_levels())
                        {
                            for j in texture.base_array_layer()
                                ..(texture.base_array_layer() + texture.array_layers())
                            {
                                if let Some(l) =
                                    result.insert((Cow::Borrowed(texture.texture()), i, j), *layout)
                                {
                                    if *layout != l {
                                        panic!("ERROR: ComputePassCommand::BindDescriptorSet uses descriptor with texture using different layouts {:?}, {:?}", *layout, l);
                                    }
                                }
                            }
                        }
                    }
                }
                Cow::Owned(d) => {
                    for (texture, layout) in d.textures() {
                        for i in texture.base_mip_level()
                            ..(texture.base_mip_level() + texture.mip_levels())
                        {
                            for j in texture.base_array_layer()
                                ..(texture.base_array_layer() + texture.array_layers())
                            {
                                if let Some(l) = result
                                    .insert((Cow::Owned(texture.texture().clone()), i, j), *layout)
                                {
                                    if *layout != l {
                                        panic!("ERROR: ComputePassCommand::BindDescriptorSet uses descriptor with texture using different layouts {:?}, {:?}", *layout, l);
                                    }
                                }
                            }
                        }
                    }
                }
            },
            ComputePassCommand::BindDescriptorSets { descriptors, .. } => {
                for descriptor in descriptors.as_ref() {
                    match descriptor {
                        Cow::Borrowed(d) => {
                            for (texture, layout) in d.textures() {
                                for i in texture.base_mip_level()
                                    ..(texture.base_mip_level() + texture.mip_levels())
                                {
                                    for j in texture.base_array_layer()
                                        ..(texture.base_array_layer() + texture.array_layers())
                                    {
                                        if let Some(l) = result.insert(
                                            (Cow::Borrowed(texture.texture()), i, j),
                                            *layout,
                                        ) {
                                            if *layout != l {
                                                panic!("ERROR: ComputePassCommand::BindDescriptorSet uses descriptor with texture using different layouts {:?}, {:?}", *layout, l);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Cow::Owned(d) => {
                            for (texture, layout) in d.textures() {
                                for i in texture.base_mip_level()
                                    ..(texture.base_mip_level() + texture.mip_levels())
                                {
                                    for j in texture.base_array_layer()
                                        ..(texture.base_array_layer() + texture.array_layers())
                                    {
                                        if let Some(l) = result.insert(
                                            (Cow::Owned(texture.texture().clone()), i, j),
                                            *layout,
                                        ) {
                                            if *layout != l {
                                                panic!("ERROR: ComputePassCommand::BindDescriptorSet uses descriptor with texture using different layouts {:?}, {:?}", *layout, l);
                                            }
                                        }
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

/// Represents an object that records compute pass commands
pub trait ComputePass<'a> {
    /// Push a command to self's commands
    fn push_command(&mut self, command: ComputePassCommand<'a>);

    /// set a single bind descriptor
    fn bind_descriptor_ref(&mut self, location: u32, descriptor: &'a gpu::DescriptorSet) {
        self.push_command(ComputePassCommand::BindDescriptorSet {
            location,
            descriptor: Cow::Borrowed(descriptor),
        })
    }

    /// set a single bind descriptor
    fn bind_descriptor_owned(&mut self, location: u32, descriptor: gpu::DescriptorSet) {
        self.push_command(ComputePassCommand::BindDescriptorSet {
            location,
            descriptor: Cow::Owned(descriptor),
        })
    }

    /// set the bind descriptors
    fn bind_descriptors_ref(
        &mut self,
        first_location: u32,
        descriptors: &[&'a gpu::DescriptorSet],
    ) {
        let descriptors = descriptors
            .into_iter()
            .map(|g| Cow::Borrowed(*g))
            .collect::<Vec<_>>();
        self.push_command(ComputePassCommand::BindDescriptorSets {
            first_location,
            descriptors: Cow::Owned(descriptors),
        })
    }

    /// set the bind descriptors
    fn bind_descriptors_owned(
        &mut self,
        first_location: u32,
        descriptors: Vec<gpu::DescriptorSet>,
    ) {
        let descriptors = descriptors
            .into_iter()
            .map(|g| Cow::Owned(g))
            .collect::<Vec<_>>();
        self.push_command(ComputePassCommand::BindDescriptorSets {
            first_location,
            descriptors: Cow::Owned(descriptors),
        })
    }

    /// Dispatch a compute pipeline
    fn dispatch(&mut self, x: u32, y: u32, z: u32) {
        self.push_command(ComputePassCommand::Dispatch { x, y, z });
    }

    /// push constants
    fn push_constants(&mut self, offset: u32, constants: &[u8], stages: gpu::ShaderStages) {
        self.push_command(ComputePassCommand::PushConstants {
            offset,
            constants: SmallVec::from_slice(constants),
            stages,
        })
    }
}

/// A ComputePass
///
/// Provides functions to operate on the pass
/// Will automatically dispatch on drop
pub struct BasicComputePass<'a, 'b> {
    pub(crate) pipeline: Md<Cow<'a, gpu::ComputePipeline>>,
    pub(crate) commands: Vec<ComputePassCommand<'a>>,
    pub(crate) encoder: &'b mut crate::CommandEncoder<'a>,
}

impl std::fmt::Debug for BasicComputePass<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "BasicComputePass")
    }
}

impl<'a, 'b> ComputePass<'a> for BasicComputePass<'a, 'b> {
    fn push_command(&mut self, command: ComputePassCommand<'a>) {
        self.commands.push(command)
    }
}

impl BasicComputePass<'_, '_> {
    /// End the compute pass by dropping it and allowing the encoder to be used again
    pub fn finish(self) {}
}

impl<'a, 'b> Drop for BasicComputePass<'a, 'b> {
    fn drop(&mut self) {
        self.encoder
            .push_command(crate::encoder::Command::ComputePass {
                pipeline: unsafe { Md::take(&mut self.pipeline) },
                commands: self.commands.drain(..).collect(),
            })
    }
}

/// A ComputePass
///
/// Provides functions to operate on the pass
/// Will automatically dispatch on drop
#[cfg(feature = "reflect")]
pub struct ReflectedComputePass<'a, 'b> {
    pub(crate) parent_id: u64,
    pub(crate) bundle_needed: bool,
    pub(crate) push_constant_names:
        Cow<'a, Option<HashMap<String, (u32, gpu::ShaderStages, TypeId)>>>,
    /// Pipeline contained inside a manually drop so that it can be taken an moved into the encoder
    pub(crate) pipeline: Md<Cow<'a, gpu::ComputePipeline>>,
    pub(crate) commands: Vec<ComputePassCommand<'a>>,
    pub(crate) encoder: &'b mut crate::CommandEncoder<'a>,
}

#[cfg(feature = "reflect")]
impl std::fmt::Debug for ReflectedComputePass<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "ReflectedComputePass parent id {}", self.parent_id)
    }
}

#[cfg(feature = "reflect")]
impl<'a, 'b> ComputePass<'a> for ReflectedComputePass<'a, 'b> {
    fn push_command(&mut self, command: ComputePassCommand<'a>) {
        self.commands.push(command)
    }
}

#[cfg(feature = "reflect")]
impl ReflectedComputePass<'_, '_> {
    /// End the compute pass by dropping it and allowing the encoder to be used again
    pub fn finish(self) {}
}

#[cfg(feature = "reflect")]
impl<'a, 'b> Drop for ReflectedComputePass<'a, 'b> {
    fn drop(&mut self) {
        self.encoder
            .push_command(crate::encoder::Command::ComputePass {
                pipeline: unsafe { Md::take(&mut self.pipeline) },
                commands: self.commands.drain(..).collect(),
            })
    }
}

#[cfg(feature = "reflect")]
impl<'a, 'b> ReflectedComputePass<'a, 'b> {
    /// Returns if the pass needs Descriptors to function
    pub fn bundle_needed(&self) -> bool {
        self.bundle_needed
    }

    /// Set a Bundle by reference
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

    /// Set a bundle cloning its data
    pub fn set_bundle_owned(&mut self, bundle: &Bundle) {
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

    /// Set a bundle consuming its data
    pub fn set_bundle_into(&mut self, bundle: Bundle) {
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
        self.bind_descriptors_owned(0, bundle.descriptor_sets);
    }

    /// Push a single constant by variable name
    /// If there are no constants by the name no action will be taken
    /// If the type supplied is different to the type expected this will panic
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
                log::error!("Call to push_constant at {} with value {:?}, with different type than expected", name, constant);
            }
        } else {
            #[cfg(feature = "logging")]
            log::error!("Call to push_constant at {} with value {:?}, when there are no push constants, No action taken", name, constant);
        }
    }
}

// macro_rules! push {
//     ($(($f:tt, $ty:tt),)*) => {
//         impl<'a, 'b> ReflectedComputePass<'a, 'b> {
//             $(
//                 #[allow(missing_docs)]
//                 pub fn $f(&mut self, name: &str, constant: $ty) {
//                     self.push_constant(name, constant);
//                 }
//             )*
//         }
//     };
// }

// push!(
//     (push_u8, u8),
//     (push_u16, u16),
//     (push_u32, u32),
//     (push_u64, u64),
//     (push_uvec2, [u32; 2]),
//     (push_uvec3, [u32; 3]),
//     (push_uvec4, [u32; 4]),
//     (push_i8, i8),
//     (push_i16, i16),
//     (push_i32, i32),
//     (push_i64, i64),
//     (push_svec2, [i32; 2]),
//     (push_svec3, [i32; 3]),
//     (push_svec4, [i32; 4]),
//     (push_f32, f32),
//     (push_f64, f64),
//     (push_vec2, [f32; 2]),
//     (push_vec3, [f32; 3]),
//     (push_vec4, [f32; 4]),
//     (push_mat2, [f32; 4]),
//     (push_mat3, [f32; 9]),
//     (push_mat4, [f32; 16]),
//     (push_dvec2, [f64; 2]),
//     (push_dvec3, [f64; 3]),
//     (push_dvec4, [f64; 4]),
//     (push_dmat2, [f64; 4]),
//     (push_dmat3, [f64; 9]),
//     (push_dmat4, [f64; 16]),
// );
