use std::fmt;
use std::io;

use crate::yaml::ParseError;

#[derive(Debug)]
pub enum RegistryError {
    Unreadable(io::Error),
    Malformed(ParseError),
    Unwritable(io::Error),
}

impl fmt::Display for RegistryError {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreadable(error) => write!(out, "реестр программ не прочитан: {error}"),
            Self::Malformed(error) => write!(out, "реестр программ не разобран: {error}"),
            Self::Unwritable(error) => write!(out, "реестр программ не записан: {error}"),
        }
    }
}

impl std::error::Error for RegistryError {}
