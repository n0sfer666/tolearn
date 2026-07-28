use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

use tolearn_core::archive::{Limits, unpack};

use super::context::Context;
use super::error::IpcError;

static TURN: AtomicU32 = AtomicU32::new(0);

#[derive(Debug)]
pub struct Taken {
    pub root: PathBuf,
    room: Option<PathBuf>,
}

pub fn taken(context: &Context, path: &str) -> Result<Taken, IpcError> {
    let given = Path::new(path);
    if given.is_dir() {
        return Ok(Taken {
            root: given.to_path_buf(),
            room: None,
        });
    }
    let room = context.unpacked().join(format!(
        ".taking-{}-{}",
        std::process::id(),
        TURN.fetch_add(1, Ordering::Relaxed)
    ));
    let root = unpack(given, &room, &Limits::DEFAULT)?;
    Ok(Taken {
        root,
        room: Some(room),
    })
}

pub fn settle(taken: &Taken, home: &Path) -> Result<PathBuf, IpcError> {
    let Some(room) = taken.room.as_deref() else {
        return Ok(taken.root.clone());
    };
    if home.exists() {
        std::fs::remove_dir_all(home)
            .map_err(|error| IpcError::unwritable(home, &error.to_string()))?;
    }
    if let Some(parent) = home.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| IpcError::unwritable(parent, &error.to_string()))?;
    }
    std::fs::rename(&taken.root, home)
        .map_err(|error| IpcError::unwritable(home, &error.to_string()))?;
    let _ = std::fs::remove_dir_all(room);
    Ok(home.to_path_buf())
}

pub fn discard(taken: &Taken) {
    if let Some(room) = taken.room.as_deref() {
        let _ = std::fs::remove_dir_all(room);
    }
}
