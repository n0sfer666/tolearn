use std::path::{Path, PathBuf};

use tauri::Manager;

use super::error::IpcError;

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

pub fn of(app: &tauri::AppHandle) -> Result<Context, IpcError> {
    let data = app
        .path()
        .app_data_dir()
        .map_err(|error| IpcError::unreadable("app-data-dir", &error.to_string()))?;
    std::fs::create_dir_all(&data)
        .map_err(|error| IpcError::unwritable(&data, &error.to_string()))?;
    Ok(Context::new(&data))
}
