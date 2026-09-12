use std::fmt;

use tolearn_provider::CheckError;

use crate::REPAIRS;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenerateError {
    Offline {
        host: String,
        reason: String,
    },
    Provider(CheckError),
    Cache(String),
    Unrepaired {
        what: &'static str,
        flaws: Vec<String>,
    },
}

impl GenerateError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Offline { .. } => "generate.offline",
            Self::Provider(error) => error.code(),
            Self::Cache(_) => "generate.cache",
            Self::Unrepaired { .. } => "generate.unrepaired",
        }
    }
}

impl fmt::Display for GenerateError {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Offline { host, reason } => write!(
                out,
                "нет сети: {host} не ответил ({reason}), а без сети источники не проверить"
            ),
            Self::Provider(error) => write!(out, "{error}"),
            Self::Cache(reason) => write!(out, "кэш источников недоступен: {reason}"),
            Self::Unrepaired { what, flaws } => write!(
                out,
                "модель не исправила {what} за {REPAIRS} круга починки: {}",
                flaws.join("; ")
            ),
        }
    }
}

impl std::error::Error for GenerateError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Offline { .. } | Self::Cache(_) | Self::Unrepaired { .. } => None,
            Self::Provider(error) => Some(error),
        }
    }
}
