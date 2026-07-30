use std::path::Path;

use super::error::StoreError;

pub(super) fn weigh(path: &Path) -> Result<u64, StoreError> {
    let facts = std::fs::metadata(path).map_err(StoreError::Unwritable)?;
    if facts.is_file() {
        return Ok(facts.len());
    }
    let mut total = 0;
    let mut stack = vec![path.to_path_buf()];
    while let Some(directory) = stack.pop() {
        let entries = std::fs::read_dir(&directory).map_err(StoreError::Unwritable)?;
        for entry in entries {
            let entry = entry.map_err(StoreError::Unwritable)?;
            let facts = entry.metadata().map_err(StoreError::Unwritable)?;
            if facts.is_dir() {
                stack.push(entry.path());
            } else {
                total += facts.len();
            }
        }
    }
    Ok(total)
}

pub(super) fn erase(path: &Path) -> Result<(), StoreError> {
    let gone = if path.is_dir() {
        std::fs::remove_dir_all(path)
    } else {
        std::fs::remove_file(path)
    };
    match gone {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(StoreError::Unwritable(error)),
    }
}
