use std::fmt;

use saphyr::Yaml;

use super::types::{Answered, Attempt, Clarification, StageState, State, Turn};
use crate::yaml::{dump, flag, list, map, number, text};

pub const SCHEMA: &str = "tolearn/state/1";

pub fn render(state: &State) -> Result<String, fmt::Error> {
    let mut entries = vec![("schema", text(SCHEMA)), ("program", text(&state.program))];
    if let Some(workdir) = &state.workdir {
        entries.push(("workdir", text(workdir)));
    }
    let stages = state
        .stages
        .iter()
        .map(|(id, entry)| (id.as_str(), stage(entry)));
    entries.push(("stages", map(stages)));
    let clarifications = state.clarifications.iter().map(clarification);
    entries.push(("clarifications", list(clarifications)));
    dump(&map(entries))
}

fn stage(entry: &StageState) -> Yaml<'_> {
    let mut entries = Vec::new();
    if let Some(opened) = &entry.opened {
        entries.push(("opened", text(opened)));
    }
    if let Some(passed) = &entry.passed {
        let by = text(passed.by.label());
        entries.push(("passed", map([("on", text(&passed.on)), ("by", by)])));
    }
    if !entry.ticks.is_empty() {
        entries.push(("ticks", texts(&entry.ticks)));
    }
    if !entry.answers.is_empty() {
        let answers = entry
            .answers
            .iter()
            .map(|(id, answer)| (id.as_str(), text(answer)));
        entries.push(("answers", map(answers)));
    }
    if !entry.attempts.is_empty() {
        entries.push(("attempts", list(entry.attempts.iter().map(attempt))));
    }
    if entry.since > 0 {
        let since = u32::try_from(entry.since).unwrap_or(u32::MAX);
        entries.push(("since", number(since)));
    }
    map(entries)
}

fn attempt(attempt: &Attempt) -> Yaml<'_> {
    let mut entries = vec![("on", text(&attempt.on)), ("by", text(attempt.by.label()))];
    if let Some(model) = &attempt.model {
        entries.push(("model", text(model)));
    }
    let answered = attempt.per_question.iter().map(answered);
    entries.push(("per_question", list(answered)));
    map(entries)
}

fn answered(answered: &Answered) -> Yaml<'_> {
    map([
        ("id", text(&answered.id)),
        ("result", text(answered.result.label())),
        ("missed", texts(&answered.missed)),
    ])
}

fn clarification(clarification: &Clarification) -> Yaml<'_> {
    let mut entries = vec![
        ("stage", text(&clarification.stage)),
        ("block", text(&clarification.block)),
        ("excerpt", text(&clarification.excerpt)),
    ];
    if let Some(fragment) = &clarification.fragment {
        entries.push(("fragment", text(fragment)));
    }
    entries.push(("turns", list(clarification.turns.iter().map(turn))));
    entries.push(("clear", flag(clarification.clear)));
    map(entries)
}

fn turn(turn: &Turn) -> Yaml<'_> {
    let mut entries = Vec::new();
    if let Some(asked) = &turn.asked {
        entries.push(("asked", text(asked)));
    }
    entries.push(("answer", text(&turn.answer)));
    map(entries)
}

fn texts(values: &[String]) -> Yaml<'_> {
    list(values.iter().map(|value| text(value)))
}
