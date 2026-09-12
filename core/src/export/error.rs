use std::fmt;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExportError {
    Occupied { path: PathBuf },
    Unwritable { path: PathBuf, kind: ErrorKind },
    Asset { file: String, reason: String },
}

impl ExportError {
    pub(super) fn unwritable(path: &Path, error: &io::Error) -> Self {
        Self::Unwritable {
            path: path.to_owned(),
            kind: error.kind(),
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Self::Occupied { .. } => "export.occupied",
            Self::Unwritable { .. } => "export.unwritable",
            Self::Asset { .. } => "export.asset",
        }
    }
}

impl fmt::Display for ExportError {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Occupied { path } => {
                write!(
                    out,
                    "`{}` is taken and is not an empty folder",
                    path.display()
                )
            }
            Self::Unwritable { path, kind } => {
                write!(out, "`{}` cannot be written: {kind}", path.display())
            }
            Self::Asset { file, reason } => {
                write!(out, "the asset `{file}` cannot be exported: {reason}")
            }
        }
    }
}

impl std::error::Error for ExportError {}
