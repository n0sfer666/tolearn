use std::fmt;
use std::io::ErrorKind;
use std::path::PathBuf;

use crate::yaml::ParseError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScanError {
    NoRoadmap { root: PathBuf },
    Unreadable { path: PathBuf, kind: ErrorKind },
    Malformed { path: PathBuf, error: ParseError },
}

impl fmt::Display for ScanError {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoRoadmap { root } => write!(
                out,
                "`{}` holds neither `roadmap.yaml` nor `roadmap.json`",
                root.display()
            ),
            Self::Unreadable { path, kind } => {
                write!(out, "`{}` cannot be read: {kind}", path.display())
            }
            Self::Malformed { path, error } => write!(out, "`{}`: {error}", path.display()),
        }
    }
}
