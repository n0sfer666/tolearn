use std::io::{Read, Write};
use std::path::Path;

use super::error::ArchiveError;
use super::limits::Tally;

const CHUNK: usize = 64 << 10;

pub fn pour(
    source: &mut impl Read,
    path: &Path,
    tally: &mut Tally<'_>,
) -> Result<(), ArchiveError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| unwritable(parent, &error))?;
    }
    let mut file = std::fs::File::create(path).map_err(|error| unwritable(path, &error))?;
    let mut chunk = vec![0; CHUNK];
    loop {
        let taken = source
            .read(&mut chunk)
            .map_err(|error| ArchiveError::Damaged {
                reason: error.to_string(),
            })?;
        if taken == 0 {
            return Ok(());
        }
        tally.grow(taken as u64)?;
        file.write_all(&chunk[..taken])
            .map_err(|error| unwritable(path, &error))?;
    }
}

fn unwritable(path: &Path, error: &std::io::Error) -> ArchiveError {
    ArchiveError::Unwritable {
        path: path.to_owned(),
        kind: error.kind(),
    }
}
