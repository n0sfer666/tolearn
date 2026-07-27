use std::fmt;
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub enum RunError {
    NoDirectory(PathBuf),
    NotStarted(io::Error),
    Broken(io::Error),
}

impl fmt::Display for RunError {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoDirectory(path) => {
                write!(out, "рабочей директории `{}` нет", path.display())
            }
            Self::NotStarted(error) => write!(out, "команда не запущена: {error}"),
            Self::Broken(error) => write!(out, "команда прервалась не своей волей: {error}"),
        }
    }
}

impl std::error::Error for RunError {}
