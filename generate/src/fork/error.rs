use std::fmt;

use crate::error::GenerateError;
use crate::located::Missing;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NextError {
    Node(String),
    Stage(String),
    Ungenerated(String),
    End(String),
    Taken(String),
    Begun(String),
    Unforked(String),
    Choice { choice: usize, count: usize },
}

impl NextError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Node(_) => "next.node",
            Self::Stage(_) => "next.stage",
            Self::Ungenerated(_) => "next.ungenerated",
            Self::End(_) => "next.end",
            Self::Taken(_) | Self::Begun(_) => "next.taken",
            Self::Unforked(_) => "next.unforked",
            Self::Choice { .. } => "next.choice",
        }
    }
}

impl fmt::Display for NextError {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Node(node) => write!(out, "в программе нет подпрограммы {node}"),
            Self::Stage(stage) => write!(out, "в карте нет этапа «{stage}»"),
            Self::Ungenerated(stage) => write!(
                out,
                "этап «{stage}» ещё не создан: развилка открывается после него"
            ),
            Self::End(stage) => write!(
                out,
                "этап «{stage}» последний в программе: карта пройдена до конца"
            ),
            Self::Taken(stage) => write!(out, "следующий этап «{stage}» уже создан"),
            Self::Begun(title) => write!(out, "следующая подпрограмма «{title}» уже начата"),
            Self::Unforked(stage) => write!(
                out,
                "развилка после этапа «{stage}» не открыта: сначала открой её"
            ),
            Self::Choice { choice, count } => write!(
                out,
                "варианта №{} нет: в развилке их {count}",
                choice.saturating_add(1)
            ),
        }
    }
}

impl std::error::Error for NextError {}

impl From<NextError> for GenerateError {
    fn from(error: NextError) -> Self {
        Self::Next(error)
    }
}

impl From<Missing> for NextError {
    fn from(missing: Missing) -> Self {
        match missing {
            Missing::Node(node) => Self::Node(node),
            Missing::Stage(stage) => Self::Stage(stage),
            Missing::Ungenerated(stage) => Self::Ungenerated(stage),
        }
    }
}
