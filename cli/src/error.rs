use std::fmt;

#[derive(Debug)]
pub enum CliError {
    Usage(String),
    Unreadable { path: String, reason: String },
    Bundle(String),
    UnknownTopic(String),
    Verdict(String),
    Write(String),
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
            Self::Unreadable { path, reason } => write!(out, "не читается `{path}`: {reason}"),
            Self::Bundle(reason) => write!(out, "бандл не читается: {reason}"),
            Self::UnknownTopic(id) => write!(out, "роадмап не знает темы `{id}`"),
            Self::Verdict(reason) => write!(out, "вердикт не принят: {reason}"),
            Self::Write(reason) => write!(out, "не удалось записать прогресс: {reason}"),
        }
    }
}

impl std::error::Error for CliError {}
