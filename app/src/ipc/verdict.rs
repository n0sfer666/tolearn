use tolearn_core::status::from_verdict;
use tolearn_core::topic::Topic;
use tolearn_core::verdict::{Verdict, parse};

use crate::ipc::error::IpcError;
use crate::ipc::open::Opened;
use crate::ipc::types::{AnswerView, VerdictView};

pub fn read<'a>(opened: &'a Opened, id: &str) -> Result<&'a Topic, IpcError> {
    opened
        .scan
        .topics
        .iter()
        .find(|topic| topic.id == id)
        .ok_or_else(|| IpcError::unknown_topic(id))
}

pub fn of(text: &str, topic: &Topic) -> Result<Verdict, IpcError> {
    parse(text, topic).map_err(IpcError::from)
}

pub fn view(verdict: &Verdict) -> VerdictView {
    VerdictView {
        result: verdict.result.label().to_owned(),
        status: from_verdict(verdict.result).label().to_owned(),
        gaps: verdict.gaps.clone(),
        notes: verdict.notes.clone(),
        missing: verdict.missing.clone(),
        unknown_questions: verdict.unknown_questions.clone(),
        failed_checks: verdict.failed_checks.clone(),
        per_question: verdict.per_question.iter().map(answer).collect(),
        hinted: verdict.hinted,
        practice_accepted: verdict.practice_accepted,
        next_action: verdict.next_action.map(|action| action.label().to_owned()),
        retry_after_days: verdict.retry_after_days,
    }
}

fn answer(answer: &tolearn_core::progress::Answer) -> AnswerView {
    AnswerView {
        id: answer.id.clone(),
        outcome: answer.outcome.label().to_owned(),
        missed: answer.missed.clone(),
    }
}
