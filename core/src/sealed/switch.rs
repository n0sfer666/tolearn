use std::ffi::OsString;
use std::path::{Path, PathBuf};

use crate::notes::{index, save};

use super::error::SealError;
use super::keys::Keys;
use super::store::{IDENTITY, Sealed};

const SWITCHING: &str = ".switching";
const RETIRED: &str = ".retired";

pub fn sealed(root: &Path) -> bool {
    root.join(IDENTITY).is_file()
}

pub fn keys(root: &Path, phrase: &str) -> Result<Keys, SealError> {
    let path = root.join(IDENTITY);
    let cipher = std::fs::read(&path).map_err(|error| SealError::Unreadable {
        path: path.display().to_string(),
        reason: error.to_string(),
    })?;
    Keys::unwrapped(&cipher, phrase)
}

pub fn settle(root: &Path) -> Result<(), SealError> {
    let retired = beside(root, RETIRED);
    if retired.exists() {
        if root.exists() {
            drop_all(&retired)?;
        } else {
            moved(&retired, root)?;
        }
    }
    let switching = beside(root, SWITCHING);
    if switching.exists() {
        drop_all(&switching)?;
    }
    Ok(())
}

pub fn lock(root: &Path, phrase: &str) -> Result<Keys, SealError> {
    settle(root)?;
    let keys = Keys::generate();
    let switching = beside(root, SWITCHING);
    fresh(&switching)?;

    let store = Sealed::new(&switching, keys.clone());
    for note in index(root).map_err(|error| SealError::Unreadable {
        path: root.display().to_string(),
        reason: error.to_string(),
    })? {
        store.save(&note.roadmap, &note.topic, &note.body, None)?;
    }
    let wrapped = switching.join(IDENTITY);
    crate::atomic::bytes(&wrapped, &keys.wrapped(phrase)?).map_err(|error| {
        SealError::Unwritable {
            path: wrapped.display().to_string(),
            reason: error.to_string(),
        }
    })?;

    swap(root, &switching)?;
    Ok(keys)
}

pub fn unlock(root: &Path, keys: &Keys) -> Result<(), SealError> {
    settle(root)?;
    let switching = beside(root, SWITCHING);
    fresh(&switching)?;

    for note in Sealed::new(root, keys.clone()).index()? {
        save(&switching, &note.roadmap, &note.topic, &note.body, None).map_err(|error| {
            SealError::Unwritable {
                path: switching.display().to_string(),
                reason: error.to_string(),
            }
        })?;
    }

    swap(root, &switching)
}

fn swap(root: &Path, switching: &Path) -> Result<(), SealError> {
    let retired = beside(root, RETIRED);
    if root.exists() {
        moved(root, &retired)?;
    }
    moved(switching, root)?;
    if retired.exists() {
        drop_all(&retired)?;
    }
    Ok(())
}

fn beside(root: &Path, suffix: &str) -> PathBuf {
    let mut name = root.file_name().map(OsString::from).unwrap_or_default();
    name.push(suffix);
    root.with_file_name(name)
}

fn fresh(path: &Path) -> Result<(), SealError> {
    std::fs::create_dir_all(path).map_err(|error| SealError::Unwritable {
        path: path.display().to_string(),
        reason: error.to_string(),
    })
}

fn moved(from: &Path, to: &Path) -> Result<(), SealError> {
    std::fs::rename(from, to).map_err(|error| SealError::Unwritable {
        path: to.display().to_string(),
        reason: error.to_string(),
    })
}

fn drop_all(path: &Path) -> Result<(), SealError> {
    std::fs::remove_dir_all(path).map_err(|error| SealError::Unwritable {
        path: path.display().to_string(),
        reason: error.to_string(),
    })
}
