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
        }
    }
}

impl std::error::Error for CheckError {}
