//! Mesh types and Vertex traits
//!
//! [`IndexedMesh`] and [`BasicMesh`] wrap vertex (and index buffers) and statically type the vertex that the mesh has as well as drawing operations
//!
//! The [`Vertex`] trait should be implemented by vertices, It allows different types of vertices to be used with the same pipeine as long as they have the correct attributes

pub mod vertex;

pub use vertex::*;

pub trait Mesh<V: Vertex> {
    /// Draw self by reference
    fn draw_ref<'a>(&'a self, pass: &mut dyn crate::GraphicsPass<'a>);

    /// Draw self by clone
    fn draw_owned<'a>(&self, pass: &mut dyn crate::GraphicsPass<'a>);

    /// Draw self by consuming
    fn draw_into(self, pass: &mut dyn crate::GraphicsPass<'_>);

    /// Draw self by reference
    /// 
    /// If using an instance buffer then that needs to be set first
    fn draw_instanced_ref<'a>(
        &'a self,
        pass: &mut dyn crate::GraphicsPass<'a>,
        first_instance: u32,
        instance_count: u32,
    );

    /// Draw self by clone
    /// 
    /// If using an instance buffer then that needs to be set first
    fn draw_instanced_owned<'a>(
        &self,
        pass: &mut dyn crate::GraphicsPass<'a>,
        first_instance: u32,
        instance_count: u32,
    );

    /// Draw self by consuming
    /// 
    /// If using an instance buffer then that needs to be set first
    fn draw_instanced_into(
        self,
        pass: &mut dyn crate::GraphicsPass<'_>,
        first_instance: u32,
        instance_count: u32,
    );
}

/// A mesh with indexing
///
/// Drawing this mesh instanced will bind the instance buffer to location 1
#[derive(Debug, Clone)]
pub struct IndexedMesh<V: Vertex> {
    /// vertex buffer, usage: COPY_SRC COPY_DST VERTEX
    pub vertex_buffer: gpu::Buffer,
    /// index buffer, usage: COPY_SRC COPY_DST INDEX
    pub index_buffer: gpu::Buffer,

    /// Marks the mesh so that the vertex state can be infered
    pub _vertex_marker: std::marker::PhantomData<V>,

    /// the number of indices in the index buffer
    pub index_count: u32,
    /// the number of vertices in the vertex buffer
    pub vertex_count: u32,
}

impl<V: Vertex> IndexedMesh<V> {
    /// Create a new Mesh
    ///
    /// The mesh won't be valid until the encoder is submitted
    pub fn new(
        encoder: &mut crate::CommandEncoder<'_>,
        device: &gpu::Device,
        vertices: &[V],
        indices: &[u32],
        name: Option<String>,
    ) -> Result<Self, gpu::Error> {
        Self::from_usage(
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
    pub fn from_usage(
        encoder: &mut crate::CommandEncoder<'_>,
        device: &gpu::Device,
        vertices: &[V],
        vertex_usage: gpu::BufferUsage,
        indices: &[u32],
        index_usage: gpu::BufferUsage,
        name: Option<String>,
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
            index_buffer,

            _vertex_marker: std::marker::PhantomData,

            vertex_count: vertices.len() as u32,
            index_count: indices.len() as u32,
        })
    }
}

impl<V: Vertex> Mesh<V> for IndexedMesh<V> {
    fn draw_ref<'a>(&'a self, pass: &mut dyn crate::GraphicsPass<'a>) {
        pass.bind_index_buffer(self.index_buffer.slice_ref(..), gpu::IndexType::U32);

        pass.bind_vertex_buffer(self.vertex_buffer.slice_ref(..), 0);

        pass.draw_indexed(0, self.index_count, 0, 1, 0);
    }

    fn draw_owned<'a>(&self, pass: &mut dyn crate::GraphicsPass<'a>) {
        pass.bind_index_buffer(self.index_buffer.slice_owned(..), gpu::IndexType::U32);

        pass.bind_vertex_buffer(self.vertex_buffer.slice_owned(..), 0);

        pass.draw_indexed(0, self.index_count, 0, 1, 0);
    }

    fn draw_into(self, pass: &mut dyn crate::GraphicsPass<'_>) {
        pass.bind_index_buffer(self.index_buffer.into_slice(..), gpu::IndexType::U32);

        pass.bind_vertex_buffer(self.vertex_buffer.into_slice(..), 0);

        pass.draw_indexed(0, self.index_count, 0, 1, 0);
    }

    fn draw_instanced_ref<'a>(
        &'a self,
        pass: &mut dyn crate::GraphicsPass<'a>,
        first_instance: u32,
        instance_count: u32,
    ) {
        pass.bind_index_buffer(self.index_buffer.slice_ref(..), gpu::IndexType::U32);

        pass.bind_vertex_buffer(self.vertex_buffer.slice_ref(..), 0);

        pass.draw_indexed(0, self.index_count, first_instance, instance_count, 0);
    }

    fn draw_instanced_owned<'a>(
        &self,
        pass: &mut dyn crate::GraphicsPass<'a>,
        first_instance: u32,
        instance_count: u32,
    ) {
        pass.bind_index_buffer(self.index_buffer.slice_owned(..), gpu::IndexType::U32);

        pass.bind_vertex_buffer(self.vertex_buffer.slice_owned(..), 0);

        pass.draw_indexed(0, self.index_count, first_instance, instance_count, 0);
    }

    fn draw_instanced_into(
        self,
        pass: &mut dyn crate::GraphicsPass<'_>,
        first_instance: u32,
        instance_count: u32,
    ) {
        pass.bind_index_buffer(self.index_buffer.into_slice(..), gpu::IndexType::U32);

        pass.bind_vertex_buffer(self.vertex_buffer.into_slice(..), 0);

        pass.draw_indexed(0, self.index_count, first_instance, instance_count, 0);
    }
}

/// A mesh without indexing
///
/// Drawing this mesh instanced will bind the instance buffer to location 1
#[derive(Debug, Clone)]
pub struct BasicMesh<V: Vertex> {
    /// vertex buffer, usage: COPY_SRC COPY_DST VERTEX
    pub vertex_buffer: gpu::Buffer,

    /// Marks the mesh so that the vertex state can be infered
    pub _vertex_marker: std::marker::PhantomData<V>,

    /// the number of vertices in the vertex buffer
    pub vertex_count: u32,
}

impl<V: Vertex> BasicMesh<V> {
    /// Create a new Mesh
    ///
    /// The mesh won't be valid until the encoder is submitted
    pub fn new(
        encoder: &mut crate::CommandEncoder<'_>,
        device: &gpu::Device,
        vertices: &[V],
        name: Option<String>,
    ) -> Result<Self, gpu::Error> {
        Self::from_usage(encoder, device, vertices, gpu::BufferUsage::empty(), name)
    }

    /// Create a new Mesh
    ///
    /// The mesh won't be valid until the encoder is submitted
    pub fn from_usage(
        encoder: &mut crate::CommandEncoder<'_>,
        device: &gpu::Device,
        vertices: &[V],
        vertex_usage: gpu::BufferUsage,
        name: Option<String>,
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

            _vertex_marker: std::marker::PhantomData,

            vertex_count: vertices.len() as u32,
        })
    }
}

impl<V: Vertex> Mesh<V> for BasicMesh<V> {
    fn draw_ref<'a>(&'a self, pass: &mut dyn crate::GraphicsPass<'a>) {
        pass.bind_vertex_buffer(self.vertex_buffer.slice_ref(..), 0);

        pass.draw(0, self.vertex_count, 0, 1);
    }

    fn draw_owned<'a>(&self, pass: &mut dyn crate::GraphicsPass<'a>) {
        pass.bind_vertex_buffer(self.vertex_buffer.slice_owned(..), 0);

        pass.draw(0, self.vertex_count, 0, 1);
    }

    fn draw_into(self, pass: &mut dyn crate::GraphicsPass<'_>) {
        pass.bind_vertex_buffer(self.vertex_buffer.into_slice(..), 0);

        pass.draw(0, self.vertex_count, 0, 1);
    }

    fn draw_instanced_ref<'a>(
        &'a self,
        pass: &mut dyn crate::GraphicsPass<'a>,
        first_instance: u32,
        instance_count: u32,
    ) {
        pass.bind_vertex_buffer(self.vertex_buffer.slice_ref(..), 0);

        pass.draw(0, self.vertex_count, first_instance, instance_count);
    }

    fn draw_instanced_owned<'a>(
        &self,
        pass: &mut dyn crate::GraphicsPass<'a>,
        first_instance: u32,
        instance_count: u32,
    ) {
        pass.bind_vertex_buffer(self.vertex_buffer.slice_owned(..), 0);

        pass.draw(0, self.vertex_count, first_instance, instance_count);
    }

    fn draw_instanced_into(
        self,
        pass: &mut dyn crate::GraphicsPass<'_>,
        first_instance: u32,
        instance_count: u32,
    ) {
        pass.bind_vertex_buffer(self.vertex_buffer.into_slice(..), 0);

        pass.draw(0, self.vertex_count, first_instance, instance_count);
    }
}
