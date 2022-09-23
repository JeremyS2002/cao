//! Mesh types and Vertex traits
//!
//! [`IndexedMesh`] and [`BasicMesh`] wrap vertex (and index buffers) and statically type the vertex that the mesh has as well as drawing operations
//!
//! The [`Vertex`] trait should be implemented by vertices, It allows different types of vertices to be used with the same pipeine as long as they have the correct attributes

pub mod vertex;

pub use vertex::*;

/// A mesh with indexing
///
/// Drawing this mesh instanced will bind the instance buffer to location 1
#[derive(Debug, Clone)]
pub struct Mesh<V: Vertex> {
    /// vertex buffer, usage: COPY_SRC COPY_DST VERTEX
    pub vertex_buffer: gpu::Buffer,
    /// (index buffer, index_count), buffer usage: COPY_SRC COPY_DST INDEX
    pub indices: Option<(gpu::Buffer, u32)>,
    /// (indirect_buffer, draw_count), buffer usage: COPY_SRC COPY_DST STORAGE UNIFORM
    pub indirect: Option<(gpu::Buffer, u32)>,

    /// Marks the mesh so that the vertex state can be infered
    pub _vertex_marker: std::marker::PhantomData<V>,

    /// the number of vertices in the vertex buffer
    pub vertex_count: u32,
}

impl<V: Vertex> Mesh<V> {
/// Create a new Mesh
    ///
    /// The mesh won't be valid until the encoder is submitted
    pub fn from_usage_indexed_indirect(
        encoder: &mut crate::CommandEncoder<'_>,
        device: &gpu::Device,
        vertices: &[V],
        vertex_usage: gpu::BufferUsage,
        indices: &[u32],
        index_usage: gpu::BufferUsage,
        indirect: &[gpu::DrawIndexedIndirectCommand],
        indirect_usage: gpu::BufferUsage,
        name: Option<&str>,
    ) -> Result<Self, gpu::Error> {
        let vertex_name = if let Some(name) = &name {
            Some(format!("{}_vertex_buffer", name))
        } else {
            None
        };
        let vertex_buffer = device.create_buffer(&gpu::BufferDesc {
            size: (std::mem::size_of::<V>() * vertices.len()) as u64,
            usage: gpu::BufferUsage::COPY_SRC
                | gpu::BufferUsage::COPY_DST
                | gpu::BufferUsage::VERTEX
                | vertex_usage,
            memory: gpu::MemoryType::Device,
            name: vertex_name,
        })?;

        let index_name = if let Some(name) = &name {
            Some(format!("{}_index_buffer", name))
        } else {
            None
        };

        let index_buffer = device.create_buffer(&gpu::BufferDesc {
            size: (std::mem::size_of::<u32>() * indices.len()) as u64,
            usage: gpu::BufferUsage::COPY_SRC
                | gpu::BufferUsage::COPY_DST
                | gpu::BufferUsage::INDEX
                | index_usage,
            memory: gpu::MemoryType::Device,
            name: index_name,
        })?;

        let indirect_name = if let Some(name) = &name {
            Some(format!("{}_indirect_buffer", name))
        } else {
            None
        };

        let indirect_buffer = device.create_buffer(&gpu::BufferDesc {
            size: (std::mem::size_of::<gpu::DrawIndexedIndirectCommand>() * indirect.len()) as u64,
            usage: gpu::BufferUsage::COPY_SRC
                | gpu::BufferUsage::COPY_DST
                | gpu::BufferUsage::STORAGE
                | gpu::BufferUsage::UNIFORM
                | indirect_usage,
            memory: gpu::MemoryType::Device,
            name: indirect_name,
        })?;

        let vertex_staging_buffer = device.create_buffer(&gpu::BufferDesc {
            size: (std::mem::size_of::<V>() * vertices.len()) as u64,
            usage: gpu::BufferUsage::COPY_SRC,
            memory: gpu::MemoryType::Host,
            name: None,
        })?;
        let index_staging_buffer = device.create_buffer(&gpu::BufferDesc {
            size: (std::mem::size_of::<u32>() * indices.len()) as u64,
            usage: gpu::BufferUsage::COPY_SRC,
            memory: gpu::MemoryType::Host,
            name: None,
        })?;
        let indirect_staging_buffer = device.create_buffer(&gpu::BufferDesc {
            size: (std::mem::size_of::<gpu::DrawIndexedIndirectCommand>() * indirect.len()) as u64,
            usage: gpu::BufferUsage::COPY_SRC,
            memory: gpu::MemoryType::Host,
            name: None,
        })?;

        vertex_staging_buffer
            .slice_owned(..)
            .write(bytemuck::cast_slice(&vertices))?;
        index_staging_buffer
            .slice_owned(..)
            .write(bytemuck::cast_slice(&indices))?;
        indirect_staging_buffer
            .slice_owned(..)
            .write(bytemuck::cast_slice(indirect))?;

        encoder.copy_buffer_to_buffer(
            vertex_staging_buffer.slice_owned(..),
            vertex_buffer.slice_owned(..),
        );
        encoder.copy_buffer_to_buffer(
            index_staging_buffer.slice_owned(..),
            index_buffer.slice_owned(..),
        );
        encoder.copy_buffer_to_buffer(
            indirect_staging_buffer.slice_owned(..),
            indirect_buffer.slice_owned(..),
        );

        Ok(Self {
            vertex_buffer,
            indices: Some((index_buffer, indices.len() as u32)),
            indirect: Some((indirect_buffer, indirect.len() as u32)),

            _vertex_marker: std::marker::PhantomData,

            vertex_count: vertices.len() as u32,
        })
    }

    /// Create a new Mesh
    ///
    /// The mesh won't be valid until the encoder is submitted
    pub fn indexed(
        encoder: &mut crate::CommandEncoder<'_>,
        device: &gpu::Device,
        vertices: &[V],
        indices: &[u32],
        name: Option<&str>,
    ) -> Result<Self, gpu::Error> {
        Self::from_usage_indexed(
            encoder,
            device,
            vertices,
            gpu::BufferUsage::empty(),
            indices,
            gpu::BufferUsage::empty(),
            name,
        )
    }

    /// Create a new Mesh
    ///
    /// The mesh won't be valid until the encoder is submitted
    pub fn from_usage_indexed(
        encoder: &mut crate::CommandEncoder<'_>,
        device: &gpu::Device,
        vertices: &[V],
        vertex_usage: gpu::BufferUsage,
        indices: &[u32],
        index_usage: gpu::BufferUsage,
        name: Option<&str>,
    ) -> Result<Self, gpu::Error> {
        let vertex_name = if let Some(name) = &name {
            Some(format!("{}_vertex_buffer", name))
        } else {
            None
        };
        let vertex_buffer = device.create_buffer(&gpu::BufferDesc {
            size: (std::mem::size_of::<V>() * vertices.len()) as u64,
            usage: gpu::BufferUsage::COPY_SRC
                | gpu::BufferUsage::COPY_DST
                | gpu::BufferUsage::VERTEX
                | vertex_usage,
            memory: gpu::MemoryType::Device,
            name: vertex_name,
        })?;

        let index_name = if let Some(name) = &name {
            Some(format!("{}_index_buffer", name))
        } else {
            None
        };

        let index_buffer = device.create_buffer(&gpu::BufferDesc {
            size: (std::mem::size_of::<u32>() * indices.len()) as u64,
            usage: gpu::BufferUsage::COPY_SRC
                | gpu::BufferUsage::COPY_DST
                | gpu::BufferUsage::INDEX
                | index_usage,
            memory: gpu::MemoryType::Device,
            name: index_name,
        })?;

        let vertex_staging_buffer = device.create_buffer(&gpu::BufferDesc {
            size: (std::mem::size_of::<V>() * vertices.len()) as u64,
            usage: gpu::BufferUsage::COPY_SRC,
            memory: gpu::MemoryType::Host,
            name: None,
        })?;
        let index_staging_buffer = device.create_buffer(&gpu::BufferDesc {
            size: (std::mem::size_of::<u32>() * indices.len()) as u64,
            usage: gpu::BufferUsage::COPY_SRC,
            memory: gpu::MemoryType::Host,
            name: None,
        })?;

        vertex_staging_buffer
            .slice_owned(..)
            .write(bytemuck::cast_slice(&vertices))?;
        index_staging_buffer
            .slice_owned(..)
            .write(bytemuck::cast_slice(&indices))?;

        encoder.copy_buffer_to_buffer(
            vertex_staging_buffer.slice_owned(..),
            vertex_buffer.slice_owned(..),
        );
        encoder.copy_buffer_to_buffer(
            index_staging_buffer.slice_owned(..),
            index_buffer.slice_owned(..),
        );

        Ok(Self {
            vertex_buffer,
            indices: Some((index_buffer, indices.len() as u32)),
            indirect: None,

            _vertex_marker: std::marker::PhantomData,

            vertex_count: vertices.len() as u32,
        })
    }

    /// Create a new Mesh
    ///
    /// The mesh won't be valid until the encoder is submitted
    pub fn basic(
        encoder: &mut crate::CommandEncoder<'_>,
        device: &gpu::Device,
        vertices: &[V],
        name: Option<&str>,
    ) -> Result<Self, gpu::Error> {
        Self::from_usage_basic(encoder, device, vertices, gpu::BufferUsage::empty(), name)
    }

    /// Create a new Mesh
    ///
    /// The mesh won't be valid until the encoder is submitted
    pub fn from_usage_basic(
        encoder: &mut crate::CommandEncoder<'_>,
        device: &gpu::Device,
        vertices: &[V],
        vertex_usage: gpu::BufferUsage,
        name: Option<&str>,
    ) -> Result<Self, gpu::Error> {
        let vertex_name = if let Some(name) = name {
            Some(format!("{}_vertex_buffer", name))
        } else {
            None
        };
        let vertex_buffer = device.create_buffer(&gpu::BufferDesc {
            size: (std::mem::size_of::<V>() * vertices.len()) as u64,
            usage: gpu::BufferUsage::COPY_SRC
                | gpu::BufferUsage::COPY_DST
                | gpu::BufferUsage::VERTEX
                | vertex_usage,
            memory: gpu::MemoryType::Device,
            name: vertex_name,
        })?;

        let vertex_staging_buffer = device.create_buffer(&gpu::BufferDesc {
            size: (std::mem::size_of::<V>() * vertices.len()) as u64,
            usage: gpu::BufferUsage::COPY_SRC,
            memory: gpu::MemoryType::Host,
            name: None,
        })?;

        vertex_staging_buffer
            .slice_owned(..)
            .write(bytemuck::cast_slice(&vertices))?;

        encoder.copy_buffer_to_buffer(
            vertex_staging_buffer.slice_owned(..),
            vertex_buffer.slice_owned(..),
        );

        Ok(Self {
            vertex_buffer,
            indices: None,
            indirect: None,

            _vertex_marker: std::marker::PhantomData,

            vertex_count: vertices.len() as u32,
        })
    }
}

impl<V: Vertex> Mesh<V> {
    /// Draw self by reference
    /// 
    /// If the mesh has an indirect draw buffer which makes use of instanced drawing then instance buffers should be bound first
    pub fn draw_ref<'a>(&'a self, pass: &mut dyn crate::GraphicsPass<'a>) {
        pass.bind_vertex_buffer(self.vertex_buffer.slice_ref(..), 0);

        if let Some((index_buffer, index_count)) = &self.indices {
            pass.bind_index_buffer(index_buffer.slice_ref(..), gpu::IndexType::U32);
            if let Some((indirect_buffer, indirect_count)) = &self.indirect {
                pass.draw_indexed_indirect_ref(indirect_buffer, 0, *indirect_count, std::mem::size_of::<gpu::DrawIndexedIndirectCommand>() as _);
            } else {
                pass.draw_indexed(0, *index_count, 0, 1, 0);
            }
        } else {
            if let Some((indirect_buffer, indirect_count)) = &self.indirect {
                pass.draw_indirect_ref(indirect_buffer, 0, *indirect_count, std::mem::size_of::<gpu::DrawIndirectCommand>() as _);
            } else {
                pass.draw(0, self.vertex_count, 0, 1);
            }
        }
    }

    /// Draw self by clone
    ///
    /// If the mesh has an indirect draw buffer which makes use of instanced drawing then instance buffers should be bound first
    pub fn draw_owned<'a>(self, pass: &mut dyn crate::GraphicsPass<'a>) {
        pass.bind_vertex_buffer(self.vertex_buffer.slice_owned(..), 0);

        if let Some((index_buffer, index_count)) = &self.indices {
            pass.bind_index_buffer(index_buffer.slice_owned(..), gpu::IndexType::U32);
            if let Some((indirect_buffer, indirect_count)) = &self.indirect {
                pass.draw_indexed_indirect_owned(indirect_buffer.clone(), 0, *indirect_count, std::mem::size_of::<gpu::DrawIndexedIndirectCommand>() as _);
            } else {
                pass.draw_indexed(0, *index_count, 0, 1, 0);
            }
        } else {
            if let Some((indirect_buffer, indirect_count)) = &self.indirect {
                pass.draw_indirect_owned(indirect_buffer.clone(), 0, *indirect_count, std::mem::size_of::<gpu::DrawIndirectCommand>() as _);
            } else {
                pass.draw(0, self.vertex_count, 0, 1);
            }
        }
    }

    /// Draw self by reference
    ///
    /// The instance buffer needs to be set first
    /// If the mesh has an indirect draw buffer that will be ignored
    pub fn draw_instanced_ref<'a>(
        &'a self,
        pass: &mut dyn crate::GraphicsPass<'a>,
        first_instance: u32,
        instance_count: u32,
    ) {
        pass.bind_vertex_buffer(self.vertex_buffer.slice_ref(..), 0);

        if let Some((index_buffer, index_count)) = &self.indices {
            pass.bind_index_buffer(index_buffer.slice_ref(..), gpu::IndexType::U32);
            pass.draw_indexed(0, *index_count, first_instance, instance_count, 0);
        } else {
            pass.draw(0, self.vertex_count, first_instance, instance_count);
        }
    }

    /// Draw self by clone
    ///
    /// The instance buffer needs to be set first
    /// If the mesh has an indirect draw buffer that will be ignored
    pub fn draw_instanced_owned<'a>(
        self,
        pass: &mut dyn crate::GraphicsPass<'a>,
        first_instance: u32,
        instance_count: u32,
    ) {
        pass.bind_vertex_buffer(self.vertex_buffer.slice_owned(..), 0);

        if let Some((index_buffer, index_count)) = &self.indices {
            pass.bind_index_buffer(index_buffer.slice_owned(..), gpu::IndexType::U32);
            pass.draw_indexed(0, *index_count, first_instance, instance_count, 0);
        } else {
            pass.draw(0, self.vertex_count, first_instance, instance_count);
        }
    }
}
