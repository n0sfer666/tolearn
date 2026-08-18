use std::path::{Path, PathBuf};

use tolearn_core::history::{Kept, prune};
use tolearn_core::settings::Settings;

use super::error::IpcError;

const STAMP: &str = "saved-at";

pub fn keep(
    store: &Path,
    bundle: &Path,
    saved_at: &str,
    settings: &Settings,
) -> Result<(), IpcError> {
    if settings.history_depth > 0 {
        let next = versions(store)?
            .iter()
            .map(|kept| kept.n)
            .max()
            .unwrap_or(0)
            + 1;
        let room = store.join(next.to_string());
        copy(bundle, &room)?;
        write(&room.join(STAMP), saved_at)?;
    }
    for n in prune(
        &versions(store)?,
        settings.history_depth,
        settings.history_bytes(),
    ) {
        let room = store.join(n.to_string());
        std::fs::remove_dir_all(&room)
            .map_err(|error| IpcError::unwritable(&room, &error.to_string()))?;
    }
    Ok(())
}

pub fn versions(store: &Path) -> Result<Vec<Kept>, IpcError> {
    let Ok(entries) = std::fs::read_dir(store) else {
        return Ok(Vec::new());
    };
    let mut kept: Vec<Kept> = entries
        .flatten()
        .filter_map(|entry| version(&entry.path()))
        .collect();
    kept.sort_by_key(|version| version.n);
    Ok(kept)
}

pub fn room(store: &Path, n: u32) -> Option<PathBuf> {
    let room = store.join(n.to_string());
    room.is_dir().then_some(room)
}

fn version(room: &Path) -> Option<Kept> {
    let n: u32 = room.file_name()?.to_str()?.parse().ok()?;
    Some(Kept {
        n,
        saved_at: std::fs::read_to_string(room.join(STAMP))
            .unwrap_or_default()
            .trim()
            .to_owned(),
        bytes: weight(room),
    })
}

fn weight(room: &Path) -> u64 {
    let Ok(entries) = std::fs::read_dir(room) else {
        return 0;
    };
    entries
        .flatten()
        .map(|entry| {
            let path = entry.path();
            if path.is_dir() {
                return weight(&path);
            }
            entry.metadata().map(|data| data.len()).unwrap_or(0)
        })
        .sum()
}

fn copy(from: &Path, to: &Path) -> Result<(), IpcError> {
    std::fs::create_dir_all(to).map_err(|error| IpcError::unwritable(to, &error.to_string()))?;
    let entries = std::fs::read_dir(from)
        .map_err(|error| IpcError::unreadable(&from.display().to_string(), &error.to_string()))?;
    for entry in entries.flatten() {
        let path = entry.path();
        if kept_out(&path) {
            continue;
        }
        let target = to.join(entry.file_name());
        if path.is_dir() {
            copy(&path, &target)?;
            continue;
        }
        std::fs::copy(&path, &target)
            .map_err(|error| IpcError::unwritable(&target, &error.to_string()))?;
    }
    Ok(())
}

fn kept_out(path: &Path) -> bool {
    path.file_stem().is_some_and(|stem| stem == "progress")
}

fn write(path: &Path, body: &str) -> Result<(), IpcError> {
    std::fs::write(path, body).map_err(|error| IpcError::unwritable(path, &error.to_string()))
}
