use std::path::{Path, PathBuf};
use std::sync::Arc;

use tauri::Manager;
use tolearn_core::library::Library;
use tolearn_generate::REACH_TIMEOUT_SECS;
use tolearn_generate::ledger::Tally;
use tolearn_offline::reach::{Ping, Reach};
use tolearn_provider::{Keychain, Vault};

use crate::discard::{Bin, Trash};

use super::error::IpcError;
use super::layout;
use super::ledger::Ledger;
use super::running::Running;
use super::tools::Tools;
use super::wired::wired;

pub type Net = Arc<dyn Reach + Send + Sync>;

#[derive(Debug, Clone)]
pub struct Context {
    config: PathBuf,
    data: PathBuf,
    resources: PathBuf,
    vault: Arc<dyn Vault>,
    reach: Option<Net>,
    tools: Tools,
    running: Running,
    ledger: Ledger,
    bin: Arc<dyn Bin>,
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
            vault: Arc::new(Keychain::app()),
            reach: None,
            tools: Tools::default(),
            running: Running::default(),
            ledger: Ledger::default(),
            bin: Arc::new(Trash::new()),
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
            reach: None,
            tools: Tools::default(),
            running: Running::default(),
            ledger: Ledger::default(),
            bin: Arc::new(Trash::new()),
        }
    }

    pub fn with_reach(mut self, reach: Net) -> Self {
        self.reach = Some(reach);
        self
    }

    pub fn with_tools(mut self, tools: Tools) -> Self {
        self.tools = tools;
        self
    }

    pub fn with_running(mut self, running: Running) -> Self {
        self.running = running;
        self
    }

    pub fn with_ledger(mut self, ledger: Ledger) -> Self {
        self.ledger = ledger;
        self
    }

    pub fn with_bin(mut self, bin: Arc<dyn Bin>) -> Self {
        self.bin = bin;
        self
    }

    pub fn bin(&self) -> &dyn Bin {
        self.bin.as_ref()
    }

    pub fn reach(&self) -> Result<Net, IpcError> {
        if let Some(reach) = &self.reach {
            return Ok(Arc::clone(reach));
        }
        let ping = Ping::new(REACH_TIMEOUT_SECS)
            .map_err(|reason| IpcError::new("generate.offline", reason))?;
        Ok(Arc::new(ping))
    }

    pub fn tools(&self) -> &Tools {
        &self.tools
    }

    pub fn running(&self) -> &Running {
        &self.running
    }

    pub fn ledger(&self) -> &Tally {
        self.ledger.tally()
    }

    pub fn book(&self) -> &Ledger {
        &self.ledger
    }

    pub fn data(&self) -> &Path {
        &self.data
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
        self.data.join(crate::journal::ROOM)
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
    let context = match app.path().resource_dir() {
        Ok(resources) => context.shipped(&resources),
        Err(_) => context,
    };
    wired(app, context)
}
