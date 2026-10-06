use std::fs;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde::de::DeserializeOwned;
use tolearn_offline::digest::digest;

use crate::error::GenerateError;

pub const CACHE: &str = "cache";

const JSON: &str = "json";
const RAW: &str = "bin";

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
        let bytes = self.read(kind, key, JSON)?;
        Ok(bytes.and_then(|bytes| serde_json::from_slice(&bytes).ok()))
    }

    pub fn load_bytes(&self, kind: &str, key: &str) -> Result<Option<Vec<u8>>, GenerateError> {
        self.read(kind, key, RAW)
    }

    pub fn save<T: Serialize>(
        &self,
        kind: &str,
        key: &str,
        value: &T,
    ) -> Result<(), GenerateError> {
        let bytes =
            serde_json::to_vec(value).map_err(|error| GenerateError::Cache(error.to_string()))?;
        self.write(kind, key, JSON, &bytes)
    }

    pub fn save_bytes(&self, kind: &str, key: &str, bytes: &[u8]) -> Result<(), GenerateError> {
        self.write(kind, key, RAW, bytes)
    }

    pub fn forget(&self, kind: &str, key: &str) -> Result<(), GenerateError> {
        let path = self.spot(kind, key, JSON);
        match fs::remove_file(&path) {
            Err(error) if error.kind() != ErrorKind::NotFound => Err(failed(&path, &error)),
            _ => Ok(()),
        }
    }

    fn read(
        &self,
        kind: &str,
        key: &str,
        extension: &str,
    ) -> Result<Option<Vec<u8>>, GenerateError> {
        let path = self.spot(kind, key, extension);
        match fs::read(&path) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
            Err(error) => Err(failed(&path, &error)),
        }
    }

    fn write(
        &self,
        kind: &str,
        key: &str,
        extension: &str,
        bytes: &[u8],
    ) -> Result<(), GenerateError> {
        let directory = self.root.join(kind);
        fs::create_dir_all(&directory).map_err(|error| failed(&directory, &error))?;
        let path = self.spot(kind, key, extension);
        let temporary = path.with_extension("tmp");
        fs::write(&temporary, bytes).map_err(|error| failed(&temporary, &error))?;
        fs::rename(&temporary, &path).map_err(|error| failed(&path, &error))
    }

    fn spot(&self, kind: &str, key: &str, extension: &str) -> PathBuf {
        self.root
            .join(kind)
            .join(format!("{}.{extension}", digest(key.as_bytes())))
    }
}

fn failed(path: &Path, error: &io::Error) -> GenerateError {
    GenerateError::Cache(format!("{}: {error}", path.display()))
}
