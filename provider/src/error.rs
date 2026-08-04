use std::fmt;
use std::io;

use tolearn_core::yaml::ParseError;

#[derive(Debug)]
pub enum ProviderError {
    Unreadable(io::Error),
    Malformed(ParseError),
    Unwritable(io::Error),
}

impl fmt::Display for ProviderError {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreadable(error) => write!(out, "провайдер не прочитан: {error}"),
            Self::Malformed(error) => write!(out, "провайдер не разобран: {error}"),
            Self::Unwritable(error) => write!(out, "провайдер не записан: {error}"),
        }
    }
}

impl std::error::Error for ProviderError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VaultError {
    reason: String,
}

impl VaultError {
    pub(crate) fn new(reason: impl fmt::Display) -> Self {
        Self {
            reason: reason.to_string(),
        }
    }
}

impl fmt::Display for VaultError {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(out, "системное хранилище не ответило: {}", self.reason)
    }
}

impl std::error::Error for VaultError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckError {
    Disabled,
    NoKey,
    NoModel,
    Unreachable(String),
    Rejected,
    Answered(u16),
    BadAnswer,
    ModelMissing(String),
    NotFound(String),
    Failed { code: Option<i32>, said: String },
    TimedOut(u32),
    Truncated,
}

impl CheckError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Disabled => "provider.disabled",
            Self::NoKey => "provider.no-key",
            Self::NoModel => "provider.no-model",
            Self::Unreachable(_) => "provider.unreachable",
            Self::Rejected => "provider.rejected",
            Self::Answered(_) => "provider.answered",
            Self::BadAnswer => "provider.bad-answer",
            Self::ModelMissing(_) => "provider.model-missing",
            Self::NotFound(_) => "harness.not-found",
            Self::Failed { .. } => "harness.failed",
            Self::TimedOut(_) => "harness.timeout",
            Self::Truncated => "harness.truncated",
        }
    }
}

impl fmt::Display for CheckError {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Disabled => write!(out, "провайдер выключен"),
            Self::NoKey => write!(out, "ключ не задан"),
            Self::NoModel => write!(out, "модель не выбрана"),
            Self::Unreachable(reason) => write!(out, "провайдер не отвечает: {reason}"),
            Self::Rejected => write!(out, "провайдер отверг ключ"),
            Self::Answered(status) => write!(out, "провайдер ответил кодом {status}"),
            Self::BadAnswer => write!(out, "провайдер ответил не в том формате, которого ждали"),
            Self::ModelMissing(model) => {
                write!(out, "модели `{model}` на сервере нет: её надо поставить")
            }
            Self::NotFound(command) => write!(
                out,
                "команда `{command}` не найдена: выполните `which {command}` и впишите полный путь"
            ),
            Self::Failed { code, said } => match code {
                Some(code) => write!(out, "харнесс завершился с кодом {code}: {said}"),
                None => write!(out, "харнесс убит сигналом: {said}"),
            },
            Self::TimedOut(seconds) => write!(out, "харнесс не ответил за {seconds} с"),
            Self::Truncated => write!(out, "харнесс напечатал больше, чем разрешено принять"),
        }
    }
}

impl std::error::Error for CheckError {}
