use std::error::Error;
use std::fmt;

use crate::page::PageError;

#[derive(Debug)]
pub enum CommonsError {
    Unreachable(PageError),
    Malformed(String),
}

impl fmt::Display for CommonsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreachable(error) => write!(f, "Wikimedia Commons недоступна: {error}"),
            Self::Malformed(reason) => {
                write!(f, "Wikimedia Commons ответила не по формату: {reason}")
            }
        }
    }
}

impl Error for CommonsError {}
