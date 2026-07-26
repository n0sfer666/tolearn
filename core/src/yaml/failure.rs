#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseFailure {
    Syntax,
    DocumentCount,
    MissingField,
    WrongType,
    OutOfRange,
    UnknownValue,
}

impl ParseFailure {
    pub fn code(self) -> &'static str {
        match self {
            Self::Syntax => "yaml.syntax",
            Self::DocumentCount => "yaml.document-count",
            Self::MissingField => "yaml.missing-field",
            Self::WrongType => "yaml.wrong-type",
            Self::OutOfRange => "yaml.out-of-range",
            Self::UnknownValue => "yaml.unknown-value",
        }
    }
}
