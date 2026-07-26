use super::enums::{
    Confidence, Liveness, MaterialTier, MaterialType, PracticeKind, PracticeTier, QuestionType,
    Retention, Volatility,
};
use crate::Hours;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Topic {
    pub schema: String,
    pub id: String,
    pub title: String,
    pub stage: u32,
    pub depends_on: Vec<String>,
    pub est_hours: Hours,
    pub volatility: Volatility,
    pub revalidate_after_days: u32,
    pub verified_at: String,
    pub confidence: Confidence,
    pub retention: Retention,
    pub version_context: Vec<String>,
    pub outcomes: Vec<String>,
    pub misconceptions: Vec<String>,
    pub materials: Vec<Material>,
    pub practice: Practice,
    pub questions: Vec<Question>,
    pub exam: Exam,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Material {
    pub title: String,
    pub url: String,
    pub kind: MaterialType,
    pub tier: MaterialTier,
    pub lang: String,
    pub liveness: Liveness,
    pub published: Option<String>,
    pub covers_version: Option<String>,
    pub checked_at: String,
    pub stale: bool,
    pub delta: Option<String>,
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Practice {
    pub kind: PracticeKind,
    pub tier: PracticeTier,
    pub task: String,
    pub deliverable: String,
    pub starting_point: Option<String>,
    pub fallback: Option<String>,
    pub time_box_min: u32,
    pub smoke_checked: bool,
    pub constraints: Vec<Check>,
    pub acceptance: Vec<Check>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Check {
    pub id: String,
    pub claim: String,
    pub check: String,
    pub expect: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Question {
    pub id: String,
    pub kind: QuestionType,
    pub text: String,
    pub expected_signals: Vec<String>,
    pub red_flags: Vec<String>,
    pub follow_up: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Exam {
    pub focus: String,
    pub traps: Vec<String>,
    pub artifact_required: bool,
    pub max_exchanges: u32,
}
