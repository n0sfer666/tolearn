use std::fmt;

use tolearn_core::Hours;
use tolearn_provider::CheckError;

use crate::REPAIRS;
use crate::plan::{STAGE_MAX_HOURS, STAGE_MIN_HOURS};

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
    StageHours {
        stage: String,
        hours: Hours,
    },
}

impl GenerateError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Offline { .. } => "generate.offline",
            Self::Provider(error) => error.code(),
            Self::Cache(_) => "generate.cache",
            Self::Unrepaired { .. } => "generate.unrepaired",
            Self::StageHours { .. } => "generate.stage-hours",
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
            Self::StageHours { stage, hours } => write!(
                out,
                "этап «{stage}» на {}–{} ч по карте, а этап занимает от {STAGE_MIN_HOURS} до {STAGE_MAX_HOURS} ч: сначала поправь карту",
                hours.min, hours.max
            ),
        }
    }
}

impl std::error::Error for GenerateError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Offline { .. }
            | Self::Cache(_)
            | Self::Unrepaired { .. }
            | Self::StageHours { .. } => None,
            Self::Provider(error) => Some(error),
        }
    }
}
