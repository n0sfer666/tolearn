use crate::date::Date;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    pub subject: String,
    pub level: Level,
    pub weekly_hours: u32,
    pub weeks: Option<u32>,
    pub locale: String,
    pub today: Date,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Beginner,
    Basics,
    Confident,
}

impl Level {
    pub fn label(self) -> &'static str {
        match self {
            Self::Beginner => "beginner",
            Self::Basics => "basics",
            Self::Confident => "confident",
        }
    }

    pub fn told(self) -> &'static str {
        match self {
            Self::Beginner => "с темой не сталкивался",
            Self::Basics => "знает основы, но почти не применял",
            Self::Confident => "применял уверенно, нужны пробелы и глубина",
        }
    }

    pub fn read(label: &str) -> Option<Self> {
        [Self::Beginner, Self::Basics, Self::Confident]
            .into_iter()
            .find(|level| level.label() == label)
    }
}
