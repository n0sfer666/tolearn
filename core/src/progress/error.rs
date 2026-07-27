use std::fmt;

use crate::yaml::ParseError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocumentError {
    Parse(ParseError),
    UnknownTopic(String),
    Malformed(String),
}

impl fmt::Display for DocumentError {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(error) => write!(out, "{error}"),
            Self::UnknownTopic(topic) => {
                write!(out, "the progress file carries no topic `{topic}`")
            }
            Self::Malformed(what) => write!(out, "the progress file is malformed: {what}"),
        }
    }
}

impl From<ParseError> for DocumentError {
    fn from(error: ParseError) -> Self {
        Self::Parse(error)
    }
}
