#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseFailure {
    Syntax,
    DocumentCount,
    DuplicateKey,
    MissingField,
    WrongType,
    Empty,
    BadDate,
    OutOfRange,
    UnknownValue,
}

impl ParseFailure {
    pub fn code(self) -> &'static str {
        match self {
            Self::Syntax => "yaml.syntax",
            Self::DocumentCount => "yaml.document-count",
            Self::DuplicateKey => "yaml.duplicate-key",
            Self::Empty => "yaml.empty",
            Self::BadDate => "yaml.bad-date",
            Self::MissingField => "yaml.missing-field",
            Self::WrongType => "yaml.wrong-type",
            Self::OutOfRange => "yaml.out-of-range",
            Self::UnknownValue => "yaml.unknown-value",
        }
    }
}
