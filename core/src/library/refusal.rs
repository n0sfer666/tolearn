use std::fmt;

use crate::program::{LoadError, Violation};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    Unloadable(LoadError),
    Invalid(Vec<Violation>),
    Misplaced { directory: String, uuid: String },
}

impl Refusal {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Unloadable(error) => error.code(),
            Self::Invalid(_) => "library.invalid",
            Self::Misplaced { .. } => "library.misplaced",
        }
    }
}

impl fmt::Display for Refusal {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unloadable(error) => write!(out, "{error}"),
            Self::Invalid(violations) => {
                write!(out, "the program breaks {} rules", violations.len())?;
                for violation in violations {
                    write!(out, "; {violation}")?;
                }
                Ok(())
            }
            Self::Misplaced { directory, uuid } => write!(
                out,
                "`{directory}` holds the program `{uuid}` instead of its namesake"
            ),
        }
    }
}

impl std::error::Error for Refusal {}
