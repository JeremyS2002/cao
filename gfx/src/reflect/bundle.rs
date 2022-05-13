//! Bundle and BundleBuilder
//!
//! A Bundle is used to automatically set uniform variables in shaders
//! BundleBuilders are used to build bundles from shader

use std::collections::HashMap;

use spirv_reflect::types::descriptor::ReflectDescriptorType;

use super::error;
use super::resource::Resource;

/// BundleBuilder
///
/// Provides methods for creating Bundles from either
/// locations or binding names
///
/// See [`crate::ReflectedGraphics::bundle`] or [`crate::ReflectedCompute::bundle`] to create a bundle builder
pub struct BundleBuilder<'a> {
    /// key of the pipeline this is related to
    pub(crate) parent_id: u64,
    /// The name of the pipeline this is related to
    pub(crate) parent_name: Option<String>,
    /// stores the name of a binding to its location
    pub(crate) map: &'a HashMap<String, (usize, usize)>,
    /// stores the types of bindings so check that the DescriptorSets created are valid
    pub(crate) types: &'a [Vec<super::ResourceType>],
    /// stores the bind descriptor layouts to create bind descriptors from
    pub(crate) layouts: &'a [gpu::DescriptorLayout],
    /// stores DescriptorSetEntries as options so that they can be filled in in any order
    pub(crate) descriptors: Vec<Vec<Option<gpu::DescriptorSetEntry<'a>>>>,
}

impl std::fmt::Debug for BundleBuilder<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "BundleBuilder id {}", self.parent_id)
    }
}

impl<'a> BundleBuilder<'a> {
    /// set a resource
    pub fn set_resource<R: Resource + ?Sized>(
        self,
        name: &str,
        resource: &'a R,
    ) -> Result<Self, error::SetResourceError> {
        resource.set(self, name)
    }

    /// set the texture by location
    pub fn set_resource_by_location<R: Resource + ?Sized>(
        self,
        set: usize,
        binding: usize,
        resource: &'a R,
    ) -> Result<Self, error::SetResourceError> {
        resource.set_by_location(self, set, binding)
    }

    /// set the texture by location name
    pub fn set_texture_ref(
        self,
        name: &str,
        texture: &'a gpu::TextureView,
    ) -> Result<Self, error::SetResourceError> {
        if let Some(&(set, binding)) = self.map.get(name) {
            self.set_texture_ref_by_location(set, binding, texture)
        } else {
            Err(error::SetResourceError::IdNotFound(name.to_string()).into())
        }
    }

    /// set the texture by location name
    pub fn set_texture_owned(
        self,
        name: &str,
        texture: gpu::TextureView,
    ) -> Result<Self, error::SetResourceError> {
        if let Some(&(set, binding)) = self.map.get(name) {
            self.set_texture_owned_by_location(set, binding, texture)
        } else {
            Err(error::SetResourceError::IdNotFound(name.to_string()).into())
        }
    }

    /// set the buffer by location name
    pub fn set_buffer(
        self,
        name: &str,
        buffer: gpu::BufferSlice<'a>,
    ) -> Result<Self, error::SetResourceError> {
        if let Some(&(set, binding)) = self.map.get(name) {
            self.set_buffer_by_location(set, binding, buffer)
        } else {
            Err(error::SetResourceError::IdNotFound(name.to_string()).into())
        }
    }

    /// set the sampler by location name
    pub fn set_sampler_ref(
        self,
        name: &str,
        sampler: &'a gpu::Sampler,
    ) -> Result<Self, error::SetResourceError> {
        if let Some(&(set, binding)) = self.map.get(name) {
            self.set_sampler_ref_by_location(set, binding, sampler)
        } else {
            Err(error::SetResourceError::IdNotFound(name.to_string()).into())
        }
    }

    /// set the sampler by location name
    pub fn set_sampler_owned(
        self,
        name: &str,
        sampler: gpu::Sampler,
    ) -> Result<Self, error::SetResourceError> {
        if let Some(&(set, binding)) = self.map.get(name) {
            self.set_sampler_owned_by_location(set, binding, sampler)
        } else {
            Err(error::SetResourceError::IdNotFound(name.to_string()).into())
        }
    }

    /// set the combined texture and sampler by location name
    pub fn set_combined_texture_sampler_ref(
        self,
        name: &str,
        combined: (&'a gpu::TextureView, &'a gpu::Sampler),
    ) -> Result<Self, error::SetResourceError> {
        if let Some(&(set, binding)) = self.map.get(name) {
            self.set_combined_texture_sampler_ref_by_location(set, binding, combined)
        } else {
            Err(error::SetResourceError::IdNotFound(name.to_string()).into())
        }
    }

    /// set the combined texture and sampler by location name
    pub fn set_combined_texture_sampler_owned(
        self,
        name: &str,
        combined: (gpu::TextureView, gpu::Sampler),
    ) -> Result<Self, error::SetResourceError> {
        if let Some(&(set, binding)) = self.map.get(name) {
            self.set_combined_texture_sampler_owned_by_location(set, binding, combined)
        } else {
            Err(error::SetResourceError::IdNotFound(name.to_string()).into())
        }
    }

    /// set the texture by location name
    pub fn set_texture_array_ref(
        self,
        name: &str,
        textures: &[&'a gpu::TextureView],
    ) -> Result<Self, error::SetResourceError> {
        if let Some(&(set, binding)) = self.map.get(name) {
            self.set_texture_array_ref_by_location(set, binding, textures)
        } else {
            Err(error::SetResourceError::IdNotFound(name.to_string()).into())
        }
    }

    /// set the texture by location name
    pub fn set_texture_array_owned(
        self,
        name: &str,
        textures: Vec<gpu::TextureView>,
    ) -> Result<Self, error::SetResourceError> {
        if let Some(&(set, binding)) = self.map.get(name) {
            self.set_texture_array_owned_by_location(set, binding, textures)
        } else {
            Err(error::SetResourceError::IdNotFound(name.to_string()).into())
        }
    }

    /// set the buffer by location name
    pub fn set_buffer_array_ref(
        self,
        name: &str,
        buffers: &'a [gpu::BufferSlice<'a>],
    ) -> Result<Self, error::SetResourceError> {
        if let Some(&(set, binding)) = self.map.get(name) {
            self.set_buffer_array_ref_by_location(set, binding, buffers)
        } else {
            Err(error::SetResourceError::IdNotFound(name.to_string()).into())
        }
    }

    /// set the buffer by location name
    pub fn set_buffer_array_owned(
        self,
        name: &str,
        buffers: Vec<gpu::BufferSlice<'a>>,
    ) -> Result<Self, error::SetResourceError> {
        if let Some(&(set, binding)) = self.map.get(name) {
            self.set_buffer_array_owned_by_location(set, binding, buffers)
        } else {
            Err(error::SetResourceError::IdNotFound(name.to_string()).into())
        }
    }

    /// set the sampler by location name
    pub fn set_sampler_array_ref(
        self,
        name: &str,
        samplers: &[&'a gpu::Sampler],
    ) -> Result<Self, error::SetResourceError> {
        if let Some(&(set, binding)) = self.map.get(name) {
            self.set_sampler_array_ref_by_location(set, binding, samplers)
        } else {
            Err(error::SetResourceError::IdNotFound(name.to_string()).into())
        }
    }

    /// set the sampler by location name
    pub fn set_sampler_array_owned(
        self,
        name: &str,
        samplers: Vec<gpu::Sampler>,
    ) -> Result<Self, error::SetResourceError> {
        if let Some(&(set, binding)) = self.map.get(name) {
            self.set_sampler_array_owned_by_location(set, binding, samplers)
        } else {
            Err(error::SetResourceError::IdNotFound(name.to_string()).into())
        }
    }

    /// set the combined texture and sampler by location name
    pub fn set_combined_texture_sampler_array_ref(
        self,
        name: &str,
        combined: &[(&'a gpu::TextureView, &'a gpu::Sampler)],
    ) -> Result<Self, error::SetResourceError> {
        if let Some(&(set, binding)) = self.map.get(name) {
            self.set_combined_texture_sampler_array_ref_by_location(set, binding, combined)
        } else {
            Err(error::SetResourceError::IdNotFound(name.to_string()).into())
        }
    }

    /// set the combined texture and sampler by location name
    pub fn set_combined_texture_sampler_array_owned(
        self,
        name: &str,
        combined: Vec<(gpu::TextureView, gpu::Sampler)>,
    ) -> Result<Self, error::SetResourceError> {
        if let Some(&(set, binding)) = self.map.get(name) {
            self.set_combined_texture_sampler_array_owned_by_location(set, binding, combined)
        } else {
            Err(error::SetResourceError::IdNotFound(name.to_string()).into())
        }
    }

    /// set the texture by set and binding
    pub fn set_texture_ref_by_location(
        mut self,
        set: usize,
        binding: usize,
        texture: &'a gpu::TextureView,
    ) -> Result<Self, error::SetResourceError> {
        if self
            .types
            .get(set)
            .expect("ERROR: Bundle created with largest set greater that max number of sets")
            .get(binding)
            .expect("ERROR: Bundle created with largest binding greater than max bindings")
            .count
            != 1
        {
            Err(error::SetResourceError::SingleExpected)?;
        }
        match self.types[set][binding].ty {
            ReflectDescriptorType::SampledImage => {
                self.descriptors[set][binding] = Some(gpu::DescriptorSetEntry::texture_ref(
                    texture,
                    gpu::TextureLayout::General,
                ))
            }
            ReflectDescriptorType::StorageImage => {
                self.descriptors[set][binding] = Some(gpu::DescriptorSetEntry::texture_ref(
                    texture,
                    gpu::TextureLayout::General,
                ))
            }
            _ => Err(error::SetResourceError::WrongType(
                ReflectDescriptorType::SampledImage,
                self.types[set][binding].ty,
            ))?,
        }
        Ok(self)
    }

    /// set the texture by set and binding
    pub fn set_texture_owned_by_location(
        mut self,
        set: usize,
        binding: usize,
        texture: gpu::TextureView,
    ) -> Result<Self, error::SetResourceError> {
        if self
            .types
            .get(set)
            .expect("ERROR: Bundle created with largest set greater that max number of sets")
            .get(binding)
            .expect("ERROR: Bundle created with largest binding greater than max bindings")
            .count
            != 1
        {
            Err(error::SetResourceError::SingleExpected)?;
        }
        match self.types[set][binding].ty {
            ReflectDescriptorType::SampledImage => {
                self.descriptors[set][binding] = Some(gpu::DescriptorSetEntry::texture_owned(
                    texture,
                    gpu::TextureLayout::General,
                ))
            }
            ReflectDescriptorType::StorageImage => {
                self.descriptors[set][binding] = Some(gpu::DescriptorSetEntry::texture_owned(
                    texture,
                    gpu::TextureLayout::General,
                ))
            }
            _ => Err(error::SetResourceError::WrongType(
                ReflectDescriptorType::SampledImage,
                self.types[set][binding].ty,
            ))?,
        }
        Ok(self)
    }

    /// set the buffer by set and binding
    pub fn set_buffer_by_location(
        mut self,
        set: usize,
        binding: usize,
        buffer: gpu::BufferSlice<'a>,
    ) -> Result<Self, error::SetResourceError> {
        if self
            .types
            .get(set)
            .expect("ERROR: Bundle created with largest set greater that max number of sets")
            .get(binding)
            .expect("ERROR: Bundle created with largest binding greater than max bindings")
            .count
            != 1
        {
            Err(error::SetResourceError::SingleExpected)?;
        }
        match self.types[set][binding].ty {
            ReflectDescriptorType::UniformBuffer => {
                self.descriptors[set][binding] = Some(gpu::DescriptorSetEntry::buffer(buffer))
            }
            ReflectDescriptorType::StorageBuffer => {
                self.descriptors[set][binding] = Some(gpu::DescriptorSetEntry::buffer(buffer))
            }
            _ => Err(error::SetResourceError::WrongType(
                ReflectDescriptorType::UniformBuffer,
                self.types[set][binding].ty,
            ))?,
        }
        Ok(self)
    }

    /// set the sampler by set and binding
    pub fn set_sampler_ref_by_location(
        mut self,
        set: usize,
        binding: usize,
        sampler: &'a gpu::Sampler,
    ) -> Result<Self, error::SetResourceError> {
        if self
            .types
            .get(set)
            .expect("ERROR: Bundle created with largest set greater that max number of sets")
            .get(binding)
            .expect("ERROR: Bundle created with largest binding greater than max bindings")
            .count
            != 1
        {
            Err(error::SetResourceError::SingleExpected)?;
        }
        match self.types[set][binding].ty {
            ReflectDescriptorType::Sampler => {
                self.descriptors[set][binding] = Some(gpu::DescriptorSetEntry::sampler_ref(sampler))
            }
            _ => Err(error::SetResourceError::WrongType(
                ReflectDescriptorType::Sampler,
                self.types[set][binding].ty,
            ))?,
        }
        Ok(self)
    }

    /// set the sampler by set and binding
    pub fn set_sampler_owned_by_location(
        mut self,
        set: usize,
        binding: usize,
        sampler: gpu::Sampler,
    ) -> Result<Self, error::SetResourceError> {
        if self
            .types
            .get(set)
            .expect("ERROR: Bundle created with largest set greater that max number of sets")
            .get(binding)
            .expect("ERROR: Bundle created with largest binding greater than max bindings")
            .count
            != 1
        {
            Err(error::SetResourceError::SingleExpected)?;
        }
        match self.types[set][binding].ty {
            ReflectDescriptorType::Sampler => {
                self.descriptors[set][binding] =
                    Some(gpu::DescriptorSetEntry::sampler_owned(sampler))
            }
            _ => Err(error::SetResourceError::WrongType(
                ReflectDescriptorType::Sampler,
                self.types[set][binding].ty,
            ))?,
        }
        Ok(self)
    }

    /// set the combined texture and sampler by location
    pub fn set_combined_texture_sampler_ref_by_location(
        mut self,
        set: usize,
        binding: usize,
        combined: (&'a gpu::TextureView, &'a gpu::Sampler),
    ) -> Result<Self, error::SetResourceError> {
        if self
            .types
            .get(set)
            .expect("ERROR: Bundle created with largest set greater that max number of sets")
            .get(binding)
            .expect("ERROR: Bundle created with largest binding greater than max bindings")
            .count
            != 1
        {
            Err(error::SetResourceError::SingleExpected)?;
        }
        match self.types[set][binding].ty {
            ReflectDescriptorType::CombinedImageSampler => {
                self.descriptors[set][binding] =
                    Some(gpu::DescriptorSetEntry::combined_texture_sampler_ref(
                        combined.0,
                        gpu::TextureLayout::General,
                        combined.1,
                    ))
            }
            _ => Err(error::SetResourceError::WrongType(
                ReflectDescriptorType::CombinedImageSampler,
                self.types[set][binding].ty,
            ))?,
        }
        Ok(self)
    }

    /// set the combined texture and sampler by location
    pub fn set_combined_texture_sampler_owned_by_location(
        mut self,
        set: usize,
        binding: usize,
        combined: (gpu::TextureView, gpu::Sampler),
    ) -> Result<Self, error::SetResourceError> {
        if self
            .types
            .get(set)
            .expect("ERROR: Bundle created with largest set greater that max number of sets")
            .get(binding)
            .expect("ERROR: Bundle created with largest binding greater than max bindings")
            .count
            != 1
        {
            Err(error::SetResourceError::SingleExpected)?;
        }
        match self.types[set][binding].ty {
            ReflectDescriptorType::CombinedImageSampler => {
                self.descriptors[set][binding] =
                    Some(gpu::DescriptorSetEntry::combined_texture_sampler_owned(
                        combined.0,
                        gpu::TextureLayout::General,
                        combined.1,
                    ))
            }
            _ => Err(error::SetResourceError::WrongType(
                ReflectDescriptorType::CombinedImageSampler,
                self.types[set][binding].ty,
            ))?,
        }
        Ok(self)
    }

    /// set the texture by set and binding
    pub fn set_texture_array_ref_by_location(
        mut self,
        set: usize,
        binding: usize,
        textures: &[&'a gpu::TextureView],
    ) -> Result<Self, error::SetResourceError> {
        match self.types[set][binding].ty {
            ReflectDescriptorType::SampledImage => {
                self.descriptors[set][binding] = Some(gpu::DescriptorSetEntry::texture_array_ref(
                    &textures
                        .iter()
                        .map(|v| (*v, gpu::TextureLayout::General))
                        .collect::<Vec<_>>(),
                ))
            }
            ReflectDescriptorType::StorageImage => {
                self.descriptors[set][binding] = Some(gpu::DescriptorSetEntry::texture_array_ref(
                    &textures
                        .iter()
                        .map(|v| (*v, gpu::TextureLayout::General))
                        .collect::<Vec<_>>(),
                ))
            }
            _ => Err(error::SetResourceError::WrongType(
                ReflectDescriptorType::SampledImage,
                self.types[set][binding].ty,
            ))?,
        }
        Ok(self)
    }

    /// set the texture by set and binding
    pub fn set_texture_array_owned_by_location(
        mut self,
        set: usize,
        binding: usize,
        textures: Vec<gpu::TextureView>,
    ) -> Result<Self, error::SetResourceError> {
        if self
            .types
            .get(set)
            .expect("ERROR: Bundle created with largest set greater that max number of sets")
            .get(binding)
            .expect("ERROR: Bundle created with largest binding greater than max bindings")
            .count
            == 1
        {
            Err(error::SetResourceError::ArrayExpected)?;
        }
        match self.types[set][binding].ty {
            ReflectDescriptorType::SampledImage => {
                self.descriptors[set][binding] = Some(gpu::DescriptorSetEntry::texture_array_owned(
                    textures
                        .into_iter()
                        .zip(std::iter::repeat(gpu::TextureLayout::General))
                        .collect::<Vec<_>>(),
                ))
            }
            ReflectDescriptorType::StorageImage => {
                self.descriptors[set][binding] = Some(gpu::DescriptorSetEntry::texture_array_owned(
                    textures
                        .into_iter()
                        .zip(std::iter::repeat(gpu::TextureLayout::General))
                        .collect::<Vec<_>>(),
                ))
            }
            _ => Err(error::SetResourceError::WrongType(
                ReflectDescriptorType::SampledImage,
                self.types[set][binding].ty,
            ))?,
        }
        Ok(self)
    }

    /// set the buffer array by set and binding
    pub fn set_buffer_array_ref_by_location(
        mut self,
        set: usize,
        binding: usize,
        buffers: &'a [gpu::BufferSlice<'a>],
    ) -> Result<Self, error::SetResourceError> {
        if self
            .types
            .get(set)
            .expect("ERROR: Bundle created with largest set greater that max number of sets")
            .get(binding)
            .expect("ERROR: Bundle created with largest binding greater than max bindings")
            .count
            == 1
        {
            Err(error::SetResourceError::ArrayExpected)?;
        }
        match self.types[set][binding].ty {
            ReflectDescriptorType::UniformBuffer => {
                self.descriptors[set][binding] =
                    Some(gpu::DescriptorSetEntry::buffer_array_ref(buffers))
            }
            ReflectDescriptorType::StorageBuffer => {
                self.descriptors[set][binding] =
                    Some(gpu::DescriptorSetEntry::buffer_array_ref(buffers))
            }
            _ => Err(error::SetResourceError::WrongType(
                ReflectDescriptorType::UniformBuffer,
                self.types[set][binding].ty,
            ))?,
        }
        Ok(self)
    }

    /// set the buffer array by set and binding
    pub fn set_buffer_array_owned_by_location(
        mut self,
        set: usize,
        binding: usize,
        buffers: Vec<gpu::BufferSlice<'a>>,
    ) -> Result<Self, error::SetResourceError> {
        if self
            .types
            .get(set)
            .expect("ERROR: Bundle created with largest set greater that max number of sets")
            .get(binding)
            .expect("ERROR: Bundle created with largest binding greater than max bindings")
            .count
            == 1
        {
            Err(error::SetResourceError::ArrayExpected)?;
        }
        match self.types[set][binding].ty {
            ReflectDescriptorType::UniformBuffer => {
                self.descriptors[set][binding] =
                    Some(gpu::DescriptorSetEntry::buffer_array_owned(buffers))
            }
            ReflectDescriptorType::StorageBuffer => {
                self.descriptors[set][binding] =
                    Some(gpu::DescriptorSetEntry::buffer_array_owned(buffers))
            }
            _ => Err(error::SetResourceError::WrongType(
                ReflectDescriptorType::UniformBuffer,
                self.types[set][binding].ty,
            ))?,
        }
        Ok(self)
    }

    /// set the sampler array by set and binding
    pub fn set_sampler_array_ref_by_location(
        mut self,
        set: usize,
        binding: usize,
        samplers: &[&'a gpu::Sampler],
    ) -> Result<Self, error::SetResourceError> {
        if self
            .types
            .get(set)
            .expect("ERROR: Bundle created with largest set greater that max number of sets")
            .get(binding)
            .expect("ERROR: Bundle created with largest binding greater than max bindings")
            .count
            == 1
        {
            Err(error::SetResourceError::ArrayExpected)?;
        }
        match self.types[set][binding].ty {
            ReflectDescriptorType::Sampler => {
                self.descriptors[set][binding] =
                    Some(gpu::DescriptorSetEntry::sampler_array_ref(samplers))
            }
            _ => Err(error::SetResourceError::WrongType(
                ReflectDescriptorType::Sampler,
                self.types[set][binding].ty,
            ))?,
        }
        Ok(self)
    }

    /// set the sampler array by set and binding
    pub fn set_sampler_array_owned_by_location(
        mut self,
        set: usize,
        binding: usize,
        samplers: Vec<gpu::Sampler>,
    ) -> Result<Self, error::SetResourceError> {
        if self
            .types
            .get(set)
            .expect("ERROR: Bundle created with largest set greater that max number of sets")
            .get(binding)
            .expect("ERROR: Bundle created with largest binding greater than max bindings")
            .count
            == 1
        {
            Err(error::SetResourceError::ArrayExpected)?;
        }
        match self.types[set][binding].ty {
            ReflectDescriptorType::Sampler => {
                self.descriptors[set][binding] =
                    Some(gpu::DescriptorSetEntry::sampler_array_owned(samplers))
            }
            _ => Err(error::SetResourceError::WrongType(
                ReflectDescriptorType::Sampler,
                self.types[set][binding].ty,
            ))?,
        }
        Ok(self)
    }

    /// set the combined texture and sampler by location
    pub fn set_combined_texture_sampler_array_ref_by_location(
        mut self,
        set: usize,
        binding: usize,
        combined: &[(&'a gpu::TextureView, &'a gpu::Sampler)],
    ) -> Result<Self, error::SetResourceError> {
        if self
            .types
            .get(set)
            .expect("ERROR: Bundle created with largest set greater that max number of sets")
            .get(binding)
            .expect("ERROR: Bundle created with largest binding greater than max bindings")
            .count
            == 1
        {
            Err(error::SetResourceError::ArrayExpected)?;
        }
        match self.types[set][binding].ty {
            ReflectDescriptorType::CombinedImageSampler => {
                self.descriptors[set][binding] =
                    Some(gpu::DescriptorSetEntry::combined_texture_sampler_array_ref(
                        &combined
                            .iter()
                            .map(|(v, s)| (*v, gpu::TextureLayout::General, *s))
                            .collect::<Vec<_>>(),
                    ))
            }
            _ => Err(error::SetResourceError::WrongType(
                ReflectDescriptorType::CombinedImageSampler,
                self.types[set][binding].ty,
            ))?,
        }
        Ok(self)
    }

    /// set the combined texture and sampler by location
    pub fn set_combined_texture_sampler_array_owned_by_location(
        mut self,
        set: usize,
        binding: usize,
        combined: Vec<(gpu::TextureView, gpu::Sampler)>,
    ) -> Result<Self, error::SetResourceError> {
        if self
            .types
            .get(set)
            .expect("ERROR: Bundle created with largest set greater that max number of sets")
            .get(binding)
            .expect("ERROR: Bundle created with largest binding greater than max bindings")
            .count
            == 1
        {
            Err(error::SetResourceError::ArrayExpected)?;
        }
        match self.types[set][binding].ty {
            ReflectDescriptorType::CombinedImageSampler => {
                self.descriptors[set][binding] = Some(
                    gpu::DescriptorSetEntry::combined_texture_sampler_array_owned(
                        combined
                            .into_iter()
                            .map(|(v, s)| (v, gpu::TextureLayout::General, s))
                            .collect::<Vec<_>>(),
                    ),
                )
            }
            _ => Err(error::SetResourceError::WrongType(
                ReflectDescriptorType::CombinedImageSampler,
                self.types[set][binding].ty,
            ))?,
        }
        Ok(self)
    }

    /// Build a single descriptor set from the bundle
    pub fn build_set(
        &self,
        device: &gpu::Device,
        set: u32,
    ) -> Result<gpu::DescriptorSet, gpu::Error> {
        let name = self.parent_name.as_ref();

        let v: &Vec<Option<gpu::DescriptorSetEntry<'_>>> =
            self.descriptors.get(set as usize).unwrap();
        let layout = self.layouts.get(set as usize).unwrap();

        let mut binding = 0;
        let entries = v.iter()
            .map(|e| {
                if let Some(e) = e {
                    binding += 1;
                    e.clone()
                } else {
                    panic!("ERROR: Call to build set {} on bundle {:?} without setting all fields\nMissing binding {}", set, name, binding);
                }
            })
            .collect::<Vec<_>>();

        Ok(device.create_descriptor_set(&gpu::DescriptorSetDesc {
            name: name
                .as_ref()
                .map(|n| format!("{}_descriptor_set_{}", n, set)),
            entries: &entries,
            layout,
        })?)
    }

    /// Build a Bundle from the current set
    pub fn build(&self, device: &gpu::Device) -> Result<Bundle, gpu::Error> {
        let mut set: u32 = 0;
        let mut binding: u32 = 0;
        let name = &self.parent_name;
        let descriptor_sets = self
            .descriptors
            .iter()
            .zip(self.layouts)
            .map(|(v, layout)| {
                let entries = v
                    .into_iter()
                    .map(|e| {
                        if let Some(e) = e {
                            binding += 1;
                            e.clone()
                        } else {
                            panic!("ERROR: Call to build on bundle from Parent ({} {:?}) without setting all fields\nMissing set: {} binding: {}", self.parent_id, self.parent_name, set, binding)
                        }
                    })
                    .collect::<Vec<_>>();

                set += 1;
                binding = 0;
                let desc = gpu::DescriptorSetDesc {
                    name: name.as_ref().map(|n| {
                        format!("{}_descriptor_set_{}", n, set)
                    }),
                    entries: &entries,
                    layout,
                };
                let descriptor = device.create_descriptor_set(&desc)?;
                Ok(descriptor)
            })
            .collect::<Result<Vec<_>, gpu::Error>>()?;

        Ok(Bundle {
            parent_id: self.parent_id,
            descriptor_sets,
        })
    }

    /// Get the parent id of self
    pub fn parent_id(&self) -> u64 {
        self.parent_id
    }
}

/// a collection of DescriptorSets specific to a Renderer
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Bundle {
    /// The key this bundle is related to
    pub(crate) parent_id: u64,
    /// the DescriptorSets
    pub descriptor_sets: Vec<gpu::DescriptorSet>,
}

impl Bundle {
    /// Create a bundle from raw
    pub fn from_raw(parent_id: u64, sets: Vec<gpu::DescriptorSet>) -> Self {
        Self {
            parent_id,
            descriptor_sets: sets,
        }
    }
}
