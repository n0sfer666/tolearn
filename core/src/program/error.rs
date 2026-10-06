use std::fmt;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use crate::yaml::ParseError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadError {
    Unreadable { path: PathBuf, kind: ErrorKind },
    Malformed { path: PathBuf, error: ParseError },
}

impl LoadError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Unreadable { .. } => "program.unreadable",
            Self::Malformed { error, .. } => error.code(),
        }
    }

    pub fn path(&self) -> &Path {
        match self {
            Self::Unreadable { path, .. } | Self::Malformed { path, .. } => path,
        }
    }

    pub fn line(&self) -> Option<usize> {
        match self {
            Self::Unreadable { .. } => None,
            Self::Malformed { error, .. } => Some(error.line()),
        }
    }
}

impl fmt::Display for LoadError {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreadable { path, kind } => {
                write!(out, "`{}` cannot be read: {kind}", path.display())
            }
            Self::Malformed { path, error } => write!(out, "`{}`: {error}", path.display()),
        }
    }
}

impl std::error::Error for LoadError {}
