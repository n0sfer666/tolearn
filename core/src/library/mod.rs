mod asset;
mod entry;
mod error;
mod examine;
mod install;
mod refusal;

pub use entry::Entry;
pub use error::LibraryError;
pub use examine::read;
pub use refusal::Refusal;

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use crate::program::Tree;
use crate::yaml::is_uuid;

pub const PROGRAMS: &str = "programs";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Library {
    root: PathBuf,
}

impl Library {
    pub fn at(data: &Path) -> Self {
        Self {
            root: data.join(PROGRAMS),
        }
    }

    pub(crate) fn root(&self) -> &Path {
        &self.root
    }

    pub fn list(&self) -> Result<Vec<Entry>, LibraryError> {
        let listing = match fs::read_dir(&self.root) {
            Ok(listing) => listing,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(LibraryError::unreadable(&self.root, &error)),
        };
        let mut entries = Vec::new();
        for item in listing {
            let path = item
                .map_err(|error| LibraryError::unreadable(&self.root, &error))?
                .path();
            let Some(name) = path.file_name() else {
                continue;
            };
            let directory = name.to_string_lossy().into_owned();
            if directory.starts_with('.') || !path.is_dir() {
                continue;
            }
            entries.push(Entry {
                program: examine::examine(&path, &directory),
                directory,
            });
        }
        entries.sort_by(|left, right| left.directory.cmp(&right.directory));
        Ok(entries)
    }

    pub fn open(&self, uuid: &str) -> Result<Tree, LibraryError> {
        let directory = self.root.join(uuid);
        if !is_uuid(uuid) || !directory.is_dir() {
            return Err(LibraryError::Absent {
                uuid: uuid.to_owned(),
            });
        }
        examine::examine(&directory, uuid).map_err(LibraryError::Refused)
    }

    pub fn install(&self, source: &Path) -> Result<String, LibraryError> {
        install::install(&self.root, source)
    }

    pub fn holds(&self, path: &Path) -> bool {
        crate::export::inside(&self.root, path)
    }
}
