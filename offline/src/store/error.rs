use std::fmt;
use std::io;

#[derive(Debug)]
pub enum StoreError {
    Unreadable(io::Error),
    Unwritable(io::Error),
    Absent(String),
    Index(rusqlite::Error),
}

impl fmt::Display for StoreError {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreadable(error) => write!(out, "объект кэша не прочитан: {error}"),
            Self::Unwritable(error) => write!(out, "объект кэша не записан: {error}"),
            Self::Absent(url) => write!(out, "адреса нет в индексе кэша: {url}"),
            Self::Index(error) => write!(out, "индекс кэша не отвечает: {error}"),
        }
    }
}

impl std::error::Error for StoreError {}

impl From<rusqlite::Error> for StoreError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Index(error)
    }
}
