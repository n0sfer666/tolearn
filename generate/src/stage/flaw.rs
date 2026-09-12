use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Flaw {
    Unreadable(String),
    UnknownImage { block: String, image: String },
    UnknownSource { block: String, source: String },
    ForeignLink { block: String, url: String },
}

impl fmt::Display for Flaw {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreadable(reason) => write!(out, "ответ не прочитан: {reason}"),
            Self::UnknownImage { block, image } => write!(
                out,
                "блок {block}: картинки «{image}» нет среди проверенных"
            ),
            Self::UnknownSource { block, source } => write!(
                out,
                "блок {block}: источника «{source}» нет среди проверенных"
            ),
            Self::ForeignLink { block, url } => {
                write!(
                    out,
                    "блок {block}: ссылка {url} ведёт не на проверенную страницу"
                )
            }
        }
    }
}
