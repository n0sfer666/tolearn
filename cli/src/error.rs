use std::fmt;

use tolearn_generate::GenerateError;

#[derive(Debug)]
pub enum CliError {
    Usage(String),
    Package(String),
    Import(String),
    Export(String),
    Setup(String),
    Data(String),
    Generate(String),
}

impl CliError {
    pub fn code(&self) -> i32 {
        2
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage(what) => write!(out, "{what}\n\n{}", super::args::USAGE),
            Self::Package(reason) => write!(out, "пакет не собран: {reason}"),
            Self::Import(reason) => write!(out, "пакет не принят: {reason}"),
            Self::Export(reason) => write!(out, "экспорт не выполнен: {reason}"),
            Self::Setup(reason) => write!(out, "настройки генерации: {reason}"),
            Self::Data(reason) => write!(out, "{reason}"),
            Self::Generate(reason) => write!(out, "генерация отказала: {reason}"),
        }
    }
}

impl std::error::Error for CliError {}

impl From<GenerateError> for CliError {
    fn from(error: GenerateError) -> Self {
        Self::Generate(format!("{} — {error}", error.code()))
    }
}
