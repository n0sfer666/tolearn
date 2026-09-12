use std::fmt;

use crate::error::GenerateError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NextError {
    Node(String),
    Stage(String),
    Ungenerated(String),
    End(String),
    Taken(String),
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
            Self::Taken(_) => "next.taken",
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
                "этап «{stage}» последний в карте: переход к следующей подпрограмме появится позже"
            ),
            Self::Taken(stage) => write!(out, "следующий этап «{stage}» уже создан"),
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
