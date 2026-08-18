mod error;
mod parse;
mod render;
mod types;

pub use error::SettingsError;
pub use types::{
    DEFAULT_BUDGET_MB, DEFAULT_HISTORY_DEPTH, DEFAULT_HISTORY_SHARE, Locale, Settings, Theme,
};

use std::io::ErrorKind;
use std::path::Path;

use crate::atomic;

impl Settings {
    pub fn read(path: &Path) -> Result<Self, SettingsError> {
        let source = match std::fs::read_to_string(path) {
            Ok(source) => source,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Self::default()),
            Err(error) => return Err(SettingsError::Unreadable(error)),
        };
        parse::settings(&source).map_err(SettingsError::Malformed)
    }

    pub fn save(&self, path: &Path) -> Result<(), SettingsError> {
        atomic::write(path, &render::text(self)).map_err(SettingsError::Unwritable)
    }
}
