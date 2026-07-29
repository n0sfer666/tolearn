use std::fmt;

use tolearn_core::progress::DocumentError;
use tolearn_core::prompt::RenderError;
use tolearn_core::registry::RegistryError;
use tolearn_core::scan::ScanError;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct IpcError {
    pub code: String,
    pub message: String,
}

impl IpcError {
    pub fn new(code: &str, message: String) -> Self {
        Self {
            code: code.to_owned(),
            message,
        }
    }

    pub fn unknown_command(name: &str) -> Self {
        Self::new(
            "ipc.unknown-command",
            format!("команды `{name}` нет в контракте"),
        )
    }

    pub fn payload(error: &serde_json::Error) -> Self {
        Self::new("ipc.malformed-payload", error.to_string())
    }

    pub fn unknown_topic(topic: &str) -> Self {
        Self::new("topic.unknown", format!("темы `{topic}` в бандле нет"))
    }

    pub fn unknown_version(version: u32, roadmap: &str) -> Self {
        Self::new(
            "history.unknown-version",
            format!("версии {version} программы `{roadmap}` в истории нет"),
        )
    }

    pub fn malformed_date(value: &str) -> Self {
        Self::new("date.malformed", format!("`{value}` — не дата"))
    }

    pub fn unwritable(path: &std::path::Path, reason: &str) -> Self {
        Self::new(
            "file.unwritable",
            format!("`{}` не записывается: {reason}", path.display()),
        )
    }

    pub fn unreadable(path: &str, reason: &str) -> Self {
        Self::new("file.unreadable", format!("`{path}` не читается: {reason}"))
    }
}

impl fmt::Display for IpcError {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(out, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for IpcError {}

impl From<tolearn_core::archive::ArchiveError> for IpcError {
    fn from(error: tolearn_core::archive::ArchiveError) -> Self {
        Self::new(error.code(), error.to_string())
    }
}

impl From<ScanError> for IpcError {
    fn from(error: ScanError) -> Self {
        let code = match error {
            ScanError::NoRoadmap { .. } => "scan.no-roadmap",
            ScanError::Unreadable { .. } => "scan.unreadable",
            ScanError::Malformed { .. } => "scan.malformed",
        };
        Self::new(code, error.to_string())
    }
}

impl From<tolearn_core::verdict::VerdictError> for IpcError {
    fn from(error: tolearn_core::verdict::VerdictError) -> Self {
        use tolearn_core::verdict::VerdictError as Broken;
        let code = match error {
            Broken::NoJson => "verdict.no-json",
            Broken::Malformed { .. } => "verdict.malformed",
            Broken::Missing { .. } => "verdict.missing-field",
            Broken::WrongTopic { .. } => "verdict.wrong-topic",
            Broken::NoAnswers => "verdict.no-answers",
        };
        Self::new(code, error.to_string())
    }
}

impl From<DocumentError> for IpcError {
    fn from(error: DocumentError) -> Self {
        Self::new("progress.malformed", error.to_string())
    }
}

impl From<RenderError> for IpcError {
    fn from(error: RenderError) -> Self {
        let code = match error {
            RenderError::NoPrompt => "prompt.no-prompt",
            RenderError::Unknown { .. } => "prompt.unknown-placeholder",
            RenderError::Unclosed { .. } => "prompt.unclosed-placeholder",
        };
        Self::new(code, error.to_string())
    }
}

impl From<RegistryError> for IpcError {
    fn from(error: RegistryError) -> Self {
        let code = match error {
            RegistryError::Unreadable(_) => "registry.unreadable",
            RegistryError::Malformed(_) => "registry.malformed",
            RegistryError::Unwritable(_) => "registry.unwritable",
        };
        Self::new(code, error.to_string())
    }
}
