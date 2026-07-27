use super::enums::{NEXT_ACTION, OUTCOME, SOURCE, STATUS, VERDICT};
use super::types::{Answer, Attempt, Progress, TopicState};
use crate::yaml::{ParseError, Reader, read};

pub fn parse(source: &str) -> Result<Progress, ParseError> {
    read(source, progress)
}

fn progress(node: &Reader<'_>) -> Result<Progress, ParseError> {
    Ok(Progress {
        schema: node.field("schema")?.text()?,
        roadmap_id: node.field("roadmap_id")?.text()?,
        topics: node
            .field("topics")?
            .entries()?
            .iter()
            .map(|(id, value)| Ok((id.clone(), state(value)?)))
            .collect::<Result<Vec<_>, ParseError>>()?,
    })
}

fn state(node: &Reader<'_>) -> Result<TopicState, ParseError> {
    Ok(TopicState {
        status: node.field("status")?.choice("status", &STATUS)?,
        attempts: node.field("attempts")?.list(attempt)?,
        passed_at: node.field("passed_at")?.optional_date()?,
        next_review_at: node.field("next_review_at")?.optional_date()?,
        gaps: node.field("gaps")?.texts()?,
    })
}

fn attempt(node: &Reader<'_>) -> Result<Attempt, ParseError> {
    Ok(Attempt {
        at: node.field("at")?.moment()?,
        source: node.field("source")?.choice("attempt source", &SOURCE)?,
        verdict: node.field("verdict")?.choice("verdict", &VERDICT)?,
        model: text_or_none(node, "model")?,
        hinted: flag_or_false(node, "hinted")?,
        practice_accepted: flag_or_false(node, "practice_accepted")?,
        failed_checks: texts_or_empty(node, "failed_checks")?,
        per_question: node.field("per_question")?.list(answer)?,
        gaps: texts_or_empty(node, "gaps")?,
        calibration: text_or_none(node, "calibration")?,
        notes: texts_or_empty(node, "notes")?,
        next_action: node
            .optional_field("next_action")?
            .map(|value| value.choice("next action", &NEXT_ACTION))
            .transpose()?,
        retry_after_days: node
            .optional_field("retry_after_days")?
            .map(|value| value.number(0))
            .transpose()?,
        raw: node.field("raw")?.any_text()?,
    })
}

fn answer(node: &Reader<'_>) -> Result<Answer, ParseError> {
    Ok(Answer {
        id: node.field("id")?.text()?,
        outcome: node.field("result")?.choice("answer result", &OUTCOME)?,
        quote: text_or_none(node, "quote")?,
        missed: texts_or_empty(node, "missed")?,
        signal_extension: flag_or_false(node, "signal_extension")?,
    })
}

fn text_or_none(node: &Reader<'_>, name: &str) -> Result<Option<String>, ParseError> {
    node.optional_field(name)?
        .map(|value| value.optional_text())
        .transpose()
        .map(Option::flatten)
}

fn texts_or_empty(node: &Reader<'_>, name: &str) -> Result<Vec<String>, ParseError> {
    Ok(node
        .optional_field(name)?
        .map(|value| value.texts())
        .transpose()?
        .unwrap_or_default())
}

fn flag_or_false(node: &Reader<'_>, name: &str) -> Result<bool, ParseError> {
    Ok(node
        .optional_field(name)?
        .map(|value| value.flag())
        .transpose()?
        .unwrap_or(false))
}
