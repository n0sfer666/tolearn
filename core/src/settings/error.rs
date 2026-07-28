use std::fmt;
use std::io;

use crate::yaml::ParseError;

#[derive(Debug)]
pub enum SettingsError {
    Unreadable(io::Error),
    Malformed(ParseError),
    Unwritable(io::Error),
}

impl fmt::Display for SettingsError {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreadable(error) => write!(out, "настройки не прочитаны: {error}"),
            Self::Malformed(error) => write!(out, "настройки не разобраны: {error}"),
            Self::Unwritable(error) => write!(out, "настройки не записаны: {error}"),
        }
    }
}

impl std::error::Error for SettingsError {}
