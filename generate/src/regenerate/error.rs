use std::fmt;

use crate::error::GenerateError;
use crate::located::Missing;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegenerateError {
    Node(String),
    Stage(String),
    Ungenerated(String),
}

impl RegenerateError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Node(_) => "regenerate.node",
            Self::Stage(_) => "regenerate.stage",
            Self::Ungenerated(_) => "regenerate.ungenerated",
        }
    }
}

impl fmt::Display for RegenerateError {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Node(node) => write!(out, "в программе нет подпрограммы {node}"),
            Self::Stage(stage) => write!(out, "в карте нет этапа «{stage}»"),
            Self::Ungenerated(stage) => {
                write!(out, "этап «{stage}» ещё не создан: перегенерировать нечего")
            }
        }
    }
}

impl std::error::Error for RegenerateError {}

impl From<RegenerateError> for GenerateError {
    fn from(error: RegenerateError) -> Self {
        Self::Regenerate(error)
    }
}

impl From<Missing> for RegenerateError {
    fn from(missing: Missing) -> Self {
        match missing {
            Missing::Node(node) => Self::Node(node),
            Missing::Stage(stage) => Self::Stage(stage),
            Missing::Ungenerated(stage) => Self::Ungenerated(stage),
        }
    }
}
