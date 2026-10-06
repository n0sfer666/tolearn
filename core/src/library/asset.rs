use std::fs;

use super::{Library, LibraryError};
use crate::program::Tree;

impl Library {
    pub fn asset(&self, tree: &Tree, file: &str) -> Result<Vec<u8>, LibraryError> {
        let foreign = || LibraryError::Foreign {
            uuid: tree.program.uuid.clone(),
            file: file.to_owned(),
        };
        if !tree.files().iter().any(|listed| listed == file) {
            return Err(foreign());
        }
        let home = self.root.join(&tree.program.uuid);
        let path = home.join(file);
        let real =
            fs::canonicalize(&path).map_err(|error| LibraryError::unreadable(&path, &error))?;
        let walls =
            fs::canonicalize(&home).map_err(|error| LibraryError::unreadable(&home, &error))?;
        if !real.starts_with(&walls) {
            return Err(foreign());
        }
        fs::read(&real).map_err(|error| LibraryError::unreadable(&path, &error))
    }
}
