use std::path::Path;

use flate2::read::GzDecoder;
use tar::Archive;

use super::error::ArchiveError;
use super::guard::place;
use super::limits::Tally;
use super::pour::pour;

pub fn spread(archive: &Path, into: &Path, tally: &mut Tally<'_>) -> Result<(), ArchiveError> {
    let file = std::fs::File::open(archive).map_err(|error| ArchiveError::Unreadable {
        path: archive.to_owned(),
        kind: error.kind(),
    })?;
    let mut tape = Archive::new(GzDecoder::new(file));
    for entry in tape.entries().map_err(damaged)? {
        let mut entry = entry.map_err(damaged)?;
        let kind = entry.header().entry_type();
        let name = named(&entry)?;
        if kind.is_symlink() || kind.is_hard_link() {
            return Err(ArchiveError::Link { entry: name });
        }
        if kind.is_dir() {
            continue;
        }
        if !kind.is_file() {
            return Err(ArchiveError::Link { entry: name });
        }
        tally.entry()?;
        let path = place(into, &name)?;
        pour(&mut entry, &path, tally)?;
    }
    Ok(())
}

fn named<R: std::io::Read>(entry: &tar::Entry<'_, R>) -> Result<String, ArchiveError> {
    entry
        .path()
        .map(|path| path.display().to_string())
        .map_err(damaged)
}

fn damaged(error: std::io::Error) -> ArchiveError {
    ArchiveError::Damaged {
        reason: error.to_string(),
    }
}
