use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderError {
    NoPrompt,
    Unknown { name: String },
    Unclosed { line: String },
}

impl fmt::Display for RenderError {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoPrompt => write!(
                out,
                "the examiner template has no prompt: two `---` rules are expected around it"
            ),
            Self::Unknown { name } => write!(out, "`{{{{{name}}}}}` is nothing the bundle holds"),
            Self::Unclosed { line } => write!(out, "a placeholder is never closed in `{line}`"),
        }
    }
}

impl std::error::Error for RenderError {}
