use std::fmt;
use std::io::ErrorKind;
use std::path::PathBuf;

use crate::library::Refusal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackError {
    Refused(Refusal),
    Unreadable { path: PathBuf, kind: ErrorKind },
    Unwritable { reason: String },
}

impl PackError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Refused(refusal) => refusal.code(),
            Self::Unreadable { .. } => "package.unreadable",
            Self::Unwritable { .. } => "package.unwritable",
        }
    }

    pub(super) fn unwritable(error: impl fmt::Display) -> Self {
        Self::Unwritable {
            reason: error.to_string(),
        }
    }
}

impl fmt::Display for PackError {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Refused(refusal) => write!(out, "{refusal}"),
            Self::Unreadable { path, kind } => {
                write!(out, "`{}` cannot be read: {kind}", path.display())
            }
            Self::Unwritable { reason } => write!(out, "the package cannot be written: {reason}"),
        }
    }
}

impl std::error::Error for PackError {}
