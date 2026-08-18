use std::path::{Path, PathBuf};

use super::error::ArchiveError;

const DEEP: usize = 8;

pub fn root(into: &Path) -> Result<PathBuf, ArchiveError> {
    let mut found: Vec<PathBuf> = Vec::new();
    look(into, 0, &mut found)?;
    let shallowest = found
        .iter()
        .map(|path| path.components().count())
        .min()
        .ok_or(ArchiveError::NoBundle)?;
    let mut roots = found
        .into_iter()
        .filter(|path| path.components().count() == shallowest);
    let root = roots.next().ok_or(ArchiveError::NoBundle)?;
    if roots.next().is_some() {
        return Err(ArchiveError::ManyBundles);
    }
    Ok(root)
}

fn look(room: &Path, depth: usize, found: &mut Vec<PathBuf>) -> Result<(), ArchiveError> {
    if depth > DEEP {
        return Ok(());
    }
    if room.join("roadmap.yaml").is_file() || room.join("roadmap.json").is_file() {
        found.push(room.to_owned());
        return Ok(());
    }
    let listing = std::fs::read_dir(room).map_err(|error| ArchiveError::Unreadable {
        path: room.to_owned(),
        kind: error.kind(),
    })?;
    for entry in listing.flatten() {
        let path = entry.path();
        if path.is_dir() {
            look(&path, depth + 1, found)?;
        }
    }
    Ok(())
}
