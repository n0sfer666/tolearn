use std::fmt;

#[derive(Debug)]
pub enum LinkError {
    Scheme { given: String },
    Action { given: String },
    Missing { field: String },
    Extra { field: String },
    Value { field: String, given: String },
}

impl LinkError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Scheme { .. } => "link.scheme",
            Self::Action { .. } => "link.action",
            Self::Missing { .. } => "link.missing",
            Self::Extra { .. } => "link.extra",
            Self::Value { .. } => "link.value",
        }
    }
}

impl fmt::Display for LinkError {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Scheme { given } => write!(out, "ссылка не начинается с `tolearn://`: `{given}`"),
            Self::Action { given } => write!(out, "неизвестное действие ссылки: `{given}`"),
            Self::Missing { field } => write!(out, "в ссылке нет `{field}`"),
            Self::Extra { field } => write!(out, "лишний параметр ссылки: `{field}`"),
            Self::Value { field, given } => {
                write!(out, "недопустимое значение `{field}`: `{given}`")
            }
        }
    }
}

impl std::error::Error for LinkError {}
