use std::fs;
use std::io::{self, ErrorKind};
use std::path::Path;

use super::scratch::scratch;
use super::unpack_error::UnpackError;
use super::{MANIFEST, verify};
use crate::program::Tree;

pub fn unpack(package: &Path, into: &Path) -> Result<Tree, UnpackError> {
    vacant(into)?;
    let scratch = scratch(above(into), &format!("{}.", name(into)))?;
    let unpacked =
        verify::open(package, &scratch).and_then(|tree| settle(&scratch, into).map(|()| tree));
    let _ = fs::remove_dir_all(&scratch);
    unpacked
}

fn vacant(into: &Path) -> Result<(), UnpackError> {
    let occupied = || UnpackError::Occupied {
        path: into.to_owned(),
    };
    match fs::read_dir(into) {
        Ok(mut listing) => match listing.next() {
            None => Ok(()),
            Some(_) => Err(occupied()),
        },
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) if error.kind() == ErrorKind::NotADirectory => Err(occupied()),
        Err(error) => Err(UnpackError::Unreadable {
            path: into.to_owned(),
            kind: error.kind(),
        }),
    }
}

fn above(into: &Path) -> &Path {
    into.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or(Path::new("."))
}

fn name(into: &Path) -> String {
    into.file_name().map_or_else(
        || "program".to_owned(),
        |name| name.to_string_lossy().into_owned(),
    )
}

fn settle(scratch: &Path, into: &Path) -> Result<(), UnpackError> {
    fs::remove_file(scratch.join(MANIFEST))
        .map_err(|error| unwritable(Path::new(MANIFEST), &error))?;
    if fs::rename(scratch, into).is_ok() {
        return Ok(());
    }
    let emptied = fs::remove_dir(into).is_ok();
    fs::rename(scratch, into).map_err(|error| {
        if emptied {
            let _ = fs::create_dir(into);
        }
        unwritable(into, &error)
    })
}

fn unwritable(path: &Path, error: &io::Error) -> UnpackError {
    UnpackError::Unwritable {
        path: path.to_owned(),
        kind: error.kind(),
    }
}
