use std::fmt;
use std::path::PathBuf;

#[derive(Debug)]
pub enum SpeechError {
    Off,
    NoModel,
    Unreadable { path: PathBuf, reason: String },
    Unsupported(String),
    Rejected { path: PathBuf, reason: String },
    Deaf(String),
    Silent,
    Failed(String),
}

impl fmt::Display for SpeechError {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Off => write!(
                out,
                "распознавание речи не собрано в этом варианте приложения"
            ),
            Self::NoModel => write!(out, "модель распознавания не найдена"),
            Self::Unreadable { path, reason } => {
                write!(out, "{} не прочитан: {reason}", path.display())
            }
            Self::Unsupported(reason) => write!(out, "запись не в том формате: {reason}"),
            Self::Rejected { path, reason } => {
                write!(out, "модель {} не открылась: {reason}", path.display())
            }
            Self::Deaf(reason) => write!(out, "микрофон не открылся: {reason}"),
            Self::Silent => write!(out, "записи нет: микрофон не дал ни одного отсчёта"),
            Self::Failed(reason) => write!(out, "распознавание не удалось: {reason}"),
        }
    }
}

impl std::error::Error for SpeechError {}
