#[derive(Debug)]
pub enum ReflectedError {
    /// An error from spirv-reflect
    #[cfg(feature = "reflect")]
    Parse(ParseSpirvError),
    /// An error from invalid builder
    #[cfg(feature = "spirv")]
    Builder(BuilderConfigError),
    /// An error from the gpu
    Gpu(gpu::Error),
}

impl std::fmt::Display for ReflectedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse(e) => writeln!(f, "{}", e),
            Self::Builder(e) => writeln!(f, "{}", e),
            Self::Gpu(e) => writeln!(f, "{}", e),
        }
    }
}

impl std::error::Error for ReflectedError {}

impl From<ParseSpirvError> for ReflectedError {
    fn from(e: ParseSpirvError) -> Self {
        Self::Parse(e)
    }
}

impl From<gpu::Error> for ReflectedError {
    fn from(e: gpu::Error) -> Self {
        Self::Gpu(e)
    }
}

#[derive(Debug)]
pub enum BuilderConfigError {
    StageIncompatibility {
        location: u32,
        src_stage_name: Option<&'static str>,
        dst_stage_name: Option<&'static str>,
        src_type: spv::PrimitiveType,
        dst_type: spv::PrimitiveType,
    }
}

impl std::error::Error for BuilderConfigError {}

impl std::fmt::Display for BuilderConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuilderConfigError::StageIncompatibility { 
                location, 
                src_stage_name, 
                dst_stage_name, 
                src_type, 
                dst_type 
            } => writeln!(f, "ERROR: Stage Incompatibility, location: {}, src: ({:?}, {:?}), dst: ({:?}, {:?})", location, src_stage_name, src_type, dst_stage_name, dst_type),
        }
    }
}

#[derive(Debug)]
pub enum ParseSpirvError {
    /// See message from reflect
    ReflectError(ReflectError),
    /// Missing set self.0
    MissingSet(u32),
    /// Missing Binding set self.0, binding self.1
    MissingBinding(u32, u32),
    /// Set name confilct set self.0, binding self.1
    SetConflict(u32, u32, String, String),
    /// Multiple bindings have the same name: self.0
    SetNameConfilct(String),
    /// Entry point not found
    EntryPointNotFound,
    /// Vertex format self.0 doesn't map embers_gpu formats yet
    VertexFormatInvalid(spirv_reflect::types::image::ReflectFormat),
    /// Shader stages {src_stage_name} and {dst_stage_name} input and output at location {location} have different types {src_type} {dst_type}
    StageIncompatibility {
        /// the location of the conflict
        location: u32,
        /// the name in the src stage
        src_stage_name: String,
        /// the type that the src emmits
        src_type: spirv_reflect::types::image::ReflectFormat,
        /// the name in the dst stage
        dst_stage_name: String,
        /// the type that the dst accepts
        dst_type: spirv_reflect::types::image::ReflectFormat,
    },
}

impl std::fmt::Display for ParseSpirvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReflectError(e) => writeln!(f, "{}", e),
            Self::MissingSet(e) => writeln!(f, "ERROR: Missing set {}", e),
            Self::MissingBinding(set, binding) => writeln!(f, "ERROR: Missing Binding set {}, binding {}", set, binding),
            Self::SetConflict(set, binding, n1, n2) => writeln!(f, "ERROR: Both {} and {} point to the same location set {}, binding {}", n1, n2, set, binding),
            Self::SetNameConfilct(name) => writeln!(f, "ERROR: Multiple bindings have the same name: {}\nThis is probably caused by different shader stages using the same name for variables however due to current limitations in embers_gfx this isn't allowed at the moment", name),
            Self::EntryPointNotFound => writeln!(f, "ERROR: Entry point not found"),
            Self::VertexFormatInvalid(fmt) => writeln!(f, "ERROR: Vertex format {:?} doesn't map the gpu formats yet", fmt),
            Self::StageIncompatibility {
                location,
                src_stage_name,
                src_type,
                dst_stage_name,
                dst_type,
            } => writeln!(f, "ERROR: Shader stages {} and {} input and output at location {} have different types {:?} and {:?}", src_stage_name, dst_stage_name, location, src_type, dst_type),
        }
    }
}

impl std::error::Error for ParseSpirvError {}

impl From<ReflectError> for ParseSpirvError {
    fn from(e: ReflectError) -> Self {
        Self::ReflectError(e)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReflectError(pub &'static str);

impl std::fmt::Display for ReflectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ERROR: {}", self.0)
    }
}

impl std::error::Error for ReflectError {}

impl From<&'static str> for ReflectError {
    fn from(s: &'static str) -> Self {
        Self(s)
    }
}

#[derive(Debug)]
pub enum SetResourceError {
    /// Expected resource type self.0 found self.1
    WrongType(
        gpu::DescriptorLayoutEntryType,
        gpu::DescriptorLayoutEntryType,
    ),
    /// Attempt to set resource at id self.0 not found
    IdNotFound(String),
    /// expected an array
    ArrayExpected,
    /// expected a single object
    SingleExpected,
}

impl std::fmt::Display for SetResourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WrongType(a, b) => writeln!(
                f,
                "ERROR: Attempt to set resource on bundle, expected resource type {:?} found {:?}",
                a, b
            ),
            Self::IdNotFound(s) => writeln!(
                f,
                "ERROR: Attempt to set resource on bundle at id {}, not found",
                s
            ),
            Self::ArrayExpected => writeln!(
                f,
                "ERROR: Attempt to set resource on bundle of unit type expected array"
            ),
            Self::SingleExpected => writeln!(
                f,
                "ERROR: Attempt to set resource on bundle of array type expected unit"
            ),
        }
    }
}

impl std::error::Error for SetResourceError {}
