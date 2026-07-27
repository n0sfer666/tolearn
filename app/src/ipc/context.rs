use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Context {
    data: PathBuf,
}

impl Context {
    pub fn new(data: &Path) -> Self {
        Self {
            data: data.to_path_buf(),
        }
    }

    pub fn registry(&self) -> PathBuf {
        self.data.join("registry.yaml")
    }
}
