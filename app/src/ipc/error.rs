use std::fmt;

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

impl From<tolearn_core::library::LibraryError> for IpcError {
    fn from(error: tolearn_core::library::LibraryError) -> Self {
        Self::new(error.code(), error.to_string())
    }
}

impl From<tolearn_core::state::StateError> for IpcError {
    fn from(error: tolearn_core::state::StateError) -> Self {
        use tolearn_core::state::StateError as Broken;
        let code = match &error {
            Broken::Stray(_) => "state.stray",
            Broken::Unreadable(_) => "state.unreadable",
            Broken::Malformed(_) => "state.malformed",
            Broken::Foreign(_) => "state.foreign",
            Broken::Unwritable(_) => "state.unwritable",
        };
        Self::new(code, error.to_string())
    }
}

impl From<tolearn_core::export::ExportError> for IpcError {
    fn from(error: tolearn_core::export::ExportError) -> Self {
        Self::new(error.code(), error.to_string())
    }
}

impl From<tolearn_core::package::UnpackError> for IpcError {
    fn from(error: tolearn_core::package::UnpackError) -> Self {
        Self::new(error.code(), error.to_string())
    }
}

impl From<tolearn_speech::SpeechError> for IpcError {
    fn from(error: tolearn_speech::SpeechError) -> Self {
        use tolearn_speech::SpeechError as Mute;
        let code = match &error {
            Mute::Off => "speech.off",
            Mute::NoModel => "speech.no-model",
            Mute::Unreadable { .. } => "speech.unreadable",
            Mute::Unsupported(_) => "speech.unsupported",
            Mute::Rejected { .. } => "speech.rejected",
            Mute::Deaf(_) => "speech.deaf",
            Mute::Silent => "speech.silent",
            Mute::Failed(_) => "speech.failed",
        };
        Self::new(code, error.to_string())
    }
}
