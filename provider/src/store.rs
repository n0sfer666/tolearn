use std::io::ErrorKind;
use std::path::Path;

use tolearn_core::atomic;

use crate::error::ProviderError;
use crate::types::Provider;
use crate::{parse, render};

impl Provider {
    pub fn read(path: &Path) -> Result<Self, ProviderError> {
        let source = match std::fs::read_to_string(path) {
            Ok(source) => source,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Self::default()),
            Err(error) => return Err(ProviderError::Unreadable(error)),
        };
        parse::provider(&source).map_err(ProviderError::Malformed)
    }

    pub fn save(&self, path: &Path) -> Result<(), ProviderError> {
        atomic::write(path, &render::text(self)).map_err(ProviderError::Unwritable)
    }
}
