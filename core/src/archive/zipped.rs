use std::path::Path;

use zip::ZipArchive;

use super::error::ArchiveError;
use super::guard::place;
use super::limits::Tally;
use super::pour::pour;

pub fn spread(archive: &Path, into: &Path, tally: &mut Tally<'_>) -> Result<(), ArchiveError> {
    let file = std::fs::File::open(archive).map_err(|error| ArchiveError::Unreadable {
        path: archive.to_owned(),
        kind: error.kind(),
    })?;
    let mut zip = ZipArchive::new(file).map_err(damaged)?;
    for index in 0..zip.len() {
        let mut entry = zip.by_index(index).map_err(damaged)?;
        let name = entry.name().to_owned();
        if entry.is_symlink() {
            return Err(ArchiveError::Link { entry: name });
        }
        if entry.is_dir() {
            continue;
        }
        tally.entry()?;
        let path = place(into, &name)?;
        pour(&mut entry, &path, tally)?;
    }
    Ok(())
}

fn damaged(error: zip::result::ZipError) -> ArchiveError {
    ArchiveError::Damaged {
        reason: error.to_string(),
    }
}
