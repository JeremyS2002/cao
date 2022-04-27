use super::bundle::BundleBuilder;
use super::error;

use crate::texture::AsDimension;

/// Dynamically set objects throught &dyn Resource in BundleBuilder
pub trait Resource {
    /// Set self by location name
    fn set<'a>(
        &'a self,
        builder: BundleBuilder<'a>,
        name: &str,
    ) -> Result<BundleBuilder<'a>, error::SetResourceError>;

    /// Set self by location
    fn set_by_location<'a>(
        &'a self,
        builder: BundleBuilder<'a>,
        set: usize,
        binding: usize,
    ) -> Result<BundleBuilder<'a>, error::SetResourceError>;
}

impl<U: bytemuck::Pod> Resource for crate::Uniform<U> {
    fn set<'a>(
        &'a self,
        builder: BundleBuilder<'a>,
        name: &str,
    ) -> Result<BundleBuilder<'a>, error::SetResourceError> {
        builder.set_buffer(name, self.buffer.slice_ref(..))
    }

    fn set_by_location<'a>(
        &'a self,
        builder: BundleBuilder<'a>,
        set: usize,
        binding: usize,
    ) -> Result<BundleBuilder<'a>, error::SetResourceError> {
        builder.set_buffer_by_location(set, binding, self.buffer.slice_ref(..))
    }
}

impl<U: bytemuck::Pod> Resource for crate::Storage<U> {
    fn set<'a>(
        &'a self,
        builder: BundleBuilder<'a>,
        name: &str,
    ) -> Result<BundleBuilder<'a>, error::SetResourceError> {
        builder.set_buffer(name, self.buffer.slice_ref(..))
    }

    fn set_by_location<'a>(
        &'a self,
        builder: BundleBuilder<'a>,
        set: usize,
        binding: usize,
    ) -> Result<BundleBuilder<'a>, error::SetResourceError> {
        builder.set_buffer_by_location(set, binding, self.buffer.slice_ref(..))
    }
}

impl<D: AsDimension> Resource for crate::GTexture<D> {
    fn set<'a>(
        &'a self,
        builder: BundleBuilder<'a>,
        name: &str,
    ) -> Result<BundleBuilder<'a>, error::SetResourceError> {
        builder.set_texture_ref(name, &self.view)
    }

    fn set_by_location<'a>(
        &'a self,
        builder: BundleBuilder<'a>,
        set: usize,
        binding: usize,
    ) -> Result<BundleBuilder<'a>, error::SetResourceError> {
        builder.set_texture_ref_by_location(set, binding, &self.view)
    }
}

impl<D: AsDimension> Resource for (&crate::GTexture<D>, &gpu::Sampler) {
    fn set<'a>(
        &'a self,
        builder: BundleBuilder<'a>,
        name: &str,
    ) -> Result<BundleBuilder<'a>, error::SetResourceError> {
        builder.set_combined_texture_sampler_ref(name, (&self.0.view, self.1))
    }

    fn set_by_location<'a>(
        &'a self,
        builder: BundleBuilder<'a>,
        set: usize,
        binding: usize,
    ) -> Result<BundleBuilder<'a>, error::SetResourceError> {
        builder.set_combined_texture_sampler_ref_by_location(set, binding, (&self.0.view, self.1))
    }
}

impl Resource for gpu::Sampler {
    fn set<'a>(
        &'a self,
        builder: BundleBuilder<'a>,
        name: &str,
    ) -> Result<BundleBuilder<'a>, error::SetResourceError> {
        builder.set_sampler_ref(name, self)
    }

    fn set_by_location<'a>(
        &'a self,
        builder: BundleBuilder<'a>,
        set: usize,
        binding: usize,
    ) -> Result<BundleBuilder<'a>, error::SetResourceError> {
        builder.set_sampler_ref_by_location(set, binding, self)
    }
}

impl Resource for gpu::BufferSlice<'_> {
    fn set<'a>(
        &'a self,
        builder: BundleBuilder<'a>,
        name: &str,
    ) -> Result<BundleBuilder<'a>, error::SetResourceError> {
        builder.set_buffer(name, self.clone())
    }

    fn set_by_location<'a>(
        &'a self,
        builder: BundleBuilder<'a>,
        set: usize,
        binding: usize,
    ) -> Result<BundleBuilder<'a>, error::SetResourceError> {
        builder.set_buffer_by_location(set, binding, self.clone())
    }
}

impl Resource for gpu::Buffer {
    fn set<'a>(
        &'a self,
        builder: BundleBuilder<'a>,
        name: &str,
    ) -> Result<BundleBuilder<'a>, error::SetResourceError> {
        builder.set_buffer(name, self.slice_ref(..))
    }

    fn set_by_location<'a>(
        &'a self,
        builder: BundleBuilder<'a>,
        set: usize,
        binding: usize,
    ) -> Result<BundleBuilder<'a>, error::SetResourceError> {
        builder.set_buffer_by_location(set, binding, self.slice_ref(..))
    }
}

impl Resource for gpu::TextureView {
    fn set<'a>(
        &'a self,
        builder: BundleBuilder<'a>,
        name: &str,
    ) -> Result<BundleBuilder<'a>, error::SetResourceError> {
        builder.set_texture_ref(name, self)
    }

    fn set_by_location<'a>(
        &'a self,
        builder: BundleBuilder<'a>,
        set: usize,
        binding: usize,
    ) -> Result<BundleBuilder<'a>, error::SetResourceError> {
        builder.set_texture_ref_by_location(set, binding, self)
    }
}

impl Resource for &'_ [&'_ gpu::Sampler] {
    fn set<'a>(
        &'a self,
        builder: BundleBuilder<'a>,
        name: &str,
    ) -> Result<BundleBuilder<'a>, error::SetResourceError> {
        builder.set_sampler_array_ref(name, *self)
    }

    fn set_by_location<'a>(
        &'a self,
        builder: BundleBuilder<'a>,
        set: usize,
        binding: usize,
    ) -> Result<BundleBuilder<'a>, error::SetResourceError> {
        builder.set_sampler_array_ref_by_location(set, binding, *self)
    }
}

impl Resource for &'_ [gpu::BufferSlice<'_>] {
    fn set<'a>(
        &'a self,
        builder: BundleBuilder<'a>,
        name: &str,
    ) -> Result<BundleBuilder<'a>, error::SetResourceError> {
        builder.set_buffer_array_ref(name, *self)
    }

    fn set_by_location<'a>(
        &'a self,
        builder: BundleBuilder<'a>,
        set: usize,
        binding: usize,
    ) -> Result<BundleBuilder<'a>, error::SetResourceError> {
        builder.set_buffer_array_ref_by_location(set, binding, *self)
    }
}

impl Resource for &'_ [&'_ gpu::TextureView] {
    fn set<'a>(
        &'a self,
        builder: BundleBuilder<'a>,
        name: &str,
    ) -> Result<BundleBuilder<'a>, error::SetResourceError> {
        builder.set_texture_array_ref(name, *self)
    }

    fn set_by_location<'a>(
        &'a self,
        builder: BundleBuilder<'a>,
        set: usize,
        binding: usize,
    ) -> Result<BundleBuilder<'a>, error::SetResourceError> {
        builder.set_texture_array_ref_by_location(set, binding, *self)
    }
}
