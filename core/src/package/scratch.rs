use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use super::unpack_error::UnpackError;
use crate::ticket::ticket;

pub(super) fn scratch(beside: &Path, stem: &str) -> Result<PathBuf, UnpackError> {
    loop {
        let path = beside.join(format!(".{stem}{}.unpack", ticket()));
        match fs::create_dir(&path) {
            Ok(()) => return Ok(path),
            Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
            Err(error) => {
                return Err(UnpackError::Unwritable {
                    path: beside.to_owned(),
                    kind: error.kind(),
                });
            }
        }
    }
}
