use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Model,
    Page,
    Book,
    Image,
}

impl Kind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Model => "модель",
            Self::Page => "страница",
            Self::Book => "книга",
            Self::Image => "картинка",
        }
    }
}
