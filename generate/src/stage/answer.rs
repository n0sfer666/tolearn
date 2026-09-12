use tolearn_core::block::{self, Block, Kind};
use tolearn_core::stage::{Check, Practice, Question, Stage};

use super::cited::cited;
use super::draft::Draft;
use super::flaw::Flaw;
use super::gathered::Gathered;
use super::place::Place;
use super::raw::{self, Raw, RawBlock, RawCheck};

pub(super) fn read(text: &str, place: &Place<'_>, gathered: &Gathered) -> Result<Draft, Flaw> {
    build(&raw::parse(text)?, place, gathered)
}

pub(super) fn build(raw: &Raw, place: &Place<'_>, gathered: &Gathered) -> Result<Draft, Flaw> {
    let every: Vec<&RawBlock> = raw.blocks.iter().chain(&raw.practice.task).collect();
    let shaped = every
        .iter()
        .map(|raw| shaped(raw, gathered))
        .collect::<Result<Vec<_>, _>>()?;
    let ids = block::ids(shaped.iter().map(|block| block.text.as_str()));
    let mut flaws = Vec::new();
    let mut blocks = Vec::new();
    let mut references: Vec<String> = Vec::new();
    for ((mut block, raw), id) in shaped.into_iter().zip(every).zip(ids) {
        block.id = id;
        cited(&block, &raw.sources, gathered, &mut flaws);
        for source in &raw.sources {
            let known = gathered.book(source).is_some() || gathered.page(source).is_some();
            if known && !references.contains(source) {
                references.push(source.clone());
            }
        }
        if block.kind == Kind::Image && block.asset.is_none() {
            flaws.push(Flaw::UnknownImage {
                block: block.id.clone(),
                image: raw.image.clone().unwrap_or_default(),
            });
        }
        blocks.push(block);
    }
    let task = blocks.split_off(raw.blocks.len());
    Ok(Draft {
        stage: Stage {
            id: place.row.id.clone(),
            title: place.row.title.clone(),
            blocks,
            practice: Practice {
                task,
                deliverable: raw.practice.deliverable.clone(),
                constraints: checks('c', &raw.practice.constraints),
                acceptance: checks('a', &raw.practice.acceptance),
            },
            questions: raw
                .questions
                .iter()
                .enumerate()
                .map(|(index, question)| Question {
                    id: format!("q{}", index + 1),
                    text: question.text.clone(),
                    answer: question.answer.clone(),
                })
                .collect(),
        },
        terms: raw.terms.clone(),
        tools: raw.tools.clone(),
        cited: references,
        flaws,
    })
}

fn shaped(raw: &RawBlock, gathered: &Gathered) -> Result<Block, Flaw> {
    let kind = Kind::ALL
        .into_iter()
        .find(|kind| kind.label() == raw.kind)
        .ok_or_else(|| Flaw::Unreadable(format!("неизвестный вид блока «{}»", raw.kind)))?;
    let mut block = Block {
        id: String::new(),
        kind,
        text: raw.text.clone(),
        lang: None,
        asset: None,
        license: None,
        attribution: None,
        source: None,
    };
    match kind {
        Kind::Code => block.lang.clone_from(&raw.lang),
        Kind::Image => {
            if let Some(found) = raw.image.as_deref().and_then(|id| gathered.image(id)) {
                if block.text.trim().is_empty() {
                    block.text.clone_from(&found.block.text);
                }
                block.asset.clone_from(&found.block.asset);
                block.license.clone_from(&found.block.license);
                block.attribution.clone_from(&found.block.attribution);
                block.source.clone_from(&found.block.source);
            }
        }
        _ => {}
    }
    Ok(block)
}

fn checks(prefix: char, raw: &[RawCheck]) -> Vec<Check> {
    raw.iter()
        .enumerate()
        .map(|(index, check)| Check {
            id: format!("{prefix}{}", index + 1),
            claim: check.claim.clone(),
            check: check
                .check
                .clone()
                .filter(|command| !command.trim().is_empty()),
            expect: check.expect.clone(),
        })
        .collect()
}
