use std::fmt;

#[derive(Debug)]
pub enum VideoError {
    NotStarted(std::io::Error),
    Failed(String),
    Cancelled,
}

impl fmt::Display for VideoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotStarted(reason) => write!(f, "yt-dlp не запустился: {reason}"),
            Self::Failed(reason) => write!(f, "yt-dlp не справился: {reason}"),
            Self::Cancelled => write!(f, "загрузка отменена"),
        }
    }
}

impl std::error::Error for VideoError {}
