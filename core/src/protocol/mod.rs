mod attempt;
mod record;
mod streak;
mod types;

pub use record::record;
pub use streak::{ENOUGH_FAILURES, counted, failing_streak};
pub use types::Applied;

use crate::date::Date;
use crate::progress::{Outcome, TopicState, Verdict as Grade};
use crate::status::{from_verdict, next_review_at};
use crate::topic::Topic;
use crate::verdict::Verdict;

pub fn apply(
    verdict: &Verdict,
    topic: &Topic,
    state: Option<&TopicState>,
    at: &str,
    today: Date,
) -> Applied {
    let passed = verdict.result == Grade::Pass;
    let day = verdict
        .date
        .as_deref()
        .and_then(Date::parse)
        .unwrap_or(today);
    let mut attempts = state
        .map(|state| state.attempts.clone())
        .unwrap_or_default();
    attempts.push(attempt::of(verdict, at));

    let state = TopicState {
        status: from_verdict(verdict.result),
        passed_at: if passed {
            Some(day.to_string())
        } else {
            state.and_then(|state| state.passed_at.clone())
        },
        next_review_at: if passed {
            next_review_at(topic, Some(day)).map(|date| date.to_string())
        } else {
            state.and_then(|state| state.next_review_at.clone())
        },
        gaps: if passed {
            Vec::new()
        } else {
            verdict.gaps.clone()
        },
        attempts,
        practice: state
            .map(|state| state.practice.clone())
            .unwrap_or_default(),
    };

    Applied {
        split_suggested: failing_streak(&state.attempts) >= ENOUGH_FAILURES,
        retry: retry(verdict),
        state,
    }
}

fn retry(verdict: &Verdict) -> Vec<String> {
    verdict
        .per_question
        .iter()
        .filter(|answer| answer.outcome != Outcome::Ok)
        .map(|answer| answer.id.clone())
        .collect()
}
