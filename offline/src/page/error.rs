use std::fmt;

#[derive(Debug)]
pub enum PageError {
    Unreachable(String, String),
    Unreadable(String),
    Prerender(String),
}

impl fmt::Display for PageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreachable(url, reason) => write!(f, "страница {url} не загрузилась: {reason}"),
            Self::Unreadable(reason) => write!(f, "страница не разобралась: {reason}"),
            Self::Prerender(reason) => write!(f, "пререндер не отработал: {reason}"),
        }
    }
}

impl std::error::Error for PageError {}
