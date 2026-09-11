use std::fmt;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

use super::refusal::Refusal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LibraryError {
    Unreadable { path: PathBuf, kind: ErrorKind },
    Unwritable { path: PathBuf, kind: ErrorKind },
    Absent { uuid: String },
    Taken { uuid: String },
    Refused(Refusal),
}

impl LibraryError {
    pub(super) fn unreadable(path: &Path, error: &io::Error) -> Self {
        Self::Unreadable {
            path: path.to_owned(),
            kind: error.kind(),
        }
    }

    pub(super) fn unwritable(path: &Path, error: &io::Error) -> Self {
        Self::Unwritable {
            path: path.to_owned(),
            kind: error.kind(),
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Self::Unreadable { .. } => "library.unreadable",
            Self::Unwritable { .. } => "library.unwritable",
            Self::Absent { .. } => "library.absent",
            Self::Taken { .. } => "library.taken",
            Self::Refused(refusal) => refusal.code(),
        }
    }
}

impl fmt::Display for LibraryError {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreadable { path, kind } => {
                write!(out, "`{}` cannot be read: {kind}", path.display())
            }
            Self::Unwritable { path, kind } => {
                write!(out, "`{}` cannot be written: {kind}", path.display())
            }
            Self::Absent { uuid } => write!(out, "the library holds no program `{uuid}`"),
            Self::Taken { uuid } => {
                write!(out, "the library already holds the program `{uuid}`")
            }
            Self::Refused(refusal) => write!(out, "{refusal}"),
        }
    }
}

impl std::error::Error for LibraryError {}
