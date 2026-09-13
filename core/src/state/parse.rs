use std::collections::BTreeMap;

use super::choices::{GRADE, PASS, SITTING};
use super::render::SCHEMA;
use super::types::{Answered, Attempt, Clarification, Passed, StageState, State, Turn};
use crate::yaml::{ParseError, Reader, read};

pub fn parse(source: &str) -> Result<State, ParseError> {
    read(source, state)
}

fn state(node: &Reader<'_>) -> Result<State, ParseError> {
    node.field("schema")?
        .choice("state schema", &[(SCHEMA, ())])?;
    Ok(State {
        program: node.field("program")?.uuid()?,
        workdir: optional(node, "workdir", Reader::text)?,
        stages: keyed(&node.field("stages")?, stage)?,
        clarifications: node.field("clarifications")?.list(clarification)?,
    })
}

fn stage(node: &Reader<'_>) -> Result<StageState, ParseError> {
    Ok(StageState {
        opened: optional(node, "opened", Reader::date)?,
        passed: optional(node, "passed", passed)?,
        ticks: optional(node, "ticks", Reader::texts)?.unwrap_or_default(),
        answers: optional(node, "answers", |answers| keyed(answers, Reader::text))?
            .unwrap_or_default(),
        attempts: optional(node, "attempts", |attempts| attempts.list(attempt))?
            .unwrap_or_default(),
        since: optional(node, "since", |since| since.number(1))?
            .map_or(0, |since| usize::try_from(since).unwrap_or(usize::MAX)),
    })
}

fn passed(node: &Reader<'_>) -> Result<Passed, ParseError> {
    Ok(Passed {
        on: node.field("on")?.date()?,
        by: node.field("by")?.choice("way of passing", &PASS)?,
    })
}

fn attempt(node: &Reader<'_>) -> Result<Attempt, ParseError> {
    Ok(Attempt {
        on: node.field("on")?.date()?,
        by: node.field("by")?.choice("way of sitting", &SITTING)?,
        model: optional(node, "model", Reader::text)?,
        per_question: node.field("per_question")?.list(answered)?,
    })
}

fn answered(node: &Reader<'_>) -> Result<Answered, ParseError> {
    Ok(Answered {
        id: node.field("id")?.text()?,
        result: node.field("result")?.choice("result", &GRADE)?,
        missed: node.field("missed")?.texts()?,
    })
}

fn clarification(node: &Reader<'_>) -> Result<Clarification, ParseError> {
    Ok(Clarification {
        stage: node.field("stage")?.text()?,
        block: node.field("block")?.text()?,
        excerpt: node.field("excerpt")?.text()?,
        turns: node.field("turns")?.list(turn)?,
        clear: node.field("clear")?.flag()?,
    })
}

fn turn(node: &Reader<'_>) -> Result<Turn, ParseError> {
    Ok(Turn {
        asked: optional(node, "asked", Reader::text)?,
        answer: node.field("answer")?.text()?,
    })
}

fn optional<'a, T>(
    node: &Reader<'a>,
    name: &str,
    each: impl Fn(&Reader<'a>) -> Result<T, ParseError>,
) -> Result<Option<T>, ParseError> {
    node.optional_field(name)?.as_ref().map(each).transpose()
}

fn keyed<'a, T>(
    node: &Reader<'a>,
    each: impl Fn(&Reader<'a>) -> Result<T, ParseError>,
) -> Result<BTreeMap<String, T>, ParseError> {
    node.entries()?
        .into_iter()
        .map(|(key, value)| Ok((key, each(&value)?)))
        .collect()
}
