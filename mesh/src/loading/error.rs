
#[derive(Debug)]
pub enum LoadError {
    Gpu(String, gpu::Error),
    Tobj(tobj::LoadError),
    MissingNormals(String),
    MissingUvs(String),
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::Gpu(n, e) => writeln!(f, "Error loading file: {}, {}", n, e),
            LoadError::MissingNormals(n) => writeln!(f ,"Error loading {}, missing normals", n),
            LoadError::MissingUvs(n) => writeln!(f, "Error loading {}, missing uv coordinates", n),
            LoadError::Tobj(e) => writeln!(f, "{}", e),
        }
    }
}

impl std::error::Error for LoadError { }
