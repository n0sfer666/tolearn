use std::fs;
use std::path::Path;

use super::error::LibraryError;
use super::examine::read;
use super::install::copy;
use crate::ticket::ticket;

pub(super) fn replace(root: &Path, source: &Path) -> Result<String, LibraryError> {
    let tree = read(source).map_err(LibraryError::Refused)?;
    let uuid = tree.program.uuid.clone();
    let target = root.join(&uuid);
    if !target.is_dir() {
        return Err(LibraryError::Absent { uuid });
    }
    let ticket = ticket();
    let temporary = root.join(format!(".{uuid}.{ticket}.tmp"));
    let old = root.join(format!(".{uuid}.{ticket}.old"));
    let swapped = copy(&tree, source, &temporary).and_then(|()| swap(&target, &temporary, &old));
    let _ = fs::remove_dir_all(&temporary);
    if target.is_dir() {
        let _ = fs::remove_dir_all(&old);
    }
    swapped.map(|()| uuid)
}

fn swap(target: &Path, temporary: &Path, old: &Path) -> Result<(), LibraryError> {
    fs::rename(target, old).map_err(|error| LibraryError::unwritable(target, &error))?;
    fs::rename(temporary, target).map_err(|error| {
        let _ = fs::rename(old, target);
        LibraryError::unwritable(target, &error)
    })
}
