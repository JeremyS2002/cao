
use ash::vk;

/// An all encompassing error type
#[derive(Debug)]
pub enum Error {
    /// An explicit error returned from the vulkan api
    /// Some variants such as SURFACE_OUT_OF_DATA can be
    /// recovered from
    Explicit(ExplicitError),
    /// An error from a validation layer
    /// Cannot be recovered from safely
    Validation(ValidationError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Explicit(t) => writeln!(f, "{}", t),
            Self::Validation(t) => write!(f, "{}", t),
        }
    }
}

impl std::error::Error for Error {}

impl From<ExplicitError> for Error {
    fn from(e: ExplicitError) -> Self {
        Self::Explicit(e)
    }
}

impl From<ValidationError> for Error {
    fn from(e: ValidationError) -> Self {
        Self::Validation(e)
    }
}

/// An error from a validation layer
pub struct ValidationError(pub Vec<String>);

impl std::fmt::Debug for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for message in &self.0 {
            writeln!(f, "{}", message)?;
            writeln!(f)?;
        }
        Ok(())
    }
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for message in &self.0 {
            writeln!(f, "{}", message)?;
            writeln!(f)?;
        }
        Ok(())
    }
}

/// An error from the vulkan api
pub struct ExplicitError(pub vk::Result);

impl std::fmt::Debug for ExplicitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:?}", self.0)
    }
}

impl std::fmt::Display for ExplicitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:?}", self.0)
    }
}

impl std::error::Error for ExplicitError {}

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
