use super::enums::{NextAction, Outcome, Source, Status, Verdict};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Progress {
    pub schema: String,
    pub roadmap_id: String,
    pub topics: Vec<(String, TopicState)>,
}

impl Progress {
    pub fn state(&self, topic: &str) -> Option<&TopicState> {
        self.topics
            .iter()
            .find(|(id, _)| id == topic)
            .map(|(_, state)| state)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TopicState {
    pub status: Status,
    pub attempts: Vec<Attempt>,
    pub passed_at: Option<String>,
    pub next_review_at: Option<String>,
    pub gaps: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attempt {
    pub at: String,
    pub source: Source,
    pub verdict: Verdict,
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Answer {
    pub id: String,
    pub outcome: Outcome,
    pub quote: Option<String>,
    pub missed: Vec<String>,
    pub signal_extension: bool,
}
