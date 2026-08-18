use std::fmt;

#[derive(Debug)]
pub enum SealError {
    Unreadable { path: String, reason: String },
    Unwritable { path: String, reason: String },
    Locked { path: String },
    WrongKey,
    MalformedKey,
    Busy { path: String },
}

impl fmt::Display for SealError {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreadable { path, reason } => write!(out, "`{path}` не читается: {reason}"),
            Self::Unwritable { path, reason } => write!(out, "`{path}` не записывается: {reason}"),
            Self::Locked { path } => write!(out, "`{path}` зашифрован: нужен ключ"),
            Self::WrongKey => write!(
                out,
                "ключ не подходит: ни ключ устройства, ни парольная фраза не открывают хранилище"
            ),
            Self::MalformedKey => write!(out, "ключ устройства испорчен и не разбирается"),
            Self::Busy { path } => write!(
                out,
                "`{path}` занят: прошлое переключение не доведено до конца"
            ),
        }
    }
}

impl std::error::Error for SealError {}
