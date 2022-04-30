use ash::vk;

/// An all encompassing error type
#[derive(Debug)]
pub enum Error {
    /// An explicit error returned from the vulkan api
    /// Some variants such as SURFACE_OUT_OF_DATA can be
    /// recovered from
    Explicit(vk::Result),
    /// An error from a validation layer
    /// Cannot be recovered from safely
    Validation(Vec<String>),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Explicit(t) => {
                writeln!(f, "{}", t)
            }
            Self::Validation(t) => {
                for message in t {
                    writeln!(f, "{}", message)?;
                    writeln!(f)?;
                }
                Ok(())
            },
        }
    }
}

impl std::error::Error for Error {}

impl From<vk::Result> for Error {
    fn from(e: vk::Result) -> Self {
        Self::Explicit(e)
    }
}

/// An error when converting data to a spirv module
#[derive(Debug)]
pub enum MakeSpirvError {
    /// Missing the spirv magic number
    MissingMagicNumber,
    /// Spirv must be 4 byte alligned
    NotMultipleOfFour,
}

impl std::fmt::Display for MakeSpirvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingMagicNumber => write!(f, "Missing magic number"),
            Self::NotMultipleOfFour => write!(f, "Spirv bytes len must be multiple of 4"),
        }
    }
}

impl std::error::Error for MakeSpirvError {}
