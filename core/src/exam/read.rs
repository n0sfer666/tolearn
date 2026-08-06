use crate::progress::{answer, texts_or_empty};
use crate::topic::Question;
use crate::verdict::block;
use crate::yaml::{ParseError, Reader, read};

use super::error::StepError;
use super::types::{ARTIFACT, NEXT, Next, Shown, Step};

pub fn step(text: &str, question: &Question) -> Result<Step, StepError> {
    inside(text, |node| graded(node, question))
}

pub fn shown(text: &str) -> Result<Shown, StepError> {
    inside(text, |node| {
        Ok(Ok(Shown {
            artifact: node.field("artifact")?.choice("artifact", &ARTIFACT)?,
            failed_checks: texts_or_empty(node, "failed_checks")?,
            next: next(node)?,
            say: say(node)?,
        }))
    })
}

fn inside<T>(
    text: &str,
    parse: impl FnOnce(&Reader<'_>) -> Result<Result<T, StepError>, ParseError>,
) -> Result<T, StepError> {
    let raw = block::last(text).ok_or(StepError::NoJson)?;
    read(raw, parse).map_err(|error| StepError::Malformed {
        reason: error.message().to_owned(),
    })?
}

fn graded(node: &Reader<'_>, question: &Question) -> Result<Result<Step, StepError>, ParseError> {
    let answer = answer(node)?;
    if answer.id != question.id {
        return Ok(Err(StepError::WrongQuestion {
            expected: question.id.clone(),
            found: answer.id,
        }));
    }
    Ok(Ok(Step {
        next: next(node)?,
        say: say(node)?,
        answer,
    }))
}

fn next(node: &Reader<'_>) -> Result<Next, ParseError> {
    match node.optional_field("next")? {
        Some(field) => field.choice("next step", &NEXT),
        None => Ok(Next::Close),
    }
}

fn say(node: &Reader<'_>) -> Result<String, ParseError> {
    Ok(node
        .optional_field("say")?
        .map(|field| field.any_text())
        .transpose()?
        .unwrap_or_default())
}
