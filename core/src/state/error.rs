use std::fmt;
use std::io;

use crate::yaml::ParseError;

#[derive(Debug)]
pub enum StateError {
    Stray(String),
    Unreadable(io::Error),
    Malformed(ParseError),
    Foreign(String),
    Unwritable(io::Error),
}

impl fmt::Display for StateError {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Stray(program) => write!(out, "не UUID программы: `{program}`"),
            Self::Unreadable(error) => write!(out, "состояние не прочитано: {error}"),
            Self::Malformed(error) => write!(out, "состояние не разобрано: {error}"),
            Self::Foreign(found) => {
                write!(out, "в файле состояние другой программы: {found}")
            }
            Self::Unwritable(error) => write!(out, "состояние не записано: {error}"),
        }
    }
}

impl std::error::Error for StateError {}
