mod block;
mod error;
mod types;

pub use error::VerdictError;
pub use types::Verdict;

use crate::progress::{NEXT_ACTION, VERDICT, answer, flag_or_false, text_or_none, texts_or_empty};
use crate::topic::Topic;
use crate::yaml::{ParseError, Reader, read};

const OPTIONAL: [&str; 10] = [
    "date",
    "model",
    "hinted",
    "practice_accepted",
    "failed_checks",
    "gaps",
    "calibration",
    "notes",
    "next_action",
    "retry_after_days",
];

pub fn parse(text: &str, topic: &Topic) -> Result<Verdict, VerdictError> {
    let raw = block::last(text).ok_or(VerdictError::NoJson)?;
    read(raw, |node| verdict(node, topic, raw)).map_err(|error| VerdictError::Malformed {
        reason: error.message().to_owned(),
    })?
}

fn verdict(
    node: &Reader<'_>,
    topic: &Topic,
    raw: &str,
) -> Result<Result<Verdict, VerdictError>, ParseError> {
    let Some(named) = node.optional_field("topic_id")? else {
        return Ok(Err(lacks("topic_id")));
    };
    let topic_id = named.text()?;
    if topic_id != topic.id {
        return Ok(Err(VerdictError::WrongTopic {
            expected: topic.id.clone(),
            found: topic_id,
        }));
    }
    let Some(graded) = node.optional_field("verdict")? else {
        return Ok(Err(lacks("verdict")));
    };
    let per_question = match node.optional_field("per_question")? {
        Some(field) => field.list(answer)?,
        None => Vec::new(),
    };
    if per_question.is_empty() {
        return Ok(Err(VerdictError::NoAnswers));
    }
    Ok(Ok(Verdict {
        unknown_questions: unknown(&per_question, topic),
        missing: absent(node)?,
        result: graded.choice("verdict", &VERDICT)?,
        date: node
            .optional_field("date")?
            .map(|value| value.date())
            .transpose()?,
        model: text_or_none(node, "model")?,
        hinted: flag_or_false(node, "hinted")?,
        practice_accepted: flag_or_false(node, "practice_accepted")?,
        failed_checks: texts_or_empty(node, "failed_checks")?,
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
        raw: raw.to_owned(),
        topic_id,
        per_question,
    }))
}

fn unknown(answers: &[crate::progress::Answer], topic: &Topic) -> Vec<String> {
    answers
        .iter()
        .filter(|given| !topic.questions.iter().any(|asked| asked.id == given.id))
        .map(|given| given.id.clone())
        .collect()
}

fn absent(node: &Reader<'_>) -> Result<Vec<String>, ParseError> {
    let mut missing = Vec::new();
    for name in OPTIONAL {
        if node.optional_field(name)?.is_none() {
            missing.push(name.to_owned());
        }
    }
    Ok(missing)
}

fn lacks(field: &str) -> VerdictError {
    VerdictError::Missing {
        field: field.to_owned(),
    }
}
