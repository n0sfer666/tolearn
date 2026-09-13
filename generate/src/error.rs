use std::fmt;

use tolearn_core::Hours;
use tolearn_core::library::LibraryError;
use tolearn_core::verdict::VerdictError;
use tolearn_provider::CheckError;

use crate::REPAIRS;
use crate::fork::NextError;
use crate::plan::{STAGE_MAX_HOURS, STAGE_MIN_HOURS};
use crate::regenerate::RegenerateError;

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
    Unfit {
        flaws: Vec<String>,
    },
    Cancelled,
    Unwritten(String),
    Library(LibraryError),
    Next(NextError),
    Regenerate(RegenerateError),
    Verdict(VerdictError),
    Unclear,
}

impl GenerateError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Offline { .. } => "generate.offline",
            Self::Provider(error) => error.code(),
            Self::Cache(_) => "generate.cache",
            Self::Unrepaired { .. } => "generate.unrepaired",
            Self::StageHours { .. } => "generate.stage-hours",
            Self::Unfit { .. } => "generate.unfit",
            Self::Cancelled => "generate.cancelled",
            Self::Unwritten(_) => "generate.unwritten",
            Self::Library(error) => error.code(),
            Self::Next(error) => error.code(),
            Self::Regenerate(error) => error.code(),
            Self::Verdict(_) => "generate.verdict",
            Self::Unclear => "generate.unclear",
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
            Self::Cache(reason) => write!(out, "кэш генерации недоступен: {reason}"),
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
            Self::Unfit { flaws } => {
                write!(out, "карта не годится для генерации: {}", flaws.join("; "))
            }
            Self::Cancelled => write!(out, "генерация отменена, библиотека не изменилась"),
            Self::Unwritten(reason) => write!(out, "не удалось записать программу: {reason}"),
            Self::Library(error) => write!(out, "{error}"),
            Self::Next(error) => write!(out, "{error}"),
            Self::Regenerate(error) => write!(out, "{error}"),
            Self::Verdict(error) => write!(
                out,
                "модель не прислала годный вердикт и после починки: {error}"
            ),
            Self::Unclear => write!(out, "модель прислала пустое объяснение"),
        }
    }
}

impl std::error::Error for GenerateError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Offline { .. }
            | Self::Cache(_)
            | Self::Unrepaired { .. }
            | Self::StageHours { .. }
            | Self::Unfit { .. }
            | Self::Cancelled
            | Self::Unwritten(_)
            | Self::Unclear => None,
            Self::Provider(error) => Some(error),
            Self::Library(error) => Some(error),
            Self::Next(error) => Some(error),
            Self::Regenerate(error) => Some(error),
            Self::Verdict(error) => Some(error),
        }
    }
}
