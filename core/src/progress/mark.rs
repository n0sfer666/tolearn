use super::enums::{Source, Status, Verdict};
use super::types::Attempt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mark {
    pub status: Status,
    pub at: String,
    pub passed_at: Option<String>,
    pub next_review_at: Option<String>,
    pub note: Option<String>,
}

pub(super) fn attempt(mark: &Mark) -> Attempt {
    Attempt {
        at: mark.at.clone(),
        source: Source::Manual,
        verdict: verdict(mark.status),
        model: None,
        hinted: false,
        practice_accepted: false,
        failed_checks: Vec::new(),
        per_question: Vec::new(),
        gaps: Vec::new(),
        calibration: None,
        notes: mark.note.clone().into_iter().collect(),
        next_action: None,
        retry_after_days: None,
        raw: String::new(),
    }
}

fn verdict(status: Status) -> Verdict {
    match status {
        Status::Passed | Status::PassedOut | Status::StalePassed => Verdict::Pass,
        Status::Failed => Verdict::Fail,
        Status::Blocked => Verdict::Blocked,
        Status::Todo | Status::InProgress | Status::ExamPending => Verdict::Partial,
    }
}
