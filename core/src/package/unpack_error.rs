use std::fmt;
use std::io::ErrorKind;
use std::path::PathBuf;

use super::{MANIFEST, SCHEMA};
use crate::archive::ArchiveError;
use crate::library::{LibraryError, Refusal};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnpackError {
    Archive {
        package: PathBuf,
        error: ArchiveError,
    },
    NoManifest,
    BadManifest {
        reason: String,
    },
    Version {
        found: String,
    },
    Unlisted {
        file: String,
    },
    Missing {
        file: String,
    },
    Checksum {
        file: String,
    },
    Refused(Refusal),
    Unused {
        file: String,
    },
    UnsafeSvg {
        file: String,
        reason: String,
    },
    Unreadable {
        path: PathBuf,
        kind: ErrorKind,
    },
    Unwritable {
        path: PathBuf,
        kind: ErrorKind,
    },
    Occupied {
        path: PathBuf,
    },
    Library(LibraryError),
}

impl UnpackError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Archive { error, .. } => error.code(),
            Self::NoManifest => "package.no-manifest",
            Self::BadManifest { .. } => "package.bad-manifest",
            Self::Version { .. } => "package.version",
            Self::Unlisted { .. } => "package.unlisted",
            Self::Missing { .. } => "package.missing",
            Self::Checksum { .. } => "package.checksum",
            Self::Refused(refusal) => refusal.code(),
            Self::Unused { .. } => "package.unused",
            Self::UnsafeSvg { .. } => "package.unsafe-svg",
            Self::Unreadable { .. } => "package.unreadable",
            Self::Unwritable { .. } => "package.unwritable",
            Self::Occupied { .. } => "package.occupied",
            Self::Library(error) => error.code(),
        }
    }
}

impl fmt::Display for UnpackError {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Archive { package, error } => write!(out, "`{}`: {error}", package.display()),
            Self::NoManifest => write!(
                out,
                "the package holds no `{MANIFEST}`, and without it nothing is known to be whole"
            ),
            Self::BadManifest { reason } => write!(out, "`{MANIFEST}` does not read: {reason}"),
            Self::Version { found } => write!(
                out,
                "`{MANIFEST}` is `{found}`, and this build reads only `{SCHEMA}`"
            ),
            Self::Unlisted { file } => {
                write!(out, "`{file}` is in the package but not in `{MANIFEST}`")
            }
            Self::Missing { file } => write!(
                out,
                "`{MANIFEST}` lists `{file}`, which the package does not hold"
            ),
            Self::Checksum { file } => {
                write!(out, "`{file}` does not match its sha256 in `{MANIFEST}`")
            }
            Self::Refused(refusal) => write!(out, "{refusal}"),
            Self::Unused { file } => write!(
                out,
                "`{file}` is in the package but not a part of the program it holds"
            ),
            Self::UnsafeSvg { file, reason } => {
                write!(out, "`{file}` is not a safe picture: {reason}")
            }
            Self::Unreadable { path, kind } => {
                write!(out, "`{}` cannot be read: {kind}", path.display())
            }
            Self::Unwritable { path, kind } => {
                write!(out, "`{}` cannot be written: {kind}", path.display())
            }
            Self::Occupied { path } => write!(
                out,
                "`{}` is not empty, and a package unpacks only into an empty directory",
                path.display()
            ),
            Self::Library(error) => write!(out, "{error}"),
        }
    }
}

impl std::error::Error for UnpackError {}
