use serde::Deserialize;
use tolearn_core::block::{self, Block, Kind};
use tolearn_core::stage::{Check, Practice, Question, Stage};

use crate::object::object;

use super::cited::cited;
use super::draft::Draft;
use super::flaw::Flaw;
use super::gathered::Gathered;
use super::place::Place;

#[derive(Deserialize)]
struct Raw {
    blocks: Vec<RawBlock>,
    practice: RawPractice,
    #[serde(default)]
    questions: Vec<RawQuestion>,
    #[serde(default)]
    terms: Vec<String>,
    #[serde(default)]
    tools: Vec<String>,
}

#[derive(Deserialize)]
struct RawBlock {
    kind: String,
    #[serde(default)]
    text: String,
    #[serde(default)]
    lang: Option<String>,
    #[serde(default)]
    image: Option<String>,
    #[serde(default)]
    sources: Vec<String>,
}

#[derive(Deserialize)]
struct RawPractice {
    #[serde(default)]
    task: Vec<RawBlock>,
    #[serde(default)]
    deliverable: String,
    #[serde(default)]
    constraints: Vec<RawCheck>,
    #[serde(default)]
    acceptance: Vec<RawCheck>,
}

#[derive(Deserialize)]
struct RawCheck {
    claim: String,
    #[serde(default)]
    check: Option<String>,
    expect: String,
}

#[derive(Deserialize)]
struct RawQuestion {
    text: String,
    answer: String,
}

struct Wish {
    sources: Vec<String>,
    image: Option<String>,
}

pub(super) fn read(text: &str, place: &Place<'_>, gathered: &Gathered) -> Result<Draft, Flaw> {
    let raw: Raw = serde_json::from_str(object(text).map_err(Flaw::Unreadable)?)
        .map_err(|error| Flaw::Unreadable(error.to_string()))?;
    let theory = raw.blocks.len();
    let shaped = raw
        .blocks
        .into_iter()
        .chain(raw.practice.task)
        .map(|raw| shaped(raw, gathered))
        .collect::<Result<Vec<_>, _>>()?;
    let ids = block::ids(shaped.iter().map(|(block, _)| block.text.as_str()));
    let mut flaws = Vec::new();
    let mut blocks = Vec::new();
    for ((mut block, wish), id) in shaped.into_iter().zip(ids) {
        block.id = id;
        cited(&block, &wish.sources, gathered, &mut flaws);
        if block.kind == Kind::Image && block.asset.is_none() {
            flaws.push(Flaw::UnknownImage {
                block: block.id.clone(),
                image: wish.image.unwrap_or_default(),
            });
        }
        blocks.push(block);
    }
    let task = blocks.split_off(theory);
    Ok(Draft {
        stage: Stage {
            id: place.row.id.clone(),
            title: place.row.title.clone(),
            blocks,
            practice: Practice {
                task,
                deliverable: raw.practice.deliverable,
                constraints: checks('c', raw.practice.constraints),
                acceptance: checks('a', raw.practice.acceptance),
            },
            questions: raw
                .questions
                .into_iter()
                .enumerate()
                .map(|(index, question)| Question {
                    id: format!("q{}", index + 1),
                    text: question.text,
                    answer: question.answer,
                })
                .collect(),
        },
        terms: raw.terms,
        tools: raw.tools,
        flaws,
    })
}

fn shaped(raw: RawBlock, gathered: &Gathered) -> Result<(Block, Wish), Flaw> {
    let kind = Kind::ALL
        .into_iter()
        .find(|kind| kind.label() == raw.kind)
        .ok_or_else(|| Flaw::Unreadable(format!("неизвестный вид блока «{}»", raw.kind)))?;
    let mut block = Block {
        id: String::new(),
        kind,
        text: raw.text,
        lang: None,
        asset: None,
        license: None,
        attribution: None,
        source: None,
    };
    match kind {
        Kind::Code => block.lang = raw.lang,
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
    Ok((
        block,
        Wish {
            sources: raw.sources,
            image: raw.image,
        },
    ))
}

fn checks(prefix: char, raw: Vec<RawCheck>) -> Vec<Check> {
    raw.into_iter()
        .enumerate()
        .map(|(index, check)| Check {
            id: format!("{prefix}{}", index + 1),
            claim: check.claim,
            check: check.check.filter(|command| !command.trim().is_empty()),
            expect: check.expect,
        })
        .collect()
}
