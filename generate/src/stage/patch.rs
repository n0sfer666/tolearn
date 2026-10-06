use std::collections::BTreeMap;

use serde::Deserialize;

use super::flaw::Flaw;
use super::mend::Mending;
use super::part::Part;
use super::raw::{self, Raw, RawBlock, RawPractice, RawQuestion};

#[derive(Deserialize)]
struct Mended {
    #[serde(default)]
    blocks: BTreeMap<String, RawBlock>,
    theory: Option<Vec<RawBlock>>,
    practice: Option<RawPractice>,
    questions: Option<Vec<RawQuestion>>,
    terms: Option<Vec<String>>,
    tools: Option<Vec<String>>,
}

pub(super) fn patch(open: &Mending, text: &str) -> Result<Raw, Flaw> {
    let mended: Mended = raw::parse(text)?;
    let mut patched = open.raw.clone();
    let theory = open.raw.blocks.len();
    for (id, block) in mended.blocks {
        if !open.parts.contains(&Part::Block(id.clone())) {
            continue;
        }
        let Some(index) = open.ids.iter().position(|known| *known == id) else {
            continue;
        };
        let slot = if index < theory {
            patched.blocks.get_mut(index)
        } else {
            patched.practice.task.get_mut(index - theory)
        };
        if let Some(slot) = slot {
            *slot = block;
        }
    }
    let whole = |part: Part| open.parts.contains(&part);
    if let Some(blocks) = mended.theory.filter(|_| whole(Part::Theory)) {
        patched.blocks = blocks;
    }
    if let Some(terms) = mended.terms.filter(|_| whole(Part::Theory)) {
        patched.terms = terms;
    }
    if let Some(practice) = mended.practice.filter(|_| whole(Part::Practice)) {
        patched.practice = practice;
    }
    if let Some(tools) = mended.tools.filter(|_| whole(Part::Practice)) {
        patched.tools = tools;
    }
    if let Some(questions) = mended.questions.filter(|_| whole(Part::Questions)) {
        patched.questions = questions;
    }
    Ok(patched)
}
