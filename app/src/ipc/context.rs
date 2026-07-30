use std::path::{Path, PathBuf};
use std::sync::Arc;

use tauri::Manager;
use tolearn_provider::{Keychain, Remembered, Vault};

use super::error::IpcError;

const SERVICE: &str = "tolearn";
const ACCOUNT: &str = "provider";
const NOTES: &str = "notes";

#[derive(Debug, Clone)]
pub struct Context {
    data: PathBuf,
    vault: Arc<dyn Vault>,
    keys: Arc<dyn Vault>,
}

impl Context {
    pub fn new(data: &Path) -> Self {
        Self::with_vaults(
            data,
            Arc::new(Keychain::new(SERVICE, ACCOUNT)),
            Arc::new(Keychain::new(SERVICE, NOTES)),
        )
    }

    pub fn with_vault(data: &Path, vault: Arc<dyn Vault>) -> Self {
        Self::with_vaults(data, vault, Arc::new(Remembered::default()))
    }

    pub fn with_vaults(data: &Path, vault: Arc<dyn Vault>, keys: Arc<dyn Vault>) -> Self {
        Self {
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
        self.data.join("provider.yaml")
    }

    pub fn notes(&self) -> PathBuf {
        self.data.join("notes")
    }

    pub fn settings(&self) -> PathBuf {
        self.data.join("settings.yaml")
    }

    pub fn search(&self, roadmap: &str) -> PathBuf {
        self.data.join(format!("search-{roadmap}.yaml"))
    }

    pub fn history(&self, roadmap: &str) -> PathBuf {
        self.data.join("history").join(roadmap)
    }

    pub fn offline(&self) -> PathBuf {
        self.data.join("offline")
    }

    pub fn unpacked(&self) -> PathBuf {
        self.data.join("unpacked")
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
