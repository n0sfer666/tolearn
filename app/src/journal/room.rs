use std::path::{Path, PathBuf};

use super::record::EXTENSION;

pub fn records(room: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(room) else {
        return Vec::new();
    };
    let mut found: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|kind| kind == EXTENSION))
        .collect();
    found.sort();
    found
}

pub fn rotate(room: &Path, keep: usize) {
    let found = records(room);
    let Some(extra) = found.len().checked_sub(keep) else {
        return;
    };
    for path in found.iter().take(extra) {
        let _ = std::fs::remove_file(path);
    }
}

pub fn clear(room: &Path) {
    for path in records(room) {
        let _ = std::fs::remove_file(path);
    }
}
