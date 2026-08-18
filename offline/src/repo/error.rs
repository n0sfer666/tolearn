use std::fmt;

#[derive(Debug)]
pub enum RepoError {
    Clone(String),
    TooBig { limit: u64, taken: u64 },
}

impl fmt::Display for RepoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Clone(reason) => write!(f, "репозиторий не склонировался: {reason}"),
            Self::TooBig { limit, taken } => write!(
                f,
                "репозиторий больше отведённого объёма: принято {taken} байт при потолке {limit}"
            ),
        }
    }
}

impl std::error::Error for RepoError {}
