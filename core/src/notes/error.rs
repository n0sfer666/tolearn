use std::fmt;

#[derive(Debug)]
pub enum NoteError {
    Unreadable { path: String, reason: String },
    Unwritable { path: String, reason: String },
    Conflict { theirs: String, ours: String },
}

impl fmt::Display for NoteError {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreadable { path, reason } => write!(out, "`{path}` не читается: {reason}"),
            Self::Unwritable { path, reason } => write!(out, "`{path}` не записывается: {reason}"),
            Self::Conflict { .. } => write!(
                out,
                "конспект изменён и снаружи, и в приложении: выбор за человеком"
            ),
        }
    }
}

impl std::error::Error for NoteError {}
