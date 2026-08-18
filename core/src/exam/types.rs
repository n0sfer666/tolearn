use crate::progress::Answer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Examiner,
    Student,
}

impl Side {
    pub fn label(self) -> &'static str {
        match self {
            Self::Examiner => "examiner",
            Self::Student => "student",
        }
    }

    pub fn read(label: &str) -> Option<Self> {
        [Self::Examiner, Self::Student]
            .into_iter()
            .find(|side| side.label() == label)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    pub side: Side,
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Next {
    FollowUp,
    Close,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Artifact {
    None,
    Shown,
    Passed,
    Skipped,
    Aside,
}

impl Artifact {
    pub fn label(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Shown => "shown",
            Self::Passed => "passed",
            Self::Skipped => "skipped",
            Self::Aside => "aside",
        }
    }

    pub fn read(label: &str) -> Option<Self> {
        [
            Self::None,
            Self::Shown,
            Self::Passed,
            Self::Skipped,
            Self::Aside,
        ]
        .into_iter()
        .find(|artifact| artifact.label() == label)
    }

    pub fn told(self) -> &'static str {
        match self {
            Self::None => "артефакт не предъявлен вовсе",
            Self::Shown => "артефакт предъявлен, но приёмка не сошлась",
            Self::Passed => "артефакт предъявлен и принят",
            Self::Skipped => "практика артефакта не требует",
            Self::Aside => {
                "практика в этом прогоне не проверялась и на вердикт не влияет: \
                 считай её в порядке, `blocked` за неё не ставь"
            }
        }
    }
}

pub const NEXT: [(&str, Next); 2] = [("follow_up", Next::FollowUp), ("close", Next::Close)];
pub const ARTIFACT: [(&str, Artifact); 3] = [
    ("none", Artifact::None),
    ("shown", Artifact::Shown),
    ("passed", Artifact::Passed),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    pub answer: Answer,
    pub next: Next,
    pub say: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shown {
    pub artifact: Artifact,
    pub failed_checks: Vec<String>,
    pub next: Next,
    pub say: String,
}
