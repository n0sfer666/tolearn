use std::fmt;

use crate::scan::ScanError;
use crate::yaml::ParseError;

#[derive(Debug)]
pub enum SearchError {
    Bundle(ScanError),
    Unreadable { path: String, reason: String },
    Unwritable { path: String, reason: String },
    Malformed { path: String, error: ParseError },
}

impl fmt::Display for SearchError {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bundle(error) => write!(out, "{error}"),
            Self::Unreadable { path, reason } => write!(out, "`{path}` не читается: {reason}"),
            Self::Unwritable { path, reason } => write!(out, "`{path}` не записывается: {reason}"),
            Self::Malformed { path, error } => write!(out, "индекс `{path}` испорчен: {error}"),
        }
    }
}

impl std::error::Error for SearchError {}
