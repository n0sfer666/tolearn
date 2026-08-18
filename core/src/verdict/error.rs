use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerdictError {
    NoJson,
    Malformed { reason: String },
    Missing { field: String },
    WrongTopic { expected: String, found: String },
    NoAnswers,
}

impl fmt::Display for VerdictError {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoJson => write!(out, "в ответе нет блока JSON с вердиктом"),
            Self::Malformed { reason } => write!(out, "блок вердикта не разобран: {reason}"),
            Self::Missing { field } => write!(out, "в вердикте нет поля `{field}`"),
            Self::WrongTopic { expected, found } => {
                write!(out, "вердикт о теме `{found}`, а экзамен по `{expected}`")
            }
            Self::NoAnswers => write!(out, "в вердикте нет разбора вопросов `per_question`"),
        }
    }
}

impl std::error::Error for VerdictError {}
