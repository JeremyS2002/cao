//! Interoperability types for converting to-from vulkan types

use std::{borrow::Cow, num::NonZeroU32};

use ash::vk;
use std::ptr;

pub use vk::FormatFeatureFlags;
pub use vk::PhysicalDeviceLimits as DeviceLimits;
pub use vk::PhysicalDeviceMemoryProperties as MemoryProperties;
pub use vk::SampleCountFlags;

bitflags::bitflags! {
    /// Optional features that a device can support
    pub struct DeviceFeatures: u32 {
        /// Device supports graphics operations
        const GRAPHICS              = 0b000000000000000000000000001;
        /// Device supports compute operations
        const COMPUTE               = 0b000000000000000000000000010;
        /// Device supports transfer operations
        const TRANSFER              = 0b000000000000000000000000100;
        /// Allows use of tessellation shaders
        const TESSELLATION_SHADER   = 0b000000000000000000000001000;
        /// Allows use of geometry shaders
        const GEOMETRY_SHADER       = 0b000000000000000000000010000;
        /// Allows use of arrays of cube images
        const CUBE_TEXTURE_ARRAY    = 0b000000000000000000000100000;
        /// Allows drawing polygons as wireframe or point vertices
        const NON_SOLID             = 0b000000000000000000001000000;
        /// Allows use of lines of length other than 1
        const WIDE_LINES            = 0b000000000000000000010000000;
        /// Allows use of points of size other than 1
        const LARGE_POINTS          = 0b000000000000000000100000000;
        /// Allows use of atomic operations on buffers in vertex, tessellation and geometry shaders
        const VERTEX_ATOMICS        = 0b000000000000000001000000000;
        /// Allows use of atomic operations on buffers in fragment shaders
        const FRAGMENT_ATOMICS      = 0b000000000000000010000000000;
        /// Allows the use of anisotropic filtering in shaders
        const SAMPLER_ANISOTROPY    = 0b000000000000000100000000000;
        /// Allows multisampled images to be used as storage images
        const MULTISAMPLE_STORAGE   = 0b000000000000001000000000000;
        /// Allows usage of 64 bit variables in shaders
        const SHADER_64             = 0b000000000000010000000000000;
        /// Allows usage of 16 bit variables in shaders
        const SHADER_16             = 0b000000000000100000000000000;
        /// Allows the use of depth clamping
        /// (fragments outside the frustrum are clipped to max depth instead of being discarded)
        const DEPTH_CLAMP           = 0b000000000001000000000000000;
        /// Allows variable rate shading
        const VARIABLE_RATE_SHADING = 0b000000000010000000000000000;

        /// Device supports all types of operations
        const BASE = Self::GRAPHICS.bits | Self::COMPUTE.bits | Self::TRANSFER.bits;
    }
}

impl Into<vk::PhysicalDeviceFeatures> for DeviceFeatures {
    fn into(self) -> vk::PhysicalDeviceFeatures {
        vk::PhysicalDeviceFeatures {
            tessellation_shader: self.contains(DeviceFeatures::TESSELLATION_SHADER).into(),
            geometry_shader: self.contains(DeviceFeatures::GEOMETRY_SHADER).into(),
            image_cube_array: self.contains(DeviceFeatures::CUBE_TEXTURE_ARRAY).into(),
            wide_lines: self.contains(DeviceFeatures::WIDE_LINES).into(),
            large_points: self.contains(DeviceFeatures::LARGE_POINTS).into(),
            vertex_pipeline_stores_and_atomics: self
                .contains(DeviceFeatures::VERTEX_ATOMICS)
                .into(),
            fragment_stores_and_atomics: self.contains(DeviceFeatures::FRAGMENT_ATOMICS).into(),
            fill_mode_non_solid: self.contains(DeviceFeatures::NON_SOLID).into(),
            sampler_anisotropy: self.contains(DeviceFeatures::SAMPLER_ANISOTROPY).into(),
            shader_storage_image_multisample: self
                .contains(DeviceFeatures::MULTISAMPLE_STORAGE)
                .into(),
            shader_float64: self.contains(DeviceFeatures::SHADER_64).into(),
            shader_int64: self.contains(DeviceFeatures::SHADER_64).into(),
            shader_int16: self.contains(DeviceFeatures::SHADER_16).into(),
            depth_clamp: self.contains(DeviceFeatures::DEPTH_CLAMP).into(),
            sample_rate_shading: self.contains(DeviceFeatures::VARIABLE_RATE_SHADING).into(),
            shader_uniform_buffer_array_dynamic_indexing: vk::TRUE,
            shader_storage_buffer_array_dynamic_indexing: vk::TRUE,
            shader_storage_image_array_dynamic_indexing: vk::TRUE,
            ..Default::default()
        }
    }
}

/// Types of Physical devices
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum DeviceType {
    /// unknown
    Other,
    /// vulkan simulated on cpu
    Cpu,
    /// virtual gpu
    VirtualGpu,
    /// integrated gpu
    IntegratedGpu,
    /// discrete gpu
    DiscreteGpu,
}

/// Methods on how images can be presented to the screen
#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub enum PresentMode {
    /// do not wait for vertical blanking to update the current image
    /// This mode may result in visible tearing
    Immediate,
    /// wait for next blanking to update current image
    /// This mode will display no tearing
    /// if a new present request occurs while waiting for blanking the
    /// new image replaces the image already pending
    Mailbox,
    /// wait for the next blanking to update the current image
    /// This mode will display no tearing
    /// if a new present request occurs while waiting for blanking the
    /// new image will be added to a queue
    Fifo,
    /// generally wait for next blanking period unless one has been missed
    /// This mode may result in visible tearing
    FifoRelaxed,
    #[doc = "hidden"]
    /// force non complete pattern matching
    __NonCompleteDoNotUse,
}

impl Into<vk::PresentModeKHR> for PresentMode {
    fn into(self) -> vk::PresentModeKHR {
        match self {
            Self::Immediate => vk::PresentModeKHR::IMMEDIATE,
            Self::Mailbox => vk::PresentModeKHR::MAILBOX,
            Self::Fifo => vk::PresentModeKHR::FIFO,
            Self::FifoRelaxed => vk::PresentModeKHR::FIFO_RELAXED,
            _ => unreachable!("invalid form of present mode"),
        }
    }
}

impl From<vk::PresentModeKHR> for PresentMode {
    fn from(m: vk::PresentModeKHR) -> Self {
        match m {
            vk::PresentModeKHR::IMMEDIATE => Self::Immediate,
            vk::PresentModeKHR::MAILBOX => Self::Mailbox,
            vk::PresentModeKHR::FIFO => Self::Fifo,
            vk::PresentModeKHR::FIFO_RELAXED => Self::FifoRelaxed,
            _ => unreachable!("invalid form of present mode"),
        }
    }
}

/// An offset from the origin of a texture
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Offset3D {
    #[allow(missing_docs)]
    pub x: i32,
    #[allow(missing_docs)]
    pub y: i32,
    #[allow(missing_docs)]
    pub z: i32,
}

impl Offset3D {
    /// No offset
    pub const ZERO: Self = Self { x: 0, y: 0, z: 0 };
}

impl Into<vk::Offset3D> for Offset3D {
    fn into(self) -> vk::Offset3D {
        vk::Offset3D {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}

/// A 2d area
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Extent2D {
    #[allow(missing_docs)]
    pub width: u32,
    #[allow(missing_docs)]
    pub height: u32,
}

impl Extent2D {
    /// the min of self and other component-wise
    pub fn min(&mut self, other: &Self) {
        self.width = self.width.min(other.width);
        self.height = self.height.min(other.height);
    }

    /// the max of self and other component-wise
    pub fn max(&mut self, other: &Self) {
        self.width = self.width.max(other.width);
        self.height = self.height.max(other.height);
    }

    /// clamp self component-wise
    pub fn clamp(&mut self, min: &Self, max: &Self) {
        self.min(max);
        self.max(min);
    }
}

impl Into<vk::Extent2D> for Extent2D {
    fn into(self) -> vk::Extent2D {
        vk::Extent2D {
            width: self.width,
            height: self.height,
        }
    }
}

impl From<vk::Extent2D> for Extent2D {
    fn from(e: vk::Extent2D) -> Self {
        Self {
            width: e.width,
            height: e.height,
        }
    }
}

impl Into<Extent3D> for Extent2D {
    fn into(self) -> Extent3D {
        Extent3D {
            width: self.width,
            height: self.height,
            depth: 1,
        }
    }
}

/// A 3d volume
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Extent3D {
    #[allow(missing_docs)]
    pub width: u32,
    #[allow(missing_docs)]
    pub height: u32,
    #[allow(missing_docs)]
    pub depth: u32,
}

impl Extent3D {
    /// min of self and other component-wise
    pub fn min(&mut self, other: &Self) {
        self.width = self.width.min(other.width);
        self.height = self.height.min(other.height);
        self.depth = self.depth.min(other.depth);
    }

    /// max of set and other component-wise
    pub fn max(&mut self, other: &Self) {
        self.width = self.width.max(other.width);
        self.height = self.height.max(other.height);
        self.depth = self.depth.max(other.depth);
    }

    /// clamp self component-wise
    pub fn clamp(&mut self, min: &Self, max: &Self) {
        self.min(max);
        self.max(min);
    }
}

impl Into<vk::Extent3D> for Extent3D {
    fn into(self) -> vk::Extent3D {
        vk::Extent3D {
            width: self.width,
            height: self.height,
            depth: self.depth,
        }
    }
}

impl From<vk::Extent3D> for Extent3D {
    fn from(e: vk::Extent3D) -> Self {
        Self {
            width: e.width,
            height: e.height,
            depth: e.depth,
        }
    }
}

impl Into<Extent2D> for Extent3D {
    fn into(self) -> Extent2D {
        Extent2D {
            width: self.width,
            height: self.height,
        }
    }
}

impl std::ops::Add<Extent3D> for Offset3D {
    type Output = Offset3D;

    fn add(self, other: Extent3D) -> Self::Output {
        Self::Output {
            x: self.x + other.width as i32,
            y: self.y + other.height as i32,
            z: self.z + other.depth as i32,
        }
    }
}

impl std::ops::Sub<Extent3D> for Offset3D {
    type Output = Offset3D;

    fn sub(self, other: Extent3D) -> Self::Output {
        Self::Output {
            x: self.x - other.width as i32,
            y: self.y - other.height as i32,
            z: self.z - other.depth as i32,
        }
    }
}

impl std::ops::AddAssign<Extent3D> for Offset3D {
    fn add_assign(&mut self, other: Extent3D) {
        *self = Self {
            x: self.x + other.width as i32,
            y: self.y + other.height as i32,
            z: self.z + other.depth as i32,
        }
    }
}

impl std::ops::SubAssign<Extent3D> for Offset3D {
    fn sub_assign(&mut self, other: Extent3D) {
        *self = Self {
            x: self.x - other.width as i32,
            y: self.y - other.height as i32,
            z: self.z - other.depth as i32,
        }
    }
}

bitflags::bitflags! {
    /// ShaderStage bitflags used for situations where objects can be in multiple stages
    pub struct ShaderStages: u32 {
        #[allow(missing_docs)]
        const VERTEX               = 0b0000000000001;
        #[allow(missing_docs)]
        const TESSELLATION_CONTROL = 0b0000000000010;
        #[allow(missing_docs)]
        const TESSELLATION_EVAL    = 0b0000000000100;
        #[allow(missing_docs)]
        const GEOMETRY             = 0b0000000001000;
        #[allow(missing_docs)]
        const FRAGMENT             = 0b0000000010000;
        #[allow(missing_docs)]
        const COMPUTE              = 0b0000000100000;
        #[cfg(feature = "mesh")]
        #[allow(missing_docs)]
        const TASK                 = 0b0000001000000;
        #[cfg(feature = "mesh")]
        #[allow(missing_docs)]
        const MESH                 = 0b0000010000000;
        #[cfg(feature = "ray")]
        #[allow(missing_docs)]
        const RAY_GEN              = 0b0000100000000;
        #[cfg(feature = "ray")]
        #[allow(missing_docs)]
        const RAY_HIT              = 0b0001000000000;
        #[cfg(feature = "ray")]
        #[allow(missing_docs)]
        const RAY_CLOSEST          = 0b0010000000000;
        #[cfg(feature = "ray")]
        #[allow(missing_docs)]
        const RAY_MISS             = 0b0100000000000;
        #[cfg(feature = "ray")]
        #[allow(missing_docs)]
        const RAY_INTERSECTION     = 0b1000000000000;
    }
}

impl Into<vk::ShaderStageFlags> for ShaderStages {
    fn into(self) -> vk::ShaderStageFlags {
        let mut result = vk::ShaderStageFlags::empty();
        if self.contains(Self::VERTEX) {
            result |= vk::ShaderStageFlags::VERTEX;
        }
        if self.contains(Self::TESSELLATION_CONTROL) {
            result |= vk::ShaderStageFlags::TESSELLATION_CONTROL;
        }
        if self.contains(Self::TESSELLATION_EVAL) {
            result |= vk::ShaderStageFlags::TESSELLATION_EVALUATION;
        }
        if self.contains(Self::GEOMETRY) {
            result |= vk::ShaderStageFlags::GEOMETRY;
        }
        if self.contains(Self::FRAGMENT) {
            result |= vk::ShaderStageFlags::FRAGMENT;
        }
        if self.contains(Self::COMPUTE) {
            result |= vk::ShaderStageFlags::COMPUTE;
        }
        #[cfg(feature = "mesh")]
        if self.contains(Self::TASK) {
            result |= vk::ShaderStageFlags::TASK_NV;
        }
        #[cfg(feature = "mesh")]
        if self.contains(Self::MESH) {
            result |= vk::ShaderStageFlags::MESH_NV;
        }
        #[cfg(feature = "ray")]
        if self.contains(Self::RAY_GEN) {
            result |= vk::ShaderStageFlags::RAYGEN_KHR;
        }
        #[cfg(feature = "ray")]
        if self.contains(Self::RAY_HIT) {
            result |= vk::ShaderStageFlags::RAYGEN_KHR;
        }
        #[cfg(feature = "ray")]
        if self.contains(Self::RAY_CLOSEST) {
            result |= vk::ShaderStageFlags::CLOSEST_HIT_KHR;
        }
        #[cfg(feature = "ray")]
        if self.contains(Self::RAY_MISS) {
            result |= vk::ShaderStageFlags::MISS_KHR;
        }
        #[cfg(feature = "ray")]
        if self.contains(Self::RAY_INTERSECTION) {
            result |= vk::ShaderStageFlags::INTERSECTION_KHR;
        }
        result
    }
}

impl Into<PipelineStageFlags> for ShaderStages {
    fn into(self) -> PipelineStageFlags {
        let mut result = PipelineStageFlags::empty();
        if self.contains(Self::VERTEX) {
            result |= PipelineStageFlags::VERTEX_SHADER;
        }
        if self.contains(Self::TESSELLATION_CONTROL) {
            result |= PipelineStageFlags::TESSELLATION_CONTROL;
        }
        if self.contains(Self::TESSELLATION_EVAL) {
            result |= PipelineStageFlags::TESSELLATION_EVAL;
        }
        if self.contains(Self::GEOMETRY) {
            result |= PipelineStageFlags::GEOMETRY;
        }
        if self.contains(Self::FRAGMENT) {
            result |= PipelineStageFlags::FRAGMENT;
        }
        if self.contains(Self::COMPUTE) {
            result |= PipelineStageFlags::COMPUTE;
        }
        #[cfg(feature = "mesh")]
        if self.contains(Self::TASK) {
            result |= PipelineStageFlags::TASK_SHADER;
        }
        #[cfg(feature = "mesh")]
        if self.contains(Self::MESH) {
            result |= PipelineStageFlags::MESH_SHADER;
        }
        #[cfg(feature = "ray")]
        if self.contains(Self::RAY_GEN) {
            result |= PipelineStageFlags::RAY_SHADER;
        }
        #[cfg(feature = "ray")]
        if self.contains(Self::RAY_HIT) {
            result |= PipelineStageFlags::RAY_SHADER;
        }
        #[cfg(feature = "ray")]
        if self.contains(Self::RAY_CLOSEST) {
            result |= PipelineStageFlags::CLOSEST_HIT;
        }
        #[cfg(feature = "ray")]
        if self.contains(Self::RAY_MISS) {
            result |= PipelineStageFlags::MISS_KHR;
        }
        #[cfg(feature = "ray")]
        if self.contains(Self::RAY_INTERSECTION) {
            result |= PipelineStageFlags::INTERSECTION_KHR;
        }
        result
    }
}

/// Describes push constants used in shaders
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct PushConstantRange {
    /// The stage that the push constants are used
    pub stage: crate::ShaderStages,
    /// the offset of the push constants
    pub offset: u32,
    /// the size of the push constants
    pub size: u32,
}

impl Into<vk::PushConstantRange> for PushConstantRange {
    fn into(self) -> vk::PushConstantRange {
        vk::PushConstantRange {
            stage_flags: self.stage.into(),
            offset: self.offset,
            size: self.size,
        }
    }
}

/// The stage in a command pipeline
#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub enum PipelineStage {
    /// Before any commands are processed
    TopOfPipe,
    /// Draw/DispatchIndirect command
    DrawIndirect,
    /// Vertex shader input
    VertexInput,
    /// Vertex shader
    VertexShader,
    /// tessellation control
    TessellationControl,
    /// tessellation evaluation
    TessellationEval,
    /// Geometry shader
    Geometry,
    /// Fragment shader
    Fragment,
    /// early Depth/stencil testing
    DepthStencilEarly,
    /// late Depth/stencil testing
    DepthStencilLate,
    /// outputting color
    ColorOutput,
    /// Compute shader
    Compute,
    /// Copy operations
    Copy,
    /// after commands have completed
    BottomOfPipe,
    /// after all graphics pipeline stages,
    AllGraphics,
    /// after all commands have completed
    AllCommands,
    /// closest hit shader
    #[cfg(feature = "ray")]
    ClosestHit,
    /// miss shader
    #[cfg(feature = "ray")]
    Miss,
    /// ray shader
    #[cfg(feature = "ray")]
    RayShader,
    /// Build acceleration structure
    #[cfg(feature = "ray")]
    AccelerationBuild,
    /// Task shader
    #[cfg(feature = "mesh")]
    TaskShader,
    /// Mesh shader
    #[cfg(feature = "mesh")]
    MeshShader,
    #[doc = "hidden"]
    __NonCompleteDoNotUse,
}

impl Into<vk::PipelineStageFlags> for PipelineStage {
    fn into(self) -> vk::PipelineStageFlags {
        match self {
            Self::TopOfPipe => vk::PipelineStageFlags::TOP_OF_PIPE,
            Self::DrawIndirect => vk::PipelineStageFlags::DRAW_INDIRECT,
            Self::VertexInput => vk::PipelineStageFlags::VERTEX_INPUT,
            Self::VertexShader => vk::PipelineStageFlags::VERTEX_SHADER,
            Self::TessellationControl => vk::PipelineStageFlags::TESSELLATION_CONTROL_SHADER,
            Self::TessellationEval => vk::PipelineStageFlags::TESSELLATION_EVALUATION_SHADER,
            Self::Geometry => vk::PipelineStageFlags::GEOMETRY_SHADER,
            Self::Fragment => vk::PipelineStageFlags::FRAGMENT_SHADER,
            Self::DepthStencilEarly => vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            Self::DepthStencilLate => vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
            Self::ColorOutput => vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            Self::Compute => vk::PipelineStageFlags::COMPUTE_SHADER,
            Self::Copy => vk::PipelineStageFlags::TRANSFER,
            Self::BottomOfPipe => vk::PipelineStageFlags::BOTTOM_OF_PIPE,
            Self::AllGraphics => vk::PipelineStageFlags::ALL_GRAPHICS,
            Self::AllCommands => vk::PipelineStageFlags::ALL_COMMANDS,
            Self::__NonCompleteDoNotUse => vk::PipelineStageFlags::empty(),
        }
    }
}

impl From<vk::PipelineStageFlags> for PipelineStage {
    fn from(p: vk::PipelineStageFlags) -> Self {
        match p {
            vk::PipelineStageFlags::TOP_OF_PIPE => Self::TopOfPipe,
            vk::PipelineStageFlags::DRAW_INDIRECT => Self::DrawIndirect,
            vk::PipelineStageFlags::VERTEX_INPUT => Self::VertexInput,
            vk::PipelineStageFlags::VERTEX_SHADER => Self::VertexShader,
            vk::PipelineStageFlags::TESSELLATION_CONTROL_SHADER => Self::TessellationControl,
            vk::PipelineStageFlags::TESSELLATION_EVALUATION_SHADER => Self::TessellationEval,
            vk::PipelineStageFlags::GEOMETRY_SHADER => Self::Geometry,
            vk::PipelineStageFlags::FRAGMENT_SHADER => Self::Fragment,
            vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS => Self::DepthStencilEarly,
            vk::PipelineStageFlags::LATE_FRAGMENT_TESTS => Self::DepthStencilLate,
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT => Self::ColorOutput,
            vk::PipelineStageFlags::COMPUTE_SHADER => Self::Compute,
            vk::PipelineStageFlags::TRANSFER => Self::Copy,
            vk::PipelineStageFlags::BOTTOM_OF_PIPE => Self::BottomOfPipe,
            vk::PipelineStageFlags::ALL_GRAPHICS => Self::AllGraphics,
            vk::PipelineStageFlags::ALL_COMMANDS => Self::AllCommands,
            _ => Self::AllCommands,
        }
    }
}

bitflags::bitflags! {
    /// The stage in a command pipeline
    pub struct PipelineStageFlags: u32 {
        /// Before any commands are processed
        const TOP_OF_PIPE              = 0b000000000000000000001;
        /// Draw/DispatchIndirect command
        const DRAW_INDIRECT            = 0b000000000000000000010;
        /// Vertex shader input
        const VERTEX_INPUT             = 0b000000000000000000100;
        /// Vertex shader
        const VERTEX_SHADER            = 0b000000000000000001000;
        /// tessellation control
        const TESSELLATION_CONTROL     = 0b000000000000000010000;
        /// tessellation evaluation
        const TESSELLATION_EVAL        = 0b000000000000000100000;
        /// Geometry shader
        const GEOMETRY                 = 0b000000000000001000000;
        /// Fragment shader
        const FRAGMENT                 = 0b000000000000010000000;
        /// early Depth/stencil testing
        const DEPTH_STENCIL_EARLY      = 0b000000000000100000000;
        /// late Depth/stencil testing
        const DEPTH_STENCIL_LATE       = 0b000000000001000000000;
        /// outputting color
        const COLOR_OUTPUT             = 0b000000000010000000000;
        /// Compute shader
        const COMPUTE                  = 0b000000000100000000000;
        /// Copy operations
        const COPY                     = 0b000000001000000000000;
        /// after commands have completed
        const BOTTOM_OF_PIPE           = 0b000000010000000000000;
        /// after all graphics pipeline stages
        const ALL_GRAPHICS             = 0b000000100000000000000;
        /// after all commands have completed
        const ALL_COMMANDS             = 0b000001000000000000000;
        /// closest hit shader
        #[cfg(feature = "raw")]
        const CLOSEST_HIT              = 0b000010000000000000000;
        /// ray shader
        #[cfg(feature = "ray")]
        const RAY_SHADER               = 0b000100000000000000000;
        /// Build acceleration structure
        #[cfg(feature = "ray")]
        const ACCELERATION_BUILD       = 0b001000000000000000000;
        /// Task shader
        #[cfg(feature = "mesh")]
        const TASK_SHADER              = 0b010000000000000000000;
        /// Mesh shader
        #[cfg(feature = "mesh")]
        const MESH_SHADER              = 0b100000000000000000000;
    }
}

impl Into<vk::PipelineStageFlags> for PipelineStageFlags {
    fn into(self) -> vk::PipelineStageFlags {
        let mut result = vk::PipelineStageFlags::empty();
        if self.contains(Self::TOP_OF_PIPE) {
            result |= vk::PipelineStageFlags::TOP_OF_PIPE
        }
        if self.contains(Self::DRAW_INDIRECT) {
            result |= vk::PipelineStageFlags::DRAW_INDIRECT
        }
        if self.contains(Self::VERTEX_INPUT) {
            result |= vk::PipelineStageFlags::VERTEX_INPUT
        }
        if self.contains(Self::VERTEX_SHADER) {
            result |= vk::PipelineStageFlags::VERTEX_SHADER
        }
        if self.contains(Self::TESSELLATION_CONTROL) {
            result |= vk::PipelineStageFlags::TESSELLATION_CONTROL_SHADER
        }
        if self.contains(Self::TESSELLATION_EVAL) {
            result |= vk::PipelineStageFlags::TESSELLATION_EVALUATION_SHADER
        }
        if self.contains(Self::GEOMETRY) {
            result |= vk::PipelineStageFlags::GEOMETRY_SHADER
        }
        if self.contains(Self::FRAGMENT) {
            result |= vk::PipelineStageFlags::FRAGMENT_SHADER
        }
        if self.contains(Self::DEPTH_STENCIL_EARLY) {
            result |= vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS
        }
        if self.contains(Self::DEPTH_STENCIL_LATE) {
            result |= vk::PipelineStageFlags::LATE_FRAGMENT_TESTS
        }
        if self.contains(Self::COLOR_OUTPUT) {
            result |= vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
        }
        if self.contains(Self::COMPUTE) {
            result |= vk::PipelineStageFlags::COMPUTE_SHADER
        }
        if self.contains(Self::COPY) {
            result |= vk::PipelineStageFlags::TRANSFER
        }
        if self.contains(Self::BOTTOM_OF_PIPE) {
            result |= vk::PipelineStageFlags::BOTTOM_OF_PIPE
        }
        if self.contains(Self::ALL_GRAPHICS) {
            result |= vk::PipelineStageFlags::ALL_GRAPHICS
        }
        if self.contains(Self::ALL_COMMANDS) {
            result |= vk::PipelineStageFlags::ALL_COMMANDS
        }
        result
    }
}

impl From<vk::PipelineStageFlags> for PipelineStageFlags {
    fn from(p: vk::PipelineStageFlags) -> Self {
        let mut result = Self::empty();
        if p.contains(vk::PipelineStageFlags::TOP_OF_PIPE) {
            result |= Self::TOP_OF_PIPE;
        }
        if p.contains(vk::PipelineStageFlags::DRAW_INDIRECT) {
            result |= Self::DRAW_INDIRECT;
        }
        if p.contains(vk::PipelineStageFlags::VERTEX_INPUT) {
            result |= Self::VERTEX_INPUT;
        }
        if p.contains(vk::PipelineStageFlags::VERTEX_SHADER) {
            result |= Self::VERTEX_SHADER;
        }
        if p.contains(vk::PipelineStageFlags::TESSELLATION_CONTROL_SHADER) {
            result |= Self::TESSELLATION_CONTROL;
        }
        if p.contains(vk::PipelineStageFlags::TESSELLATION_EVALUATION_SHADER) {
            result |= Self::TESSELLATION_EVAL;
        }
        if p.contains(vk::PipelineStageFlags::GEOMETRY_SHADER) {
            result |= Self::GEOMETRY;
        }
        if p.contains(vk::PipelineStageFlags::FRAGMENT_SHADER) {
            result |= Self::FRAGMENT;
        }
        if p.contains(vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS) {
            result |= Self::DEPTH_STENCIL_EARLY;
        }
        if p.contains(vk::PipelineStageFlags::LATE_FRAGMENT_TESTS) {
            result |= Self::DEPTH_STENCIL_LATE;
        }
        if p.contains(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT) {
            result |= Self::COLOR_OUTPUT;
        }
        if p.contains(vk::PipelineStageFlags::COMPUTE_SHADER) {
            result |= Self::COMPUTE;
        }
        if p.contains(vk::PipelineStageFlags::TRANSFER) {
            result |= Self::COPY;
        }
        if p.contains(vk::PipelineStageFlags::BOTTOM_OF_PIPE) {
            result |= Self::BOTTOM_OF_PIPE;
        }
        if p.contains(vk::PipelineStageFlags::ALL_GRAPHICS) {
            result |= Self::ALL_GRAPHICS;
        }
        if p.contains(vk::PipelineStageFlags::ALL_COMMANDS) {
            result |= Self::ALL_COMMANDS;
        }
        result
    }
}

/// Decides how fragments are generated from polygons
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum PolygonMode {
    /// fill the area of polygons with fragments
    Fill,
    /// edges are drawn as lines, using this requires
    /// enabling DeviceFeatures::NonSolid
    Line,
    /// vertices are drawn as points, using this requires
    /// enabling DeviceFeatures::NonSolid
    Point,
}

impl Into<vk::PolygonMode> for PolygonMode {
    fn into(self) -> vk::PolygonMode {
        match self {
            Self::Fill => vk::PolygonMode::FILL,
            Self::Line => vk::PolygonMode::LINE,
            Self::Point => vk::PolygonMode::POINT,
        }
    }
}

/// Decides what the rasterizer calls the front face of a triangle
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum FrontFace {
    /// clockwise vertices are the front
    Clockwise,
    /// counter-clockwise vertices are the front
    CounterClockwise,
}

impl Into<vk::FrontFace> for FrontFace {
    fn into(self) -> vk::FrontFace {
        match self {
            Self::Clockwise => vk::FrontFace::CLOCKWISE,
            Self::CounterClockwise => vk::FrontFace::COUNTER_CLOCKWISE,
        }
    }
}

/// Decides if face culling should be used
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum CullFace {
    /// front faces won't be rendered
    Front,
    /// back faces won't be rendered
    Back,
    /// both front and back faces won't be rendered
    FrontAndBack,
    /// all faces will be rendered
    None,
}

impl Into<vk::CullModeFlags> for CullFace {
    fn into(self) -> vk::CullModeFlags {
        match self {
            Self::Front => vk::CullModeFlags::FRONT,
            Self::Back => vk::CullModeFlags::BACK,
            Self::FrontAndBack => vk::CullModeFlags::FRONT_AND_BACK,
            Self::None => vk::CullModeFlags::NONE,
        }
    }
}

/// Part of the fixed functions in vulkan, this controls how the rasterization process occurs
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Rasterizer {
    /// if this is enabled then fragments that would be discarded because
    /// they are outside the frustrum will be clipped to the max depth
    /// using this requires enabling DeviceFeatures::D
    pub depth_clamp: bool,
    /// how to render assembled polygons
    pub polygon_mode: PolygonMode,
    /// the type
    pub primitive_topology: PrimitiveTopology,
    /// what is the front of a triangle
    pub front_face: FrontFace,
    /// what side of faces should be culled
    pub cull_face: CullFace,
    /// the width of lines, any value other than 1.0 needs DeviceFeatures::WIDE_LINES enabled
    pub line_width: f32,
    /// the rasterizer can alter the depth values by adding a constant value or biasing them based on
    /// fragment slope. this can be used for shadow mapping
    pub depth_bias: bool,
    /// the constant that can be added to fragment depth values
    pub depth_bias_constant: f32,
    /// the slope factor that can influence fragment depth values
    pub depth_bias_slope: f32,
}

impl Into<vk::PipelineRasterizationStateCreateInfo> for Rasterizer {
    fn into(self) -> vk::PipelineRasterizationStateCreateInfo {
        vk::PipelineRasterizationStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineRasterizationStateCreateFlags::empty(),
            depth_clamp_enable: if self.depth_clamp {
                vk::TRUE
            } else {
                vk::FALSE
            },
            cull_mode: self.cull_face.into(),
            front_face: self.front_face.into(),
            line_width: self.line_width,
            polygon_mode: self.polygon_mode.into(),
            depth_bias_enable: if self.depth_bias { vk::TRUE } else { vk::FALSE },
            depth_bias_constant_factor: self.depth_bias_constant,
            depth_bias_slope_factor: self.depth_bias_slope,
            ..Default::default()
        }
    }
}

impl Default for Rasterizer {
    fn default() -> Self {
        Self {
            depth_clamp: false,
            polygon_mode: PolygonMode::Fill,
            primitive_topology: PrimitiveTopology::TriangleList,
            front_face: FrontFace::Clockwise,
            cull_face: CullFace::None,
            line_width: 1.0,
            depth_bias: false,
            depth_bias_constant: 0.0,
            depth_bias_slope: 0.0,
        }
    }
}

/// when blending what should the color be multiplied by
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum BlendFactor {
    /// rgb factors (0, 0, 0)                alpha factor 0
    Zero,
    /// rgb factors (1, 1, 1)                alpha factor 1
    One,
    /// rgb factors (Sr, Sg, Sb)             alpha factor Sa
    SrcColor,
    /// rgb factors (1-Sr, 1-Sg, 1-Sb)       alpha factor 1-Sa
    OneMinusSrcColor,
    /// rgb factors (Sa, Sa, Sa)             alpha factor Sa
    SrcAlpha,
    /// rgb factors (1-Sa, 1-Sa, 1-Sa)       alpha factor 1-Sa
    OneMinusSrcAlpha,
    /// rgb factors (Dr, Dg, Db)             alpha factor Da
    DstColor,
    /// rgb factors (1-Dr, 1-Dg, 1-Db)       alpha factor 1-Da
    OneMinusDstColor,
    /// rgb factors (Da, Da, Da)             alpha factor Da
    DstAlpha,
    /// rgb factors (1-Da, 1-Da, 1-Da)       alpha factor 1-Da
    OneMinusDstAlpha,
}

impl Into<vk::BlendFactor> for BlendFactor {
    fn into(self) -> vk::BlendFactor {
        match self {
            Self::Zero => vk::BlendFactor::ZERO,
            Self::One => vk::BlendFactor::ONE,
            Self::SrcColor => vk::BlendFactor::SRC_COLOR,
            Self::OneMinusSrcColor => vk::BlendFactor::ONE_MINUS_SRC_COLOR,
            Self::SrcAlpha => vk::BlendFactor::SRC_ALPHA,
            Self::OneMinusSrcAlpha => vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
            Self::DstColor => vk::BlendFactor::DST_COLOR,
            Self::OneMinusDstColor => vk::BlendFactor::ONE_MINUS_DST_COLOR,
            Self::DstAlpha => vk::BlendFactor::DST_ALPHA,
            Self::OneMinusDstAlpha => vk::BlendFactor::ONE_MINUS_DST_ALPHA,
        }
    }
}

/// how the colors should be blended together
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum BlendOp {
    /// Add the components together
    Add,
    /// subtract dst from src
    Sub,
    /// subtract src from dst
    ReverseSub,
    /// min of src / dst
    Min,
    /// max of src / dst
    Max,
}

impl Into<vk::BlendOp> for BlendOp {
    fn into(self) -> vk::BlendOp {
        match self {
            Self::Add => vk::BlendOp::ADD,
            Self::Sub => vk::BlendOp::SUBTRACT,
            Self::ReverseSub => vk::BlendOp::REVERSE_SUBTRACT,
            Self::Min => vk::BlendOp::MIN,
            Self::Max => vk::BlendOp::MAX,
        }
    }
}

bitflags::bitflags! {
    /// Decides what components of the pixel should be output to
    pub struct ColorMask: u8 {
        /// the r value should be written to
        const R = 0b0001;
        /// the g vaule should be written to
        const G = 0b0010;
        /// the b value should be written to
        const B = 0b0100;
        /// the a value should be written to
        const A = 0b1000;
    }
}

impl Into<vk::ColorComponentFlags> for ColorMask {
    fn into(self) -> vk::ColorComponentFlags {
        let mut res = vk::ColorComponentFlags::empty();
        if self.contains(ColorMask::R) {
            res |= vk::ColorComponentFlags::R;
        }
        if self.contains(ColorMask::G) {
            res |= vk::ColorComponentFlags::G;
        }
        if self.contains(ColorMask::B) {
            res |= vk::ColorComponentFlags::B;
        }
        if self.contains(ColorMask::A) {
            res |= vk::ColorComponentFlags::A;
        }
        res
    }
}

/// how a single attachment to a render pass should blend src and dst
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct BlendState {
    /// if blending is enabled
    pub blend_enable: bool,
    /// what componets of pixels should be output to
    pub write_mask: ColorMask,
    /// how the src should factor into the blend
    pub src_blend: BlendFactor,
    /// how the dst should factor into the blend
    pub dst_blend: BlendFactor,
    /// how the src and dst should be combined
    pub blend_op: BlendOp,
    /// how the src alpha component should factor into the blend
    pub src_alpha_blend: BlendFactor,
    /// how the dst alpha component should factor into the blend
    pub dst_alpha_blend: BlendFactor,
    /// how the src and dst alpha components should e combined
    pub alpha_blend_op: BlendOp,
}

impl BlendState {
    /// Alpha blending
    pub const ALPHA: Self = Self {
        blend_enable: true,
        write_mask: ColorMask::all(),
        src_blend: BlendFactor::SrcAlpha,
        dst_blend: BlendFactor::OneMinusSrcAlpha,
        blend_op: BlendOp::Add,
        src_alpha_blend: BlendFactor::One,
        dst_alpha_blend: BlendFactor::Zero,
        alpha_blend_op: BlendOp::Add,
    };

    /// Alpha blending based on the dst
    pub const DST_ALPHA: Self = Self {
        blend_enable: true,
        write_mask: ColorMask::all(),
        src_blend: BlendFactor::OneMinusDstAlpha,
        dst_blend: BlendFactor::DstAlpha,
        blend_op: BlendOp::Add,
        src_alpha_blend: BlendFactor::Zero,
        dst_alpha_blend: BlendFactor::One,
        alpha_blend_op: BlendOp::Add,
    };

    /// Replace with the new color
    pub const REPLACE: Self = Self {
        blend_enable: false,
        write_mask: ColorMask::all(),
        src_blend: BlendFactor::One,
        dst_blend: BlendFactor::Zero,
        blend_op: BlendOp::Add,
        src_alpha_blend: BlendFactor::One,
        dst_alpha_blend: BlendFactor::Zero,
        alpha_blend_op: BlendOp::Add,
    };

    /// Add the colors together
    pub const ADD: Self = Self {
        blend_enable: true,
        write_mask: ColorMask::all(),
        src_blend: BlendFactor::One,
        dst_blend: BlendFactor::One,
        blend_op: BlendOp::Add,
        src_alpha_blend: BlendFactor::One,
        dst_alpha_blend: BlendFactor::One,
        alpha_blend_op: BlendOp::Add,
    };

    /// Subtract the src from the dst
    pub const SUB: Self = Self {
        blend_enable: true,
        write_mask: ColorMask::all(),
        src_blend: BlendFactor::One,
        dst_blend: BlendFactor::One,
        blend_op: BlendOp::Sub,
        src_alpha_blend: BlendFactor::One,
        dst_alpha_blend: BlendFactor::One,
        alpha_blend_op: BlendOp::Sub,
    };

    /// Multiply the src and dst together
    pub const MUL: Self = Self {
        blend_enable: true,
        write_mask: ColorMask::all(),
        src_blend: BlendFactor::DstColor,
        dst_blend: BlendFactor::Zero,
        blend_op: BlendOp::Add,
        src_alpha_blend: BlendFactor::DstAlpha,
        dst_alpha_blend: BlendFactor::Zero,
        alpha_blend_op: BlendOp::Add,
    };
}

impl Into<vk::PipelineColorBlendAttachmentState> for BlendState {
    fn into(self) -> vk::PipelineColorBlendAttachmentState {
        vk::PipelineColorBlendAttachmentState {
            blend_enable: if self.blend_enable {
                vk::TRUE
            } else {
                vk::FALSE
            },
            color_write_mask: self.write_mask.into(),
            src_color_blend_factor: self.src_blend.into(),
            dst_color_blend_factor: self.dst_blend.into(),
            color_blend_op: self.blend_op.into(),
            src_alpha_blend_factor: self.src_alpha_blend.into(),
            dst_alpha_blend_factor: self.dst_alpha_blend.into(),
            alpha_blend_op: self.alpha_blend_op.into(),
        }
    }
}

impl Default for BlendState {
    fn default() -> Self {
        Self {
            blend_enable: true,
            write_mask: ColorMask::all(),
            src_blend: BlendFactor::One,
            dst_blend: BlendFactor::Zero,
            blend_op: BlendOp::Add,
            src_alpha_blend: BlendFactor::One,
            dst_alpha_blend: BlendFactor::Zero,
            alpha_blend_op: BlendOp::Add,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Viewport {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub min_depth: f32,
    pub max_depth: f32,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            min_depth: 0.0,
            max_depth: 1.0,
        }
    }
}

impl Into<vk::Viewport> for Viewport {
    fn into(self) -> vk::Viewport {
        vk::Viewport {
            x: self.x as f32,
            y: self.y as f32,
            width: self.width as f32,
            height: self.height as f32,
            min_depth: self.min_depth,
            max_depth: self.max_depth,
        }
    }
}

/// Decides how verties should be interpreted
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum PrimitiveTopology {
    /// every vertex represents a single point
    PointList,
    /// every two vertices represent part of a line without reuse
    LineList,
    /// the end of each line segment is used as the start of the next
    LineStrip,
    /// every three vertices represent a triangle
    TriangleList,
    /// the second and third vertices of every triangle are used as the first two
    /// in the next triangle
    TriangleStrip,
    /// series of connected triangle primitives all sharing a common vertex
    TriangleFan,
}

impl Into<vk::PrimitiveTopology> for PrimitiveTopology {
    fn into(self) -> vk::PrimitiveTopology {
        match self {
            Self::PointList => vk::PrimitiveTopology::POINT_LIST,
            Self::LineList => vk::PrimitiveTopology::LINE_LIST,
            Self::LineStrip => vk::PrimitiveTopology::LINE_STRIP,
            Self::TriangleList => vk::PrimitiveTopology::TRIANGLE_LIST,
            Self::TriangleStrip => vk::PrimitiveTopology::TRIANGLE_STRIP,
            Self::TriangleFan => vk::PrimitiveTopology::TRIANGLE_FAN,
        }
    }
}

/// how the vertex buffer should be processed
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VertexInputRate {
    /// Update the buffer data for all vertices
    Vertex,
    /// Update the buffer once for all verteices
    Instance,
}

impl Into<vk::VertexInputRate> for VertexInputRate {
    fn into(self) -> vk::VertexInputRate {
        match self {
            Self::Vertex => vk::VertexInputRate::VERTEX,
            Self::Instance => vk::VertexInputRate::INSTANCE,
        }
    }
}

/// the format of an element in a vertex buffer
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VertexFormat {
    /// Input a float
    ///
    /// In glsl looks like `layout(location = _) in float in_f;
    Float1,
    /// Input a Vec2
    ///
    /// In glsl looks like `layout(location = _) in vec2 in_v2;
    Vec2,
    /// Input a vec3
    ///
    /// In glsl looks like `layout(location = _) in vec3 in_v3;
    Vec3,
    /// Input a vec4
    ///
    /// In glsl looks like `layout(location = _) in vec4 in_v4;
    Vec4,
}

impl Into<vk::Format> for VertexFormat {
    fn into(self) -> vk::Format {
        match self {
            Self::Float1 => vk::Format::R32_SFLOAT,
            Self::Vec2 => vk::Format::R32G32_SFLOAT,
            Self::Vec3 => vk::Format::R32G32B32_SFLOAT,
            Self::Vec4 => vk::Format::R32G32B32A32_SFLOAT,
        }
    }
}

/// One attribute in a vertex buffer
///
/// Represented in glsl by
/// layout(location = <location>) in <format> in_name;
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct VertexAttribute {
    /// the location in the shader
    pub location: u32,
    /// the format of the element
    pub format: VertexFormat,
    /// the offset from the start of the buffer
    pub offset: u32,
}

/// Describes how to interpret the data in a vertex buffer
///
/// Note that a pipeline can have multiple vertex buffers
/// and that their attributes will share vertex locations
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct VertexState<'a> {
    /// how far in bytes across all the attributes
    pub stride: u32,
    /// how the vertex shader consumes vertices
    pub input_rate: VertexInputRate,
    /// the attributes of the vertex
    pub attributes: &'a [VertexAttribute],
}

/// how to compare depth values
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum CompareOp {
    /// comparison never passes
    Never,
    /// comparison passes if new < old
    Less,
    /// comparison passes if new == old
    Equal,
    /// comparison passes if new <= old
    LessEqual,
    /// comparison passes if new > old
    Greater,
    /// comparison passes if new != old
    NotEqual,
    /// comparison passes if new >= old
    GreaterEqual,
    /// comparison always passes
    Always,
}

impl Into<vk::CompareOp> for CompareOp {
    fn into(self) -> vk::CompareOp {
        match self {
            Self::Never => vk::CompareOp::NEVER,
            Self::Less => vk::CompareOp::LESS,
            Self::Equal => vk::CompareOp::EQUAL,
            Self::LessEqual => vk::CompareOp::LESS_OR_EQUAL,
            Self::Greater => vk::CompareOp::GREATER,
            Self::NotEqual => vk::CompareOp::NOT_EQUAL,
            Self::GreaterEqual => vk::CompareOp::GREATER_OR_EQUAL,
            Self::Always => vk::CompareOp::ALWAYS,
        }
    }
}

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum StencilOp {
    Keep,
    Replace,
    Zero,
    Invert,
    DecrClamp,
    DecrWrap,
    IncrClamp,
    IncrWrap,
}

impl Into<vk::StencilOp> for StencilOp {
    fn into(self) -> vk::StencilOp {
        match self {
            Self::Keep => vk::StencilOp::KEEP,
            Self::Replace => vk::StencilOp::REPLACE,
            Self::Zero => vk::StencilOp::ZERO,
            Self::Invert => vk::StencilOp::INVERT,
            Self::DecrClamp => vk::StencilOp::DECREMENT_AND_CLAMP,
            Self::DecrWrap => vk::StencilOp::DECREMENT_AND_WRAP,
            Self::IncrClamp => vk::StencilOp::INCREMENT_AND_CLAMP,
            Self::IncrWrap => vk::StencilOp::INCREMENT_AND_WRAP,
        }
    }
}

/// Describes how a pipeline will do stencil testing
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct StencilState {
    /// What to do when the stencil test passes
    pub pass_op: StencilOp,
    /// What to do when the stencil test fails
    pub fail_op: StencilOp,
    /// What to do when the depth test fails
    pub depth_fail_op: StencilOp,
    /// How to compare the stencil state
    pub compare: CompareOp,
    /// The value to compare to
    pub compare_mask: u32,
    /// The value to write when replacing
    pub write_mask: u32,
    /// idk
    pub reference: u32,
}

/// Describes how a [`crate::GraphicsPipeline`] performs depth testing
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct DepthState {
    /// decides if depth testing is enabled
    pub test_enable: bool,
    /// decides if to output to the depth image
    pub write_enable: bool,
    /// how to compare depth values
    pub compare_op: CompareOp,
}

impl Default for DepthState {
    fn default() -> Self {
        Self {
            test_enable: true,
            write_enable: true,
            compare_op: CompareOp::LessEqual,
        }
    }
}

/// Describes how a GraphicsPipeline performs depth testing and stencil
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct DepthStencilState {
    /// The state for depth testing
    pub depth: Option<DepthState>,
    /// The state for stencil operations when drawing a front facing polygon
    pub stencil_front: Option<StencilState>,
    /// The state for stencil operations when drawing a back facing polygon
    pub stencil_back: Option<StencilState>,
}

impl DepthStencilState {
    /// Create a DepthStencilState with only depth state
    pub fn depth(test_enable: bool, write_enable: bool, compare_op: CompareOp) -> Self {
        Self {
            depth: Some(DepthState {
                test_enable,
                write_enable,
                compare_op,
            }),
            ..Default::default()
        }
    }

    /// Create a DepthStencilState with only depth that reads and writes to the buffer
    pub fn default_depth() -> Self {
        Self::depth(true, true, CompareOp::LessEqual)
    }
}

impl Default for DepthStencilState {
    fn default() -> Self {
        Self {
            depth: None,
            stencil_front: None,
            stencil_back: None,
        }
    }
}

impl Into<vk::PipelineDepthStencilStateCreateInfo> for DepthStencilState {
    fn into(self) -> vk::PipelineDepthStencilStateCreateInfo {
        let front = self
            .stencil_front
            .map(|s| vk::StencilOpState {
                fail_op: s.fail_op.into(),
                pass_op: s.pass_op.into(),
                depth_fail_op: s.depth_fail_op.into(),
                compare_op: s.compare.into(),
                compare_mask: s.compare_mask,
                write_mask: s.write_mask,
                reference: s.reference,
            })
            .unwrap_or(vk::StencilOpState::default());

        let back = self
            .stencil_back
            .map(|s| vk::StencilOpState {
                fail_op: s.fail_op.into(),
                pass_op: s.pass_op.into(),
                depth_fail_op: s.depth_fail_op.into(),
                compare_op: s.compare.into(),
                compare_mask: s.compare_mask,
                write_mask: s.write_mask,
                reference: s.reference,
            })
            .unwrap_or(vk::StencilOpState::default());

        vk::PipelineDepthStencilStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineDepthStencilStateCreateFlags::empty(),
            depth_test_enable: if let Some(d) = self.depth {
                if d.test_enable {
                    vk::TRUE
                } else {
                    vk::FALSE
                }
            } else {
                vk::FALSE
            },
            depth_write_enable: if let Some(d) = self.depth {
                if d.write_enable {
                    vk::TRUE
                } else {
                    vk::FALSE
                }
            } else {
                vk::FALSE
            },
            depth_compare_op: self
                .depth
                .map(|d| d.compare_op.into())
                .unwrap_or(vk::CompareOp::ALWAYS),
            depth_bounds_test_enable: vk::FALSE,
            front,
            back,
            stencil_test_enable: if self.stencil_front.is_some() || self.stencil_back.is_some() {
                vk::TRUE
            } else {
                vk::FALSE
            },
            ..Default::default()
        }
    }
}

/// how should attachments be loaded
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum LoadOp {
    /// Clear the attachment on load
    Clear,
    /// Leave the attachment on load
    Load,
    /// Don't care, the existing contents are undefined
    DontCare,
}

impl Into<vk::AttachmentLoadOp> for LoadOp {
    fn into(self) -> vk::AttachmentLoadOp {
        match self {
            Self::Clear => vk::AttachmentLoadOp::CLEAR,
            Self::Load => vk::AttachmentLoadOp::LOAD,
            Self::DontCare => vk::AttachmentLoadOp::DONT_CARE,
        }
    }
}

/// how should attachments be stored
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum StoreOp {
    /// store the attachments
    Store,
    /// the attachments contents are undefined after usage
    DontCare,
}

impl Into<vk::AttachmentStoreOp> for StoreOp {
    fn into(self) -> vk::AttachmentStoreOp {
        match self {
            Self::Store => vk::AttachmentStoreOp::STORE,
            Self::DontCare => vk::AttachmentStoreOp::DONT_CARE,
        }
    }
}

/// what value should the attachment be cleared to
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ClearValue {
    /// clear the color from floats
    ColorFloat([f32; 4]),
    /// clear the color from integers
    ColorInt([i32; 4]),
    /// clear the color from unsigned integers
    ColorUint([u32; 4]),
    /// clear depth
    Depth(f32),
    /// clear depth and stencil
    DepthStencil(f32, u32),
}

impl ClearValue {
    /// returns if the clear value is for color
    pub fn color(&self) -> bool {
        match self {
            Self::Depth(_) => false,
            Self::DepthStencil(_, _) => false,
            _ => true,
        }
    }
}

impl std::hash::Hash for ClearValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match *self {
            Self::ColorFloat(a) => unsafe { std::mem::transmute::<_, [u32; 4]>(a) }.hash(state),
            Self::ColorInt(a) => a.hash(state),
            Self::ColorUint(a) => a.hash(state),
            Self::Depth(d) => unsafe { std::mem::transmute::<_, u32>(d) }.hash(state),
            Self::DepthStencil(d, s) => {
                unsafe { std::mem::transmute::<_, u32>(d) }.hash(state);
                s.hash(state);
            }
        }
    }
}

impl Into<vk::ClearValue> for ClearValue {
    fn into(self) -> vk::ClearValue {
        match self {
            Self::ColorFloat(c) => vk::ClearValue {
                color: vk::ClearColorValue { float32: c },
            },
            Self::ColorInt(c) => vk::ClearValue {
                color: vk::ClearColorValue { int32: c },
            },
            Self::ColorUint(c) => vk::ClearValue {
                color: vk::ClearColorValue { uint32: c },
            },
            Self::Depth(d) => vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: d,
                    stencil: 0,
                },
            },
            Self::DepthStencil(d, s) => vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: d,
                    stencil: s,
                },
            },
        }
    }
}

impl Into<vk::ClearColorValue> for ClearValue {
    fn into(self) -> vk::ClearColorValue {
        match self {
            Self::ColorFloat(c) => vk::ClearColorValue { float32: c },
            Self::ColorInt(c) => vk::ClearColorValue { int32: c },
            Self::ColorUint(c) => vk::ClearColorValue { uint32: c },
            n => panic!(
                "ERROR: Attempt to clear color with a depth clear value {:?}",
                n
            ),
        }
    }
}

impl Into<vk::ClearDepthStencilValue> for ClearValue {
    fn into(self) -> vk::ClearDepthStencilValue {
        match self {
            Self::Depth(d) => vk::ClearDepthStencilValue {
                depth: d,
                stencil: 0,
            },
            Self::DepthStencil(d, s) => vk::ClearDepthStencilValue {
                depth: d,
                stencil: s,
            },
            n => panic!(
                "ERROR: Attempt to clear depth with a color clear value {:?}",
                n
            ),
        }
    }
}

/// number of samples of an image
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Samples {
    /// one sample
    S1 = 1,
    /// two samples
    S2 = 2,
    /// four samples
    S4 = 4,
    /// eight samples
    S8 = 8,
    /// sixteen samples
    S16 = 16,
    /// thirty two samples
    S32 = 32,
    /// sixty four samples
    S64 = 64,
}

impl Samples {
    /// Get the equivalent number of flags
    pub const fn flags(&self) -> SampleCountFlags {
        match self {
            Self::S1 => SampleCountFlags::TYPE_1,
            Self::S2 => SampleCountFlags::TYPE_2,
            Self::S4 => SampleCountFlags::TYPE_4,
            Self::S8 => SampleCountFlags::TYPE_8,
            Self::S16 => SampleCountFlags::TYPE_16,
            Self::S32 => SampleCountFlags::TYPE_32,
            Self::S64 => SampleCountFlags::TYPE_64,
        }
    }
}

impl Into<vk::SampleCountFlags> for Samples {
    fn into(self) -> vk::SampleCountFlags {
        match self {
            Self::S1 => vk::SampleCountFlags::TYPE_1,
            Self::S2 => vk::SampleCountFlags::TYPE_2,
            Self::S4 => vk::SampleCountFlags::TYPE_4,
            Self::S8 => vk::SampleCountFlags::TYPE_8,
            Self::S16 => vk::SampleCountFlags::TYPE_16,
            Self::S32 => vk::SampleCountFlags::TYPE_32,
            Self::S64 => vk::SampleCountFlags::TYPE_64,
        }
    }
}

/// A single entry to a DescriptorLayout
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum DescriptorLayoutEntry {
    /// At this location shaders should accept a uniform buffer
    ///
    /// In glsl looks like
    /// `layout(set = _, binding = _) uniform Struct { .. };`
    UniformBuffer {
        /// The shader stages that can access the buffer
        stage: crate::ShaderStages,
        /// The number of buffers to accept
        count: NonZeroU32,
    },
    /// At this location shaders should accept a storage buffer
    ///
    /// In glsl looks like
    /// `layout(set = _, binding = _) buffer Buffer { ..[] }'`
    StorageBuffer {
        /// The shader stages that can access the buffer
        stage: crate::ShaderStages,
        /// If the buffer is read only
        read_only: bool,
        /// The number of buffers to accept
        count: NonZeroU32,
    },
    /// At this location shaders should accept a sampled texture
    ///
    /// In glsl looks like
    /// `layout(set = _, binding = _) uniform texture2D u_texture;`
    SampledTexture {
        /// The shader stages that can access the texture
        stage: crate::ShaderStages,
        /// The number of textures
        count: NonZeroU32,
    },
    /// At this location shaders should accept a storage texture
    ///
    /// In glsl looks like
    /// TODO
    StorageTexture {
        /// The shader stages that can access the texture
        stage: crate::ShaderStages,
        /// If the texture is readonly or not
        read_only: bool,
        /// The number of textures
        count: NonZeroU32,
    },
    /// At this location shaders should accept a combined texture/sampler
    ///
    /// In glsl looks like
    /// `layout(set = _, binding = _) uniform sampler2D u_sampled;`
    CombinedTextureSampler {
        /// The shader stages that can access the combined texture/sampler
        stage: crate::ShaderStages,
        /// The number of combined texture/samplers
        count: NonZeroU32,
    },
    /// At this location shaders should accept a sampler
    ///
    /// In glsl looks like
    /// `layout(set = _, binding = _) uniform sampler u_samper`
    Sampler {
        /// The shader stages that can access the sampler
        stage: crate::ShaderStages,
        /// The number of samplers
        count: NonZeroU32,
    },
}

impl DescriptorLayoutEntry {
    /// Get the shader stage for this binding
    pub fn stage(&self) -> crate::ShaderStages {
        match self {
            Self::UniformBuffer { stage, .. } => *stage,
            Self::StorageBuffer { stage, .. } => *stage,
            Self::SampledTexture { stage, .. } => *stage,
            Self::StorageTexture { stage, .. } => *stage,
            Self::Sampler { stage, .. } => *stage,
            Self::CombinedTextureSampler { stage, .. } => *stage,
        }
    }

    /// The count of the binding
    pub fn count(&self) -> u32 {
        match self {
            Self::UniformBuffer { count, .. } => count.get(),
            Self::StorageBuffer { count, .. } => count.get(),
            Self::SampledTexture { count, .. } => count.get(),
            Self::StorageTexture { count, .. } => count.get(),
            Self::Sampler { count, .. } => count.get(),
            Self::CombinedTextureSampler { count, .. } => count.get(),
        }
    }
}

impl Into<vk::DescriptorType> for DescriptorLayoutEntry {
    fn into(self) -> vk::DescriptorType {
        match self {
            Self::UniformBuffer { .. } => vk::DescriptorType::UNIFORM_BUFFER,
            Self::StorageBuffer { .. } => vk::DescriptorType::STORAGE_BUFFER,
            Self::SampledTexture { .. } => vk::DescriptorType::SAMPLED_IMAGE,
            Self::StorageTexture { .. } => vk::DescriptorType::STORAGE_IMAGE,
            Self::CombinedTextureSampler { .. } => vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            Self::Sampler { .. } => vk::DescriptorType::SAMPLER,
        }
    }
}

impl Into<vk::DescriptorPoolSize> for DescriptorLayoutEntry {
    fn into(self) -> vk::DescriptorPoolSize {
        vk::DescriptorPoolSize {
            ty: self.into(),
            descriptor_count: self.count(),
        }
    }
}

/// An entry to a DescriptorLayout
#[derive(Debug, Clone)]
pub enum DescriptorSetEntry<'a> {
    /// Write a buffer to this binding
    Buffer(crate::BufferSlice<'a>),
    /// Write an array of buffers to this binding
    BufferArray(Cow<'a, [crate::BufferSlice<'a>]>),
    /// write a texture to this binding
    Texture(Cow<'a, crate::TextureView>, crate::TextureLayout),
    /// write an array of textures to this binding
    TextureArray(Cow<'a, [(Cow<'a, crate::TextureView>, crate::TextureLayout)]>),
    /// write a sampler to this binding
    Sampler(Cow<'a, crate::Sampler>),
    /// write a sampler array to this binding
    SamplerArray(Cow<'a, [Cow<'a, crate::Sampler>]>),
    /// write a combined texture/sampler to this binding
    CombinedTextureSampler(
        Cow<'a, crate::TextureView>,
        crate::TextureLayout,
        Cow<'a, crate::Sampler>,
    ),
    /// write an array of combined texture/samplers to this binding
    CombinedTextureSamplerArray(
        Cow<
            'a,
            [(
                Cow<'a, crate::TextureView>,
                crate::TextureLayout,
                Cow<'a, crate::Sampler>,
            )],
        >,
    ),
}

impl<'a> DescriptorSetEntry<'a> {
    pub fn into_owned(self) -> DescriptorSetEntry<'static> {
        match self {
            DescriptorSetEntry::Buffer(b) => DescriptorSetEntry::Buffer(b.as_owned()),
            DescriptorSetEntry::BufferArray(b) => {
                DescriptorSetEntry::BufferArray(b.into_iter().map(|a| a.as_owned()).collect())
            }
            DescriptorSetEntry::Texture(t, l) => {
                DescriptorSetEntry::Texture(Cow::Owned(t.clone().into_owned()), l)
            }
            DescriptorSetEntry::TextureArray(t) => DescriptorSetEntry::TextureArray(
                t.into_iter()
                    .map(|(t, l)| (Cow::Owned(t.clone().into_owned()), *l))
                    .collect(),
            ),
            DescriptorSetEntry::Sampler(s) => {
                DescriptorSetEntry::Sampler(Cow::Owned(s.clone().into_owned()))
            }
            DescriptorSetEntry::SamplerArray(s) => DescriptorSetEntry::SamplerArray(
                s.into_iter()
                    .map(|s| Cow::Owned(s.clone().into_owned()))
                    .collect(),
            ),
            DescriptorSetEntry::CombinedTextureSampler(t, l, s) => {
                DescriptorSetEntry::CombinedTextureSampler(
                    Cow::Owned(t.clone().into_owned()),
                    l,
                    Cow::Owned(s.clone().into_owned()),
                )
            }
            DescriptorSetEntry::CombinedTextureSamplerArray(a) => {
                DescriptorSetEntry::CombinedTextureSamplerArray(
                    a.into_iter()
                        .map(|(t, l, s)| {
                            (
                                Cow::Owned(t.clone().into_owned()),
                                *l,
                                Cow::Owned(s.clone().into_owned()),
                            )
                        })
                        .collect(),
                )
            }
        }
    }

    pub fn as_owned(&self) -> DescriptorSetEntry<'static> {
        match self {
            DescriptorSetEntry::Buffer(b) => DescriptorSetEntry::Buffer(b.as_owned()),
            DescriptorSetEntry::BufferArray(b) => {
                DescriptorSetEntry::BufferArray(b.into_iter().map(|a| a.as_owned()).collect())
            }
            DescriptorSetEntry::Texture(t, l) => {
                DescriptorSetEntry::Texture(Cow::Owned(t.clone().into_owned()), *l)
            }
            DescriptorSetEntry::TextureArray(t) => DescriptorSetEntry::TextureArray(
                t.into_iter()
                    .map(|(t, l)| (Cow::Owned(t.clone().into_owned()), *l))
                    .collect(),
            ),
            DescriptorSetEntry::Sampler(s) => {
                DescriptorSetEntry::Sampler(Cow::Owned(s.clone().into_owned()))
            }
            DescriptorSetEntry::SamplerArray(s) => DescriptorSetEntry::SamplerArray(
                s.into_iter()
                    .map(|s| Cow::Owned(s.clone().into_owned()))
                    .collect(),
            ),
            DescriptorSetEntry::CombinedTextureSampler(t, l, s) => {
                DescriptorSetEntry::CombinedTextureSampler(
                    Cow::Owned(t.clone().into_owned()),
                    *l,
                    Cow::Owned(s.clone().into_owned()),
                )
            }
            DescriptorSetEntry::CombinedTextureSamplerArray(a) => {
                DescriptorSetEntry::CombinedTextureSamplerArray(
                    a.into_iter()
                        .map(|(t, l, s)| {
                            (
                                Cow::Owned(t.clone().into_owned()),
                                *l,
                                Cow::Owned(s.clone().into_owned()),
                            )
                        })
                        .collect(),
                )
            }
        }
    }

    /// Create a buffer entry from a reference to a buffer
    #[inline]
    pub fn buffer(buffer: crate::BufferSlice<'a>) -> Self {
        Self::Buffer(buffer)
    }

    /// Create a buffer array entry from references to buffers
    #[inline]
    pub fn buffer_array_ref(buffers: &'a [crate::BufferSlice<'a>]) -> Self {
        Self::BufferArray(Cow::Borrowed(buffers))
    }

    /// Create a buffer array entry from buffers
    #[inline]
    pub fn buffer_array_owned(buffers: Vec<crate::BufferSlice<'a>>) -> Self {
        let buffers = buffers.into_iter().map(|b| b).collect::<Vec<_>>();
        Self::BufferArray(Cow::Owned(buffers))
    }

    /// Create a texture entry from a reference to a texture
    #[inline]
    pub fn texture_ref(texture: &'a crate::TextureView, layout: crate::TextureLayout) -> Self {
        Self::Texture(Cow::Borrowed(texture), layout)
    }

    /// Create a texture entry from a texture
    #[inline]
    pub fn texture_owned(texture: crate::TextureView, layout: crate::TextureLayout) -> Self {
        Self::Texture(Cow::Owned(texture), layout)
    }

    /// Create a texture array entry from references to textures
    #[inline]
    pub fn texture_array_ref(textures: &[(&'a crate::TextureView, crate::TextureLayout)]) -> Self {
        let textures = textures
            .iter()
            .map(|&(t, l)| (Cow::Borrowed(t), l))
            .collect::<Vec<_>>();
        Self::TextureArray(Cow::Owned(textures))
    }

    /// Create a texture array entry from textures
    #[inline]
    pub fn texture_array_owned(textures: Vec<(crate::TextureView, crate::TextureLayout)>) -> Self {
        let textures = textures
            .into_iter()
            .map(|(t, l)| (Cow::Owned(t), l))
            .collect::<Vec<_>>();
        Self::TextureArray(Cow::Owned(textures))
    }

    /// Create a sampler entry from a reference to a sampler
    #[inline]
    pub fn sampler_ref(sampler: &'a crate::Sampler) -> Self {
        Self::Sampler(Cow::Borrowed(sampler))
    }

    /// Create a sampler entry from a sampler
    #[inline]
    pub fn sampler_owned(sampler: crate::Sampler) -> Self {
        Self::Sampler(Cow::Owned(sampler))
    }

    /// Create a sampler array entry from references to samplers
    #[inline]
    pub fn sampler_array_ref(samplers: &[&'a crate::Sampler]) -> Self {
        let samplers = samplers
            .iter()
            .map(|&s| Cow::Borrowed(s))
            .collect::<Vec<_>>();
        Self::SamplerArray(Cow::Owned(samplers))
    }

    /// Create a sampler array entry from samplers
    #[inline]
    pub fn sampler_array_owned(samplers: Vec<crate::Sampler>) -> Self {
        let samplers = samplers
            .into_iter()
            .map(|s| Cow::Owned(s))
            .collect::<Vec<_>>();
        Self::SamplerArray(Cow::Owned(samplers))
    }

    /// Create a combined texture sampler entry from references
    #[inline]
    pub fn combined_texture_sampler_ref(
        texture: &'a crate::TextureView,
        layout: crate::TextureLayout,
        sampler: &'a crate::Sampler,
    ) -> Self {
        Self::CombinedTextureSampler(Cow::Borrowed(texture), layout, Cow::Borrowed(sampler))
    }

    /// Create a combined texture sampler entry from values
    #[inline]
    pub fn combined_texture_sampler_owned(
        texture: crate::TextureView,
        layout: crate::TextureLayout,
        sampler: crate::Sampler,
    ) -> Self {
        Self::CombinedTextureSampler(Cow::Owned(texture), layout, Cow::Owned(sampler))
    }

    /// Create a combined texture sampler entry from references
    #[inline]
    pub fn combined_texture_sampler_array_ref(
        refs: &[(
            &'a crate::TextureView,
            crate::TextureLayout,
            &'a crate::Sampler,
        )],
    ) -> Self {
        let result = refs
            .iter()
            .map(|&(t, l, s)| (Cow::Borrowed(t), l, Cow::Borrowed(s)))
            .collect::<Vec<_>>();
        Self::CombinedTextureSamplerArray(Cow::Owned(result))
    }

    /// Create a combined texture sampler entry from references
    #[inline]
    pub fn combined_texture_sampler_array_owned(
        refs: Vec<(crate::TextureView, crate::TextureLayout, crate::Sampler)>,
    ) -> Self {
        let result = refs
            .into_iter()
            .map(|(t, l, s)| (Cow::Owned(t), l, Cow::Owned(s)))
            .collect::<Vec<_>>();
        Self::CombinedTextureSamplerArray(Cow::Owned(result))
    }
}

bitflags::bitflags! {
    /// how a buffer can be used
    pub struct BufferUsage: u32 {
        /// Allows the buffer to be copied from
        const COPY_SRC    = 0b00000001;
        /// Allows the buffer to be copied to
        const COPY_DST    = 0b00000010;
        /// Allows the buffer to be used as a uniform variable
        const UNIFORM     = 0b00000100;
        /// Allows the buffer to be used as a storage variable
        const STORAGE     = 0b00001000;
        /// Allows the buffer to be used as a vertex buffer
        const VERTEX      = 0b00010000;
        /// Allows the buffer to be used as an index buffer
        const INDEX       = 0b00100000;
        #[cfg(feature = "ray")]
        const RAY_TRACING = 0b01000000;
    }
}

impl Into<vk::BufferUsageFlags> for BufferUsage {
    fn into(self) -> vk::BufferUsageFlags {
        let mut result = vk::BufferUsageFlags::empty();
        if self.contains(BufferUsage::COPY_SRC) {
            result |= vk::BufferUsageFlags::TRANSFER_SRC;
        }
        if self.contains(BufferUsage::COPY_DST) {
            result |= vk::BufferUsageFlags::TRANSFER_DST;
        }
        if self.contains(BufferUsage::UNIFORM) {
            result |= vk::BufferUsageFlags::UNIFORM_BUFFER;
        }
        if self.contains(BufferUsage::STORAGE) {
            result |= vk::BufferUsageFlags::STORAGE_BUFFER;
        }
        if self.contains(BufferUsage::VERTEX) {
            result |= vk::BufferUsageFlags::VERTEX_BUFFER;
        }
        if self.contains(BufferUsage::INDEX) {
            result |= vk::BufferUsageFlags::INDEX_BUFFER;
        }
        #[cfg(feature = "ray")]
        if self.contains(BufferUsage::RAY_TRACING) {
            result |= vk::BufferUsageFlags::RAY_TRACING_KHR;
        }
        result
    }
}

/// Describes the type of memory to use for an object
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum MemoryType {
    /// Faster to read and write from for the gpu but invisible to the cpu
    Device,
    /// Slower to read and write from for the gpu but visible to the cpu
    Host,
}

impl Into<vk::MemoryPropertyFlags> for MemoryType {
    fn into(self) -> vk::MemoryPropertyFlags {
        match self {
            Self::Device => vk::MemoryPropertyFlags::DEVICE_LOCAL,
            Self::Host => {
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT
            }
        }
    }
}

bitflags::bitflags! {
    /// Describes how a texture is allowed to be used
    pub struct TextureUsage: u32 {
        /// allows images to be copied from
        const COPY_SRC       = 0b00000000001;
        /// allows images to be copied to
        const COPY_DST       = 0b00000000010;
        /// allows images to be sampled from in shaders
        const SAMPLED        = 0b00000000100;
        /// allows images to be used as storage images
        const STORAGE        = 0b00000001000;
        /// allows render passes to output color to images
        const COLOR_OUTPUT   = 0b00000010000;
        /// allows render passes to output depth to images
        const DEPTH_OUTPUT   = 0b00000100000;
        /// allows creating views of different formats that base
        const MUTABLE_FORMAT = 0b00001000000;
        /// indicates that image data is not needed outside of rendering
        const TRANSIENT      = 0b00010000000;
    }
}

impl TextureUsage {
    pub(crate) fn flags(&self) -> vk::ImageCreateFlags {
        (*self).into()
    }
}

impl Into<vk::ImageUsageFlags> for TextureUsage {
    fn into(self) -> vk::ImageUsageFlags {
        let mut result = vk::ImageUsageFlags::empty();
        if self.contains(Self::COPY_SRC) {
            result |= vk::ImageUsageFlags::TRANSFER_SRC;
        }
        if self.contains(Self::COPY_DST) {
            result |= vk::ImageUsageFlags::TRANSFER_DST;
        }
        if self.contains(Self::SAMPLED) {
            result |= vk::ImageUsageFlags::SAMPLED;
        }
        if self.contains(Self::STORAGE) {
            result |= vk::ImageUsageFlags::STORAGE;
        }
        if self.contains(Self::COLOR_OUTPUT) {
            result |= vk::ImageUsageFlags::COLOR_ATTACHMENT;
        }
        if self.contains(Self::DEPTH_OUTPUT) {
            result |= vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT;
        }
        if self.contains(Self::TRANSIENT) {
            result |= vk::ImageUsageFlags::TRANSIENT_ATTACHMENT;
        }
        result
    }
}

impl Into<vk::ImageCreateFlags> for TextureUsage {
    fn into(self) -> vk::ImageCreateFlags {
        if self.contains(TextureUsage::MUTABLE_FORMAT) {
            vk::ImageCreateFlags::MUTABLE_FORMAT
        } else {
            vk::ImageCreateFlags::empty()
        }
    }
}

/// Describes the type of indices in an index buffer
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum IndexType {
    /// indices are interpreted as u16
    U16,
    /// indices are interpreted as u32
    U32,
}

impl Into<vk::IndexType> for IndexType {
    fn into(self) -> vk::IndexType {
        match self {
            Self::U16 => vk::IndexType::UINT16,
            Self::U32 => vk::IndexType::UINT32,
        }
    }
}

/// Represents a dimension like width/height
pub type Size = u32;
/// Represents the number of array layers
pub type Layer = u32;

/// More representitive of how texture dimensions are represented in vulkan
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
#[allow(missing_docs)]
pub enum TextureKind {
    D1,
    D2,
    D3,
}

impl Into<vk::ImageType> for TextureKind {
    fn into(self) -> vk::ImageType {
        match self {
            Self::D1 => vk::ImageType::TYPE_1D,
            Self::D2 => vk::ImageType::TYPE_2D,
            Self::D3 => vk::ImageType::TYPE_3D,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub struct TextureFormatProperties {
    pub max_extent: Extent3D,
    pub max_mip_levels: u32,
    pub sample_counts: SampleCountFlags,
    pub max_array_layers: u32,
}

impl From<vk::ImageFormatProperties> for TextureFormatProperties {
    fn from(p: vk::ImageFormatProperties) -> Self {
        Self {
            max_extent: p.max_extent.into(),
            max_mip_levels: p.max_mip_levels,
            max_array_layers: p.max_array_layers,
            sample_counts: p.sample_counts,
        }
    }
}

/// Describes the dimension of a texture
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TextureDimension {
    /// 1 dimensional image with Size number of pixels
    D1(Size),
    /// 1 dimensional image with Size number of pixels and Layer number of layers
    D1Array(Size, Layer),
    /// 2 dimensional image of Size x Size pixels
    D2(Size, Size, Samples),
    /// 2 dimensional image of Size x Size pixels and Layer number of layers
    D2Array(Size, Size, Samples, Layer),
    /// Cube image with each face of Size x Size pixels
    Cube(Size, Size),
    /// Cube image with each face of Size x Size pixels and Layer number of layers
    CubeArray(Size, Size, Layer),
    // Cube image with each face of Size x Size and multisampling support
    //CubeMs(Size, Size, Samples),
    // Cube image with each face of Size x Size and Layer number of layers and multisampling support
    //CubeArrayMs(Size, Size, Layer, Samples),
    /// 3 dimensions image with Size x Size x Size dimensions
    D3(Size, Size, Size),
}

impl TextureDimension {
    /// Get the number of array layers in the dimension
    pub fn layers(&self) -> Layer {
        match self {
            TextureDimension::D1Array(_, l) => *l,
            TextureDimension::D2Array(_, _, _, l) => *l,
            TextureDimension::Cube(_, _) => 6,
            TextureDimension::CubeArray(_, _, l) => 6 * *l,
            //TextureDimension::CubeMs(_, _, _) => 6,
            //TextureDimension::CubeArrayMs(_, _, l, _) => 6 * *l,
            _ => 1,
        }
    }

    /// Get the number of samples in the dimension
    pub fn samples(&self) -> Samples {
        match self {
            TextureDimension::D2(_, _, s) => *s,
            TextureDimension::D2Array(_, _, s, _) => *s,
            //TextureDimension::CubeMs(_, _, s) => *s,
            //TextureDimension::CubeArrayMs(_, _, _, s) => *s,
            _ => Samples::S1,
        }
    }

    /// Get the kind of the dimension
    pub fn kind(&self) -> TextureKind {
        match self {
            TextureDimension::D1(_) => TextureKind::D1,
            TextureDimension::D1Array(_, _) => TextureKind::D1,
            TextureDimension::D2(_, _, _) => TextureKind::D2,
            TextureDimension::D2Array(_, _, _, _) => TextureKind::D2,
            TextureDimension::Cube(_, _) => TextureKind::D2,
            TextureDimension::CubeArray(_, _, _) => TextureKind::D2,
            //TextureDimension::CubeMs(_, _, _) => TextureKind::D2,
            //TextureDimension::CubeArrayMs(_, _, _, _) => TextureKind::D2,
            TextureDimension::D3(_, _, _) => TextureKind::D3,
        }
    }

    pub(crate) fn flags(&self) -> vk::ImageCreateFlags {
        match self {
            TextureDimension::D2Array(_, _, _, _) => vk::ImageCreateFlags::TYPE_2D_ARRAY_COMPATIBLE,
            TextureDimension::Cube(_, _) => vk::ImageCreateFlags::CUBE_COMPATIBLE,
            TextureDimension::CubeArray(_, _, _) => vk::ImageCreateFlags::CUBE_COMPATIBLE,
            //TextureDimension::CubeMs(_, _, _) => vk::ImageCreateFlags::CUBE_COMPATIBLE,
            //TextureDimension::CubeArrayMs(_, _, _, _) => vk::ImageCreateFlags::CUBE_COMPATIBLE,
            TextureDimension::D3(_, _, _) => vk::ImageCreateFlags::TYPE_2D_ARRAY_COMPATIBLE,
            _ => vk::ImageCreateFlags::empty(),
        }
    }
}

impl Into<vk::ImageType> for TextureDimension {
    fn into(self) -> vk::ImageType {
        match self {
            TextureDimension::D1(_) => vk::ImageType::TYPE_1D,
            TextureDimension::D1Array(_, _) => vk::ImageType::TYPE_1D,
            TextureDimension::D2(_, _, _) => vk::ImageType::TYPE_2D,
            TextureDimension::D2Array(_, _, _, _) => vk::ImageType::TYPE_2D,
            TextureDimension::Cube(_, _) => vk::ImageType::TYPE_2D,
            TextureDimension::CubeArray(_, _, _) => vk::ImageType::TYPE_2D,
            //TextureDimension::CubeMs(_, _, _) => vk::ImageType::TYPE_2D,
            //TextureDimension::CubeArrayMs(_, _, _, _) => vk::ImageType::TYPE_2D,
            TextureDimension::D3(_, _, _) => vk::ImageType::TYPE_3D,
        }
    }
}

impl Into<vk::ImageViewType> for TextureDimension {
    fn into(self) -> vk::ImageViewType {
        match self {
            TextureDimension::D1(_) => vk::ImageViewType::TYPE_1D,
            TextureDimension::D1Array(_, _) => vk::ImageViewType::TYPE_1D,
            TextureDimension::D2(_, _, _) => vk::ImageViewType::TYPE_2D,
            TextureDimension::D2Array(_, _, _, _) => vk::ImageViewType::TYPE_2D_ARRAY,
            TextureDimension::Cube(_, _) => vk::ImageViewType::CUBE,
            TextureDimension::CubeArray(_, _, _) => vk::ImageViewType::CUBE_ARRAY,
            //TextureDimension::CubeMs(_, _, _) => vk::ImageViewType::TYPE_2D,
            //TextureDimension::CubeArrayMs(_, _, _, _) => vk::ImageViewType::TYPE_2D,
            TextureDimension::D3(_, _, _) => vk::ImageViewType::TYPE_3D,
        }
    }
}

impl Into<vk::Extent3D> for TextureDimension {
    fn into(self) -> vk::Extent3D {
        let tmp: crate::Extent3D = self.into();
        tmp.into()
    }
}

impl Into<crate::Extent3D> for TextureDimension {
    fn into(self) -> crate::Extent3D {
        match self {
            TextureDimension::D1(w) => crate::Extent3D {
                width: w,
                height: 1,
                depth: 1,
            },
            TextureDimension::D1Array(w, _) => crate::Extent3D {
                width: w,
                height: 1,
                depth: 1,
            },
            TextureDimension::D2(w, h, _) => crate::Extent3D {
                width: w,
                height: h,
                depth: 1,
            },
            TextureDimension::D2Array(w, h, _, _) => crate::Extent3D {
                width: w,
                height: h,
                depth: 1,
            },
            TextureDimension::Cube(w, h) => crate::Extent3D {
                width: w,
                height: h,
                depth: 1,
            },
            TextureDimension::CubeArray(w, h, _) => crate::Extent3D {
                width: w,
                height: h,
                depth: 1,
            },
            /*
            TextureDimension::CubeMs(w, h, _) => crate::Extent3D {
                width: w,
                height: h,
                depth: 1,
            },
            TextureDimension::CubeArrayMs(w, h, _, _) => crate::Extent3D {
                width: w,
                height: h,
                depth: 1,
            },*/
            TextureDimension::D3(w, h, d) => crate::Extent3D {
                width: w,
                height: h,
                depth: d,
            },
        }
    }
}

/// Describes how sampling outside image dimensions should be performed
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum WrapMode {
    /// repeat with the image if sampling outside the image dimensions
    Repeat,
    /// reflect and repeat the image if sampling outside the image dimensions
    MirroredRepeat,
    /// clamp the color to the edge of the image
    ClampToEdge,
    /// clamp the color to the border of the image
    ClampToBorder,
}

impl Into<vk::SamplerAddressMode> for WrapMode {
    fn into(self) -> vk::SamplerAddressMode {
        match self {
            Self::Repeat => vk::SamplerAddressMode::REPEAT,
            Self::MirroredRepeat => vk::SamplerAddressMode::MIRRORED_REPEAT,
            Self::ClampToEdge => vk::SamplerAddressMode::CLAMP_TO_EDGE,
            Self::ClampToBorder => vk::SamplerAddressMode::CLAMP_TO_BORDER,
        }
    }
}

/// Descibes how sampling between pixels is performed
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum FilterMode {
    /// take the nearest pixel to the coordinate
    Nearest,
    /// linearly interpolate between pixels
    Linear,
}

impl Into<vk::SamplerMipmapMode> for FilterMode {
    fn into(self) -> vk::SamplerMipmapMode {
        match self {
            Self::Nearest => vk::SamplerMipmapMode::NEAREST,
            Self::Linear => vk::SamplerMipmapMode::LINEAR,
        }
    }
}

impl Into<vk::Filter> for FilterMode {
    fn into(self) -> vk::Filter {
        match self {
            Self::Nearest => vk::Filter::NEAREST,
            Self::Linear => vk::Filter::LINEAR,
        }
    }
}

/// Describes the color to be used when WrapMode::ClampToBorder is used
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum BorderColor {
    /// Border opaque black
    OpaqueBlack,
    /// Border transparent black
    TransparentBlack,
    /// Border opaque white
    OpaqueWhite,
}

impl Into<vk::BorderColor> for BorderColor {
    fn into(self) -> vk::BorderColor {
        match self {
            Self::OpaqueBlack => vk::BorderColor::FLOAT_OPAQUE_BLACK,
            Self::TransparentBlack => vk::BorderColor::FLOAT_TRANSPARENT_BLACK,
            Self::OpaqueWhite => vk::BorderColor::FLOAT_OPAQUE_WHITE,
        }
    }
}

/// A Layout of a texture in memory
///
/// will be different for different implementations
/// or some may be the same in the underlying implementation of vulkan
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[allow(missing_docs)]
pub enum TextureLayout {
    Undefined,
    General,
    ColorAttachmentOptimal,
    DepthStencilAttachmentOptimal,
    DepthStencilReadOnlyOptimal,
    ShaderReadOnlyOptimal,
    CopySrcOptimal,
    CopyDstOptimal,
    DepthAttachmentOptimal,
    DepthReadOnlyOptimal,
    StencilReadOnlyOptimal,
    SwapchainPresent,
}

impl Into<vk::ImageLayout> for TextureLayout {
    fn into(self) -> vk::ImageLayout {
        match self {
            Self::Undefined => vk::ImageLayout::UNDEFINED,
            Self::General => vk::ImageLayout::GENERAL,
            Self::ColorAttachmentOptimal => vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            Self::DepthStencilAttachmentOptimal => {
                vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL
            }
            Self::DepthStencilReadOnlyOptimal => vk::ImageLayout::DEPTH_STENCIL_READ_ONLY_OPTIMAL,
            Self::ShaderReadOnlyOptimal => vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            Self::CopySrcOptimal => vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            Self::CopyDstOptimal => vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            Self::DepthAttachmentOptimal => vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL,
            Self::DepthReadOnlyOptimal => vk::ImageLayout::DEPTH_READ_ONLY_OPTIMAL,
            Self::StencilReadOnlyOptimal => vk::ImageLayout::STENCIL_READ_ONLY_OPTIMAL,
            Self::SwapchainPresent => vk::ImageLayout::PRESENT_SRC_KHR,
        }
    }
}

/// Tells a CommandRecoder where to set a DescriptorSet
#[derive(Copy, Clone, Debug)]
pub enum PipelineBindPoint {
    /// set the DescriptorSet in the graphics pipeline bound
    Graphics,
    /// set the DescriptorSet in the compute pipeline bound
    Compute,
    /// set the DescriptorSet in the ray pipeline bound
    #[cfg(feature = "ray")]
    Ray,
    /// set the DescriptorSet in the mesh pipeline bound
    #[cfg(feature = "mesh")]
    Mesh,
}

impl Into<vk::PipelineBindPoint> for PipelineBindPoint {
    fn into(self) -> vk::PipelineBindPoint {
        match self {
            Self::Graphics => vk::PipelineBindPoint::GRAPHICS,
            Self::Compute => vk::PipelineBindPoint::COMPUTE,
            #[cfg(feature = "ray")]
            Self::Ray => vk::PipelineBindPoint::RAY_TRACING_KHR,
        }
    }
}

bitflags::bitflags! {
    /// Describes how a resource (Buffer/Texture) is accessed in a command buffer
    pub struct AccessFlags: u32 {
        #[allow(missing_docs)]
        const INDEX_READ                     = 0b000000000000000001;
        #[allow(missing_docs)]
        const VERTEX_ATTRIBUTE_READ          = 0b000000000000000010;
        #[allow(missing_docs)]
        const UNIFORM_READ                   = 0b000000000000000100;
        #[allow(missing_docs)]
        #[allow(missing_docs)]
        const INPUT_ATTACHMENT_READ          = 0b000000000000001000;
        #[allow(missing_docs)]
        const SHADER_READ                    = 0b000000000000010000;
        #[allow(missing_docs)]
        const SHADER_WRITE                   = 0b000000000000100000;
        #[allow(missing_docs)]
        const COLOR_ATTACHMENT_READ          = 0b000000000001000000;
        #[allow(missing_docs)]
        const COLOR_ATTACHMENT_WRITE         = 0b000000000010000000;
        #[allow(missing_docs)]
        const DEPTH_STENCIL_ATTACHMENT_READ  = 0b000000000100000000;
        #[allow(missing_docs)]
        const DEPTH_STENCIL_ATTACHMENT_WRITE = 0b000000001000000000;
        #[allow(missing_docs)]
        const COPY_READ                      = 0b000000010000000000;
        #[allow(missing_docs)]
        const COPY_WRITE                     = 0b000000100000000000;
        #[allow(missing_docs)]
        const HOST_READ                      = 0b000001000000000000;
        #[allow(missing_docs)]
        const MEMORY_READ                    = 0b000010000000000000;
        #[allow(missing_docs)]
        const MEMORY_WRITE                   = 0b000100000000000000;
        #[cfg(feature = "ray")]
        #[allow(missing_docs)]
        const ACCELERATION_STRUCTURE_READ    = 0b001000000000000000;
        #[cfg(feature = "ray")]
        #[allow(missing_docs)]
        const ACCELERATION_STRUCTURE_WRITE   = 0b010000000000000000;
    }
}

impl Into<vk::AccessFlags> for AccessFlags {
    fn into(self) -> vk::AccessFlags {
        let mut result = vk::AccessFlags::empty();
        if self.contains(Self::INDEX_READ) {
            result |= vk::AccessFlags::INDEX_READ;
        }
        if self.contains(Self::VERTEX_ATTRIBUTE_READ) {
            result |= vk::AccessFlags::VERTEX_ATTRIBUTE_READ;
        }
        if self.contains(Self::UNIFORM_READ) {
            result |= vk::AccessFlags::UNIFORM_READ;
        }
        if self.contains(Self::INPUT_ATTACHMENT_READ) {
            result |= vk::AccessFlags::INPUT_ATTACHMENT_READ;
        }
        if self.contains(Self::SHADER_READ) {
            result |= vk::AccessFlags::SHADER_READ;
        }
        if self.contains(Self::SHADER_WRITE) {
            result |= vk::AccessFlags::SHADER_WRITE;
        }
        if self.contains(Self::COLOR_ATTACHMENT_READ) {
            result |= vk::AccessFlags::COLOR_ATTACHMENT_READ;
        }
        if self.contains(Self::COLOR_ATTACHMENT_WRITE) {
            result |= vk::AccessFlags::COLOR_ATTACHMENT_WRITE;
        }
        if self.contains(Self::DEPTH_STENCIL_ATTACHMENT_READ) {
            result |= vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ;
        }
        if self.contains(Self::DEPTH_STENCIL_ATTACHMENT_WRITE) {
            result |= vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE;
        }
        if self.contains(Self::COPY_READ) {
            result |= vk::AccessFlags::TRANSFER_READ;
        }
        if self.contains(Self::COPY_WRITE) {
            result |= vk::AccessFlags::TRANSFER_WRITE;
        }
        if self.contains(Self::HOST_READ) {
            result |= vk::AccessFlags::HOST_READ;
        }
        if self.contains(Self::MEMORY_READ) {
            result |= vk::AccessFlags::MEMORY_READ;
        }
        if self.contains(Self::MEMORY_WRITE) {
            result |= vk::AccessFlags::MEMORY_WRITE;
        }
        #[cfg(feature = "ray")]
        if self.contains(Self::ACCELERATION_STRUCTURE_READ) {
            result |= vk::AccessFlags::ACCELERATION_STRUCTURE_READ_KHR;
        }
        #[cfg(feature = "ray")]
        if self.contains(Self::ACCELERATION_STRUCTURE_WRITE) {
            result |= vk::AccessFlags::ACCELERATION_STRUCTURE_WRITE_KHR;
        }
        return result;
    }
}

/// Tessellation options for graphics shaders
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Tesselation<'a> {
    /// the tessellation evaulation shader
    pub eval: &'a crate::ShaderModule,
    /// the tessellation control shader, not required
    pub control: Option<&'a crate::ShaderModule>,
    /// the number of control points per patch, not required
    pub patch_points: Option<u32>,
}

/// Describes what a color attachment will look like in a RenderPass
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ColorAttachmentDesc {
    pub format: crate::Format,
    pub load: crate::LoadOp,
    pub store: crate::StoreOp,
    pub initial_layout: crate::TextureLayout,
    pub final_layout: crate::TextureLayout,
}

/// Describes what a resolve attachment will look like in a RenderPass
///
/// Note that there is no format field as the resolve attachment must have the same format as the
/// corresponding color attachment
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResolveAttachmentDesc {
    pub load: crate::LoadOp,
    pub store: crate::StoreOp,
    pub initial_layout: crate::TextureLayout,
    pub final_layout: crate::TextureLayout,
}

/// Describes what a depth attachment will look like in a RenderPass
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DepthAttachmentDesc {
    pub format: crate::Format,
    pub load: crate::LoadOp,
    pub store: crate::StoreOp,
    pub initial_layout: crate::TextureLayout,
    pub final_layout: crate::TextureLayout,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Attachment<'a> {
    View(&'a crate::TextureView, crate::ClearValue),
    Swapchain(&'a crate::SwapchainView<'a>, crate::ClearValue),
}

impl<'a> Attachment<'a> {
    pub fn clear_value(&self) -> crate::ClearValue {
        match self {
            Self::View(_, c) => *c,
            Self::Swapchain(_, c) => *c,
        }
    }

    pub fn view(&self) -> &'a crate::TextureView {
        match self {
            Self::View(v, _) => v,
            Self::Swapchain(s, _) => s.view,
        }
    }
}
