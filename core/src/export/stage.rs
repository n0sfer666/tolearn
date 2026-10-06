use crate::stage::{Check, Practice, Question, Stage};

use super::blocks::blocks;
use super::escape::opened;
use super::lines::{Doc, flat, label, spanned};
use super::words::Words;

pub fn page(stage: &Stage, node: &str, words: &Words) -> String {
    let mut doc = Doc::default();
    doc.heading(1, &stage.title);
    doc.raw(vec![format!("← [{}](index.md)", label(node))]);
    blocks(&mut doc, &stage.blocks, 2, words);
    practice(&mut doc, &stage.practice, words);
    questions(&mut doc, &stage.questions, words);
    doc.text()
}

fn practice(doc: &mut Doc, practice: &Practice, words: &Words) {
    doc.heading(2, words.practice);
    blocks(doc, &practice.task, 3, words);
    doc.bullets(&[format!(
        "{}: {}",
        words.deliverable,
        flat(&practice.deliverable)
    )]);
    checks(doc, words.constraints, &practice.constraints, words);
    checks(doc, words.acceptance, &practice.acceptance, words);
}

fn checks(doc: &mut Doc, title: &str, checks: &[Check], words: &Words) {
    if checks.is_empty() {
        return;
    }
    doc.heading(3, title);
    doc.raw(checks.iter().map(|check| item(check, words)).collect());
}

fn item(check: &Check, words: &Words) -> String {
    let command = check
        .check
        .as_deref()
        .map(|command| format!("{}: {}, ", words.check, spanned(command)))
        .unwrap_or_default();
    format!(
        "- {} — {command}{}: {}",
        opened(&flat(&check.claim)),
        words.expect,
        flat(&check.expect)
    )
}

fn questions(doc: &mut Doc, questions: &[Question], words: &Words) {
    if questions.is_empty() {
        return;
    }
    doc.heading(2, words.questions);
    let texts: Vec<String> = questions
        .iter()
        .map(|question| flat(&question.text))
        .collect();
    doc.numbered(&texts);
    doc.line(words.hidden);
}
