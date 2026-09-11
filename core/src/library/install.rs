use std::fs;
use std::path::Path;

use super::error::LibraryError;
use super::examine::read;
use crate::atomic::spill;
use crate::program::Tree;

pub(super) fn install(root: &Path, source: &Path) -> Result<String, LibraryError> {
    let tree = read(source).map_err(LibraryError::Refused)?;
    let uuid = tree.program.uuid.clone();
    let target = root.join(&uuid);
    if fs::symlink_metadata(&target).is_ok() {
        return Err(LibraryError::Taken { uuid });
    }
    let temporary = root.join(format!(".{uuid}.{}.tmp", std::process::id()));
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
    let stages = tree.stages.keys().map(|id| format!("stages/{id}.yaml"));
    let files = std::iter::once("program.yaml".to_owned())
        .chain(stages)
        .chain(tree.assets.iter().cloned());
    for file in files {
        carry(&from.join(&file), &to.join(&file))?;
    }
    for (uuid, child) in &tree.children {
        let place = Path::new("children").join(uuid);
        copy(child, &from.join(&place), &to.join(&place))?;
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
