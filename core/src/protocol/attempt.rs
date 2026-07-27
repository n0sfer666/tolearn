use crate::progress::{Attempt, Source};
use crate::verdict::Verdict;

pub fn of(verdict: &Verdict, at: &str) -> Attempt {
    Attempt {
        at: at.to_owned(),
        source: Source::Exam,
        verdict: verdict.result,
        model: verdict.model.clone(),
        hinted: verdict.hinted,
        practice_accepted: verdict.practice_accepted,
        failed_checks: verdict.failed_checks.clone(),
        per_question: verdict.per_question.clone(),
        gaps: verdict.gaps.clone(),
        calibration: verdict.calibration.clone(),
        notes: verdict.notes.clone(),
        next_action: verdict.next_action,
        retry_after_days: verdict.retry_after_days,
        raw: verdict.raw.clone(),
    }
}
