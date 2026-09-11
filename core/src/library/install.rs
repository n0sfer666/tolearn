use std::fs;
use std::path::Path;

use super::error::LibraryError;
use super::examine::read;
use crate::atomic::spill;
use crate::program::Tree;
use crate::ticket::ticket;

pub(super) fn install(root: &Path, source: &Path) -> Result<String, LibraryError> {
    let tree = read(source).map_err(LibraryError::Refused)?;
    let uuid = tree.program.uuid.clone();
    let target = root.join(&uuid);
    if fs::symlink_metadata(&target).is_ok() {
        return Err(LibraryError::Taken { uuid });
    }
    let temporary = root.join(format!(".{uuid}.{}.tmp", ticket()));
    let _ = fs::remove_dir_all(&temporary);
    let placed = copy(&tree, source, &temporary).and_then(|()| {
        fs::rename(&temporary, &target).map_err(|error| LibraryError::unwritable(&target, &error))
    });
    if placed.is_err() {
        let _ = fs::remove_dir_all(&temporary);
    }
    placed.map(|()| uuid)
}

fn copy(tree: &Tree, from: &Path, to: &Path) -> Result<(), LibraryError> {
    fs::create_dir_all(to).map_err(|error| LibraryError::unwritable(to, &error))?;
    for file in tree.files() {
        carry(&from.join(&file), &to.join(&file))?;
    }
    Ok(())
}

fn carry(from: &Path, to: &Path) -> Result<(), LibraryError> {
    let data = fs::read(from).map_err(|error| LibraryError::unreadable(from, &error))?;
    if let Some(folder) = to.parent() {
        fs::create_dir_all(folder).map_err(|error| LibraryError::unwritable(folder, &error))?;
    }
    spill(to, &data).map_err(|error| LibraryError::unwritable(to, &error))
}
