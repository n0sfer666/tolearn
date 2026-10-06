use std::error::Error;
use std::fmt;

use crate::page::PageError;

#[derive(Debug)]
pub enum BookError {
    Unreachable(PageError),
    Malformed(String),
}

impl fmt::Display for BookError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreachable(error) => write!(f, "Open Library недоступна: {error}"),
            Self::Malformed(reason) => write!(f, "Open Library ответила не по формату: {reason}"),
        }
    }
}

impl Error for BookError {}
