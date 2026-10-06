use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerdictError {
    Absent,
    Stage {
        expected: String,
        found: String,
    },
    Shape(String),
    Questions {
        missing: Vec<String>,
        stray: Vec<String>,
        repeated: Vec<String>,
    },
}

impl fmt::Display for VerdictError {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Absent => write!(out, "в тексте нет JSON-блока с вердиктом"),
            Self::Stage { expected, found } => write!(
                out,
                "вердикт этапа `{found}` вставлен не в тот этап: открыт `{expected}`"
            ),
            Self::Shape(reason) => write!(out, "вердикт не разобран: {reason}"),
            Self::Questions {
                missing,
                stray,
                repeated,
            } => {
                let parts = [
                    ("нет оценки", missing),
                    ("не из этого этапа", stray),
                    ("оценены дважды", repeated),
                ]
                .into_iter()
                .filter(|(_, ids)| !ids.is_empty())
                .map(|(what, ids)| format!("{what}: {}", ids.join(", ")))
                .collect::<Vec<_>>();
                write!(
                    out,
                    "вопросы вердикта не совпадают с вопросами этапа — {}",
                    parts.join("; ")
                )
            }
        }
    }
}

impl std::error::Error for VerdictError {}
