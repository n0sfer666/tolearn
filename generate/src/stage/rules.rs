use std::collections::BTreeSet;

use tolearn_core::block::{Block, Kind};
use tolearn_core::program::Violation;
use tolearn_core::stage::{self, Stage};

use super::draft::Draft;
use super::flaw::Flaw;
use super::part::Part;
use super::place::Place;
use super::{FIRST_STAGE_TOOLS, MAX_TERMS, MAX_THEORY_CHARS, MIN_QUESTIONS, MIN_THEORY_CHARS};

pub fn invariants(draft: &Draft, place: &Place<'_>) -> Vec<Flaw> {
    let mut flaws = draft.flaws.clone();
    format(&draft.stage, &mut flaws);
    theory(draft, &mut flaws);
    practice(draft, place, &mut flaws);
    questions(&draft.stage, &mut flaws);
    flaws
}

fn format(stage: &Stage, flaws: &mut Vec<Flaw>) {
    for block in stage.every_block() {
        if block.text.trim().is_empty() {
            flaws.push(Flaw::BlankBlock {
                block: block.id.clone(),
            });
        }
    }
    for violation in stage::check(stage) {
        let block = match &violation {
            Violation::BlockWithoutAsset {
                kind: Kind::Diagram,
                ..
            } => continue,
            Violation::BlockId { block, .. }
            | Violation::BlockWithoutAsset { block, .. }
            | Violation::ForeignBlockField { block, .. }
            | Violation::UnlicensedImage { block, .. }
            | Violation::MissingAsset { block, .. } => block.clone(),
            _ => String::new(),
        };
        let part = Some(Part::Block(block.clone()));
        if !flaws.iter().any(|flaw| flaw.part() == part) {
            flaws.push(Flaw::Format {
                block,
                reason: violation.to_string(),
            });
        }
    }
}

fn theory(draft: &Draft, flaws: &mut Vec<Flaw>) {
    let prose: Vec<&Block> = draft
        .stage
        .blocks
        .iter()
        .filter(|block| matches!(block.kind, Kind::Heading | Kind::Paragraph | Kind::Callout))
        .collect();
    if prose.iter().all(|block| block.kind == Kind::Heading) {
        flaws.push(Flaw::NoTheory);
    } else {
        let chars = prose.iter().map(|block| block.text.chars().count()).sum();
        if !(MIN_THEORY_CHARS..=MAX_THEORY_CHARS).contains(&chars) {
            flaws.push(Flaw::TheoryLength(chars));
        }
    }
    let terms = distinct(&draft.terms).len();
    if terms > MAX_TERMS {
        flaws.push(Flaw::Terms(terms));
    }
}

fn practice(draft: &Draft, place: &Place<'_>, flaws: &mut Vec<Flaw>) {
    let practice = &draft.stage.practice;
    if practice.task.is_empty() {
        flaws.push(Flaw::Practice("нет задания: task пуст".to_owned()));
    }
    if practice.deliverable.trim().is_empty() {
        flaws.push(Flaw::Practice(
            "не сказано, что ученик сдаёт: deliverable пуст".to_owned(),
        ));
    }
    if practice.acceptance.is_empty() {
        flaws.push(Flaw::Practice(
            "нет критериев приёмки: acceptance пуст".to_owned(),
        ));
    }
    for check in practice.constraints.iter().chain(&practice.acceptance) {
        if check.claim.trim().is_empty() {
            flaws.push(Flaw::Practice(format!("у пункта {} пусто claim", check.id)));
        }
        if check.expect.trim().is_empty() {
            flaws.push(Flaw::Practice(format!(
                "у пункта {} пусто expect",
                check.id
            )));
        }
    }
    let tools = distinct(&draft.tools);
    if place.first() && tools.len() != FIRST_STAGE_TOOLS {
        flaws.push(Flaw::Tools(tools.into_iter().map(str::to_owned).collect()));
    }
}

fn questions(stage: &Stage, flaws: &mut Vec<Flaw>) {
    if stage.questions.len() < MIN_QUESTIONS {
        flaws.push(Flaw::Questions(format!(
            "их {}, а нужно не меньше {MIN_QUESTIONS}, у каждого эталонный ответ",
            stage.questions.len()
        )));
    }
    for question in &stage.questions {
        if question.text.trim().is_empty() {
            flaws.push(Flaw::Questions(format!("у {} нет текста", question.id)));
        }
        if question.answer.trim().is_empty() {
            flaws.push(Flaw::Questions(format!(
                "у {} нет эталонного ответа",
                question.id
            )));
        }
    }
}

fn distinct(names: &[String]) -> BTreeSet<&str> {
    names
        .iter()
        .map(|name| name.trim())
        .filter(|name| !name.is_empty())
        .collect()
}
