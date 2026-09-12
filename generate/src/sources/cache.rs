use std::fs;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde::de::DeserializeOwned;
use tolearn_offline::digest::digest;

use crate::error::GenerateError;

pub const CACHE: &str = "cache";

#[derive(Debug, Clone)]
pub struct Cache {
    root: PathBuf,
}

impl Cache {
    pub fn at(data: &Path, program: &str) -> Self {
        Self {
            root: data.join(CACHE).join(program),
        }
    }

    pub fn load<T: DeserializeOwned>(
        &self,
        kind: &str,
        key: &str,
    ) -> Result<Option<T>, GenerateError> {
        let path = self.spot(kind, key);
        match fs::read(&path) {
            Ok(bytes) => Ok(serde_json::from_slice(&bytes).ok()),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
            Err(error) => Err(failed(&path, &error)),
        }
    }

    pub fn save<T: Serialize>(
        &self,
        kind: &str,
        key: &str,
        value: &T,
    ) -> Result<(), GenerateError> {
        let directory = self.root.join(kind);
        fs::create_dir_all(&directory).map_err(|error| failed(&directory, &error))?;
        let path = self.spot(kind, key);
        let bytes =
            serde_json::to_vec(value).map_err(|error| GenerateError::Cache(error.to_string()))?;
        let temporary = path.with_extension("tmp");
        fs::write(&temporary, bytes).map_err(|error| failed(&temporary, &error))?;
        fs::rename(&temporary, &path).map_err(|error| failed(&path, &error))
    }

    pub fn forget(&self, kind: &str, key: &str) -> Result<(), GenerateError> {
        let path = self.spot(kind, key);
        match fs::remove_file(&path) {
            Err(error) if error.kind() != ErrorKind::NotFound => Err(failed(&path, &error)),
            _ => Ok(()),
        }
    }

    fn spot(&self, kind: &str, key: &str) -> PathBuf {
        self.root
            .join(kind)
            .join(format!("{}.json", digest(key.as_bytes())))
    }
}

fn failed(path: &Path, error: &io::Error) -> GenerateError {
    GenerateError::Cache(format!("{}: {error}", path.display()))
}
