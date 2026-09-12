use std::path::{Path, PathBuf};
use std::sync::Arc;

use tauri::Manager;
use tolearn_core::library::Library;
use tolearn_provider::{Keychain, Vault};

use super::error::IpcError;
use super::layout;

const SERVICE: &str = "tolearn";
const ACCOUNT: &str = "provider";

#[derive(Debug, Clone)]
pub struct Context {
    config: PathBuf,
    data: PathBuf,
    resources: PathBuf,
    vault: Arc<dyn Vault>,
}

impl Context {
    pub fn new(data: &Path) -> Self {
        Self::split(data, data)
    }

    pub fn split(config: &Path, data: &Path) -> Self {
        Self {
            config: config.to_path_buf(),
            data: data.to_path_buf(),
            resources: data.to_path_buf(),
            vault: Arc::new(Keychain::new(SERVICE, ACCOUNT)),
        }
    }

    pub fn shipped(mut self, resources: &Path) -> Self {
        self.resources = resources.to_path_buf();
        self
    }

    pub fn with_vault(data: &Path, vault: Arc<dyn Vault>) -> Self {
        Self {
            config: data.to_path_buf(),
            data: data.to_path_buf(),
            resources: data.to_path_buf(),
            vault,
        }
    }

    pub fn resources(&self) -> &Path {
        &self.resources
    }

    pub fn vault(&self) -> &dyn Vault {
        self.vault.as_ref()
    }

    pub fn provider(&self) -> PathBuf {
        self.config.join("provider.yaml")
    }

    pub fn settings(&self) -> PathBuf {
        self.config.join("settings.yaml")
    }

    pub fn search(&self) -> PathBuf {
        self.data.join("search.yaml")
    }

    pub fn llm_log(&self) -> PathBuf {
        self.data.join("llm-log")
    }

    pub fn unpacked(&self) -> PathBuf {
        self.data.join("unpacked")
    }

    pub fn registry(&self) -> PathBuf {
        self.config.join("registry.yaml")
    }

    pub fn library(&self) -> Library {
        Library::at(&self.data)
    }
}

pub fn of(app: &tauri::AppHandle) -> Result<Context, IpcError> {
    let home = app
        .path()
        .home_dir()
        .map_err(|error| IpcError::unreadable("home-dir", &error.to_string()))?;
    let places = layout::places(&home);
    for room in [&places.config, &places.data] {
        std::fs::create_dir_all(room)
            .map_err(|error| IpcError::unwritable(room, &error.to_string()))?;
    }
    if let Ok(old) = app.path().app_data_dir() {
        layout::migrate(&old, &places)
            .map_err(|error| IpcError::unwritable(&places.data, &error.to_string()))?;
    }
    let context = Context::split(&places.config, &places.data);
    Ok(match app.path().resource_dir() {
        Ok(resources) => context.shipped(&resources),
        Err(_) => context,
    })
}
