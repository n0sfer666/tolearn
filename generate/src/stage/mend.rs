use std::collections::BTreeMap;

use serde::Serialize;

use super::flaw::Flaw;
use super::gathered::Gathered;
use super::part::Part;
use super::place::Place;
use super::prompt;
use super::raw::{Raw, RawBlock, RawPractice, RawQuestion};

#[derive(Debug)]
pub(super) struct Mending {
    pub(super) raw: Raw,
    pub(super) ids: Vec<String>,
    pub(super) parts: Vec<Part>,
    pub(super) flaws: Vec<Flaw>,
}

#[derive(Serialize)]
struct Sent<'a> {
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    blocks: BTreeMap<&'a str, &'a RawBlock>,
    #[serde(skip_serializing_if = "Option::is_none")]
    theory: Option<&'a [RawBlock]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    practice: Option<&'a RawPractice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    questions: Option<&'a [RawQuestion]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    terms: Option<&'a [String]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<&'a [String]>,
}

pub(super) fn whole(task: &str, answer: &str, flaws: &[Flaw]) -> String {
    format!(
        "{task}\n\n\
         Твой прошлый ответ:\n{answer}\n\n\
         В нём нарушены правила:\n{}\n\n\
         Исправь нарушения и пришли этап целиком, снова одним объектом JSON.",
        listed(flaws)
    )
}

pub(super) fn parts(
    place: &Place<'_>,
    gathered: &Gathered,
    open: &Mending,
    flaws: &[Flaw],
) -> String {
    let raw = &open.raw;
    let asked = |part: Part| open.parts.contains(&part);
    let theory = asked(Part::Theory);
    let practice = asked(Part::Practice);
    let sent = Sent {
        blocks: open
            .ids
            .iter()
            .zip(raw.blocks.iter().chain(&raw.practice.task))
            .filter(|(id, _)| asked(Part::Block((*id).clone())))
            .map(|(id, block)| (id.as_str(), block))
            .collect(),
        theory: theory.then_some(raw.blocks.as_slice()),
        practice: practice.then_some(&raw.practice),
        questions: asked(Part::Questions).then_some(raw.questions.as_slice()),
        terms: theory.then_some(raw.terms.as_slice()),
        tools: practice.then_some(raw.tools.as_slice()),
    };
    format!(
        "Ты чинишь один этап учебной программы. Код проверил этап и нашёл нарушения в частях ниже. Остальные части уже прошли проверку: их не присылай, они не изменятся.\n\n\
         {}\n\
         Язык программы: {} — на нём пиши весь текст.\n\n\
         Проверенные источники:\n{}\n\n\
         Правила:\n{}\n\n\
         Нарушения:\n{}\n\n\
         Части с нарушениями:\n{}\n\n\
         Ответь одним объектом JSON с теми же ключами без пояснений: в blocks — каждый блок под тем же ключом, theory — вся теория заново, practice — практика целиком, questions — все вопросы, terms и tools — списки заново.",
        prompt::whereabouts(place),
        place.program.generation.locale,
        prompt::listed(gathered, false),
        prompt::rules(place),
        listed(flaws),
        serde_json::to_string(&sent).unwrap_or_default()
    )
}

fn listed(flaws: &[Flaw]) -> String {
    flaws
        .iter()
        .map(|flaw| format!("- {flaw}"))
        .collect::<Vec<_>>()
        .join("\n")
}
