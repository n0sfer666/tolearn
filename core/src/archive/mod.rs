mod error;
mod guard;
mod limits;
mod pour;
mod root;
mod tarball;
mod zipped;

pub use error::ArchiveError;
pub use limits::Limits;

use std::io::Read;
use std::path::{Path, PathBuf};

use limits::Tally;

pub fn unpack(archive: &Path, into: &Path, limits: &Limits) -> Result<PathBuf, ArchiveError> {
    match spread(archive, into, limits) {
        Ok(root) => Ok(root),
        Err(error) => {
            let _ = std::fs::remove_dir_all(into);
            Err(error)
        }
    }
}

fn spread(archive: &Path, into: &Path, limits: &Limits) -> Result<PathBuf, ArchiveError> {
    let packed = std::fs::metadata(archive)
        .map_err(|error| ArchiveError::Unreadable {
            path: archive.to_owned(),
            kind: error.kind(),
        })?
        .len();
    let mut tally = Tally::new(limits, packed);
    std::fs::create_dir_all(into).map_err(|error| ArchiveError::Unwritable {
        path: into.to_owned(),
        kind: error.kind(),
    })?;
    match kind(archive)? {
        Kind::Zip => zipped::spread(archive, into, &mut tally)?,
        Kind::Gzip => tarball::spread(archive, into, &mut tally)?,
    }
    guard::confined(into)?;
    root::root(into)
}

enum Kind {
    Zip,
    Gzip,
}

fn kind(archive: &Path) -> Result<Kind, ArchiveError> {
    let mut head = [0; 2];
    let mut file = std::fs::File::open(archive).map_err(|error| ArchiveError::Unreadable {
        path: archive.to_owned(),
        kind: error.kind(),
    })?;
    file.read_exact(&mut head)
        .map_err(|_| ArchiveError::UnknownFormat)?;
    match head {
        [0x50, 0x4b] => Ok(Kind::Zip),
        [0x1f, 0x8b] => Ok(Kind::Gzip),
        _ => Err(ArchiveError::UnknownFormat),
    }
}
