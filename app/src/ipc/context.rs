use std::path::{Path, PathBuf};
use std::sync::Arc;

use tauri::Manager;
use tolearn_provider::{Keychain, Remembered, Vault};

use super::error::IpcError;
use super::layout;

const SERVICE: &str = "tolearn";
const ACCOUNT: &str = "provider";
const NOTES: &str = "notes";

#[derive(Debug, Clone)]
pub struct Context {
    config: PathBuf,
    data: PathBuf,
    vault: Arc<dyn Vault>,
    keys: Arc<dyn Vault>,
}

impl Context {
    pub fn new(data: &Path) -> Self {
        Self::split(data, data)
    }

    pub fn split(config: &Path, data: &Path) -> Self {
        Self {
            config: config.to_path_buf(),
            data: data.to_path_buf(),
            vault: Arc::new(Keychain::new(SERVICE, ACCOUNT)),
            keys: Arc::new(Keychain::new(SERVICE, NOTES)),
        }
    }

    pub fn with_vault(data: &Path, vault: Arc<dyn Vault>) -> Self {
        Self::with_vaults(data, vault, Arc::new(Remembered::default()))
    }

    pub fn with_vaults(data: &Path, vault: Arc<dyn Vault>, keys: Arc<dyn Vault>) -> Self {
        Self {
            config: data.to_path_buf(),
            data: data.to_path_buf(),
            vault,
            keys,
        }
    }

    pub fn vault(&self) -> &dyn Vault {
        self.vault.as_ref()
    }

    pub fn keys(&self) -> &dyn Vault {
        self.keys.as_ref()
    }

    pub fn provider(&self) -> PathBuf {
        self.config.join("provider.yaml")
    }

    pub fn notes(&self) -> PathBuf {
        self.data.join("notes")
    }

    pub fn settings(&self) -> PathBuf {
        self.config.join("settings.yaml")
    }

    pub fn search(&self, roadmap: &str) -> PathBuf {
        self.data.join(format!("search-{roadmap}.yaml"))
    }

    pub fn history(&self, roadmap: &str) -> PathBuf {
        self.data.join("history").join(roadmap)
    }

    pub fn dialogs(&self) -> PathBuf {
        self.data.join("dialogs")
    }

    pub fn offline(&self) -> PathBuf {
        self.data.join("offline")
    }

    pub fn unpacked(&self) -> PathBuf {
        self.data.join("unpacked")
    }

    pub fn registry(&self) -> PathBuf {
        self.config.join("registry.yaml")
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
    Ok(Context::split(&places.config, &places.data))
}
