use std::path::{Component, Path, PathBuf};

use super::error::ArchiveError;

pub fn place(into: &Path, entry: &str) -> Result<PathBuf, ArchiveError> {
    let named = Path::new(entry);
    let mut path = into.to_path_buf();
    for part in named.components() {
        match part {
            Component::Normal(part) => path.push(part),
            Component::CurDir => {}
            _ => {
                return Err(ArchiveError::Escaping {
                    entry: entry.to_owned(),
                });
            }
        }
    }
    if path == into {
        return Err(ArchiveError::Escaping {
            entry: entry.to_owned(),
        });
    }
    Ok(path)
}

pub fn confined(into: &Path) -> Result<(), ArchiveError> {
    let root = into
        .canonicalize()
        .map_err(|error| ArchiveError::Unreadable {
            path: into.to_owned(),
            kind: error.kind(),
        })?;
    walk(&root, &root)
}

fn walk(root: &Path, room: &Path) -> Result<(), ArchiveError> {
    let listing = std::fs::read_dir(room).map_err(|error| ArchiveError::Unreadable {
        path: room.to_owned(),
        kind: error.kind(),
    })?;
    for entry in listing.flatten() {
        let path = entry.path();
        let real = path
            .canonicalize()
            .map_err(|error| ArchiveError::Unreadable {
                path: path.clone(),
                kind: error.kind(),
            })?;
        if !real.starts_with(root) {
            return Err(ArchiveError::Escaping {
                entry: path.display().to_string(),
            });
        }
        if real.is_dir() {
            walk(root, &real)?;
        }
    }
    Ok(())
}
