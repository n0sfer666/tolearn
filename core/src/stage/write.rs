use std::fmt;

use saphyr::Yaml;

use super::parse::SCHEMA;
use super::types::{Check, Question, Stage};
use crate::block::written;
use crate::yaml::{dump, list, map, text};

pub fn write(stage: &Stage) -> Result<String, fmt::Error> {
    let practice = &stage.practice;
    dump(&map([
        ("schema", text(SCHEMA)),
        ("id", text(&stage.id)),
        ("title", text(&stage.title)),
        ("blocks", list(stage.blocks.iter().map(written))),
        (
            "practice",
            map([
                ("task", list(practice.task.iter().map(written))),
                ("deliverable", text(&practice.deliverable)),
                ("constraints", list(practice.constraints.iter().map(check))),
                ("acceptance", list(practice.acceptance.iter().map(check))),
            ]),
        ),
        ("questions", list(stage.questions.iter().map(question))),
    ]))
}

fn check(check: &Check) -> Yaml<'_> {
    let mut entries = vec![("id", text(&check.id)), ("claim", text(&check.claim))];
    entries.extend(
        check
            .check
            .as_deref()
            .map(|command| ("check", text(command))),
    );
    entries.push(("expect", text(&check.expect)));
    map(entries)
}

fn question(question: &Question) -> Yaml<'_> {
    map([
        ("id", text(&question.id)),
        ("text", text(&question.text)),
        ("answer", text(&question.answer)),
    ])
}
