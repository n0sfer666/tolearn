use std::fmt;

use saphyr::{Marker, ScanError};

use super::failure::ParseFailure;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    failure: ParseFailure,
    line: usize,
    column: usize,
    path: String,
    message: String,
}

impl ParseError {
    pub(crate) fn at(
        failure: ParseFailure,
        marker: Marker,
        path: &str,
        message: impl Into<String>,
    ) -> Self {
        Self {
            failure,
            line: marker.line(),
            column: marker.col() + 1,
            path: path.to_owned(),
            message: message.into(),
        }
    }

    pub(crate) fn at_line(
        failure: ParseFailure,
        line: usize,
        path: &str,
        message: impl Into<String>,
    ) -> Self {
        Self {
            failure,
            line,
            column: 1,
            path: path.to_owned(),
            message: message.into(),
        }
    }

    pub(crate) fn at_start(failure: ParseFailure, message: impl Into<String>) -> Self {
        Self {
            failure,
            line: 1,
            column: 1,
            path: String::new(),
            message: message.into(),
        }
    }

    pub(crate) fn from_scan(error: ScanError) -> Self {
        Self::at(ParseFailure::Syntax, *error.marker(), "", error.info())
    }

    pub fn failure(&self) -> ParseFailure {
        self.failure
    }

    pub fn code(&self) -> &'static str {
        self.failure.code()
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn column(&self) -> usize {
        self.column
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            failure,
            line,
            column,
            path,
            message,
        } = self;
        let code = failure.code();
        if path.is_empty() {
            write!(formatter, "{line}:{column}: {code}: {message}")
        } else {
            write!(formatter, "{line}:{column}: {path}: {code}: {message}")
        }
    }
}

impl std::error::Error for ParseError {}
