#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Report {
    pub kept: Vec<String>,
    pub stale: Vec<Stale>,
    pub added: Vec<String>,
    pub orphaned: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stale {
    pub id: String,
    pub changed: Vec<Part>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Part {
    DependsOn,
    Outcomes,
    Misconceptions,
    Practice,
    Questions,
    Exam,
}

impl Part {
    pub fn label(self) -> &'static str {
        match self {
            Self::DependsOn => "depends_on",
            Self::Outcomes => "outcomes",
            Self::Misconceptions => "misconceptions",
            Self::Practice => "practice",
            Self::Questions => "questions",
            Self::Exam => "exam",
        }
    }
}
