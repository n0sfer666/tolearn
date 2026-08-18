use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepError {
    NoJson,
    Malformed { reason: String },
    WrongQuestion { expected: String, found: String },
}

impl StepError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::NoJson => "exam.no-json",
            Self::Malformed { .. } => "exam.malformed",
            Self::WrongQuestion { .. } => "exam.wrong-question",
        }
    }
}

impl fmt::Display for StepError {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoJson => write!(out, "в ответе нет блока JSON с оценкой"),
            Self::Malformed { reason } => write!(out, "блок оценки не разобран: {reason}"),
            Self::WrongQuestion { expected, found } => {
                write!(out, "оценка о вопросе `{found}`, а спрашивали `{expected}`")
            }
        }
    }
}

impl std::error::Error for StepError {}
