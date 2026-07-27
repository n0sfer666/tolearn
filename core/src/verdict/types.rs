use crate::progress::{Answer, NextAction, Verdict as Grade};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verdict {
    pub topic_id: String,
    pub result: Grade,
    pub date: Option<String>,
    pub model: Option<String>,
    pub hinted: bool,
    pub practice_accepted: bool,
    pub failed_checks: Vec<String>,
    pub per_question: Vec<Answer>,
    pub gaps: Vec<String>,
    pub calibration: Option<String>,
    pub notes: Vec<String>,
    pub next_action: Option<NextAction>,
    pub retry_after_days: Option<u32>,
    pub raw: String,
    pub missing: Vec<String>,
    pub unknown_questions: Vec<String>,
}
