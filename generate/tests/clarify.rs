#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "clarify gate: a panic here is the report"
)]

#[allow(dead_code, reason = "the plan helpers are shared with the plan tests")]
mod support;

use std::path::PathBuf;

use support::{Scripted, Up};
use tolearn_core::Hours;
use tolearn_core::block::{Block, Kind};
use tolearn_core::program::{self, Program, StageRow};
use tolearn_core::stage::{self, Stage};
use tolearn_core::state::Turn;
use tolearn_generate::clarify::{
    self, ANSWER_CHARS, BLOCK_CHARS, CHAIN_CHARS, CLARIFY_PROMPT_CHARS, Doubt, FRAGMENT_CHARS,
    QUESTION_CHARS,
};
use tolearn_generate::ledger;
use tolearn_generate::plan::MAX_STAGES;
use tolearn_generate::stage::Place;
use tolearn_generate::{GenerateError, Step, online};

const PROGRAM: &str = "3f6c2a1e-8b4d-4c7a-9e21-5d0f7b3a6c84";
const NODE: &str = "9d1e7b40-2c6a-4f8e-b3d5-71a0c4e2f9b6";
const AT: i64 = 1_789_000_000;
const EARLIER: &str = "Прежние объяснения этого фрагмента";
const PICKED: &str = "Ученик выделил вот это место:";

fn chiptune() -> Program {
    program::parse(&std::fs::read_to_string("../examples/chiptune/program.yaml").unwrap()).unwrap()
}

fn voices() -> Stage {
    stage::parse(&std::fs::read_to_string("../examples/chiptune/stages/voices.yaml").unwrap())
        .unwrap()
}

fn paragraph(stage: &Stage) -> &Block {
    stage
        .blocks
        .iter()
        .find(|block| block.kind == Kind::Paragraph)
        .unwrap()
}

fn turn(asked: Option<&str>, answer: &str) -> Turn {
    Turn {
        asked: asked.map(str::to_owned),
        answer: answer.to_owned(),
    }
}

fn doubt<'a>(
    program: &'a Program,
    block: &'a Block,
    chain: &'a [Turn],
    question: Option<&'a str>,
) -> Doubt<'a> {
    Doubt {
        program: PROGRAM,
        node: NODE,
        place: Place::find(program, "voices").unwrap(),
        block,
        fragment: None,
        chain,
        question,
    }
}

fn picked<'a>(program: &'a Program, block: &'a Block, fragment: &'a str) -> Doubt<'a> {
    Doubt {
        fragment: Some(fragment),
        ..doubt(program, block, &[], None)
    }
}

fn scratch(name: &str) -> PathBuf {
    support::scratch::named(&format!("clarify-{name}"))
}

#[test]
fn the_prompt_carries_the_block_its_place_level_and_locale() {
    let program = chiptune();
    let stage = voices();
    let block = paragraph(&stage);
    let place = Place::find(&program, "voices").unwrap();

    let prompt = clarify::prompt(&doubt(&program, block, &[], None));

    assert!(prompt.contains(&block.text));
    assert!(prompt.contains(&format!("→ {}. {}", place.index + 1, place.row.title)));
    assert!(prompt.contains(&format!("Уровень ученика: {}", program.level)));
    assert!(prompt.contains(&format!("Язык программы: {}", program.generation.locale)));
    assert!(prompt.contains(&format!("Не длиннее {ANSWER_CHARS} знаков")));
    assert!(prompt.contains("Без заголовков, картинок, схем и ссылок"));
    assert!(!prompt.contains(EARLIER));
    assert!(!prompt.contains("Вопрос ученика"));
    assert!(!prompt.contains(PICKED));
}

#[test]
fn a_picked_fragment_stands_apart_before_the_block() {
    let program = chiptune();
    let stage = voices();
    let block = paragraph(&stage);

    let prompt = clarify::prompt(&picked(&program, block, "скважность"));

    let fragment = prompt.find(PICKED).unwrap();
    let whole = prompt.find("Непонятный фрагмент этапа:").unwrap();
    assert!(fragment < whole, "{prompt}");
    assert!(prompt.contains(&format!("{PICKED} скважность")));
    assert!(prompt.contains(&block.text));
}

#[test]
fn a_picked_fragment_is_cut_to_its_ceiling() {
    let program = chiptune();
    let stage = voices();

    let prompt = clarify::prompt(&picked(
        &program,
        paragraph(&stage),
        &"ф".repeat(FRAGMENT_CHARS + 500),
    ));

    assert!(prompt.contains(&"ф".repeat(FRAGMENT_CHARS)));
    assert!(!prompt.contains(&"ф".repeat(FRAGMENT_CHARS + 1)));
}

#[test]
fn a_follow_up_carries_the_whole_chain_and_the_question() {
    let program = chiptune();
    let stage = voices();
    let chain = [
        turn(None, "Пульс — прямоугольная волна."),
        turn(Some("А скважность?"), "Доля времени, когда волна наверху."),
    ];

    let prompt = clarify::prompt(&doubt(
        &program,
        paragraph(&stage),
        &chain,
        Some("Всё равно неясно, зачем 12,5%"),
    ));

    let earlier = prompt.find(EARLIER).unwrap();
    let first = prompt.find("Пульс — прямоугольная волна.").unwrap();
    let second = prompt.find("Доля времени, когда волна наверху.").unwrap();
    assert!(earlier < first && first < second);
    assert!(prompt.contains("Вопрос ученика: А скважность?"));
    assert!(prompt.contains("Вопрос ученика: Всё равно неясно, зачем 12,5%"));
}

#[test]
fn an_overflowing_chain_keeps_the_latest_turns_within_the_limit() {
    let program = chiptune();
    let stage = voices();
    let chain: Vec<Turn> = (0..300)
        .map(|index| {
            turn(
                None,
                &format!("Объяснение номер {index} про пульс и скважность."),
            )
        })
        .collect();

    let prompt = clarify::prompt(&doubt(&program, paragraph(&stage), &chain, None));

    let start = prompt.find(EARLIER).unwrap();
    let end = prompt.find("Правила:").unwrap();
    assert!(prompt[start..end].trim_end().chars().count() <= CHAIN_CHARS);
    assert!(prompt.contains("Объяснение номер 299 "));
    assert!(!prompt.contains("Объяснение номер 0 "));
}

#[test]
fn the_prompt_fits_its_ceiling_with_the_fullest_map_block_chain_and_question() {
    let mut program = chiptune();
    let title = "Этап с длинным названием про звук старых игровых приставок ".repeat(2);
    while program.map.stages.len() < MAX_STAGES {
        program.map.stages.push(StageRow {
            id: format!("extra-{}", program.map.stages.len()),
            title: title.clone(),
            hours: Hours { min: 1, max: 2 },
        });
    }
    let mut block = paragraph(&voices()).clone();
    block.text = "я".repeat(10_000);
    let chain = [turn(Some(&"в".repeat(3_000)), &"о".repeat(9_000))];
    let question = "ж".repeat(5_000);
    let fragment = "ф".repeat(5_000);

    let full = Doubt {
        fragment: Some(&fragment),
        ..doubt(&program, &block, &chain, Some(&question))
    };
    let prompt = clarify::prompt(&full);
    let size = prompt.chars().count();

    assert!(
        size <= CLARIFY_PROMPT_CHARS,
        "{size} > {CLARIFY_PROMPT_CHARS}"
    );
    assert!(prompt.contains(&"я".repeat(BLOCK_CHARS)));
    assert!(!prompt.contains(&"я".repeat(BLOCK_CHARS + 1)));
    assert!(prompt.contains(&"ж".repeat(QUESTION_CHARS)));
    assert!(!prompt.contains(&"ж".repeat(QUESTION_CHARS + 1)));
    assert!(prompt.contains(&"ф".repeat(FRAGMENT_CHARS)));
}

#[test]
fn the_limits_are_pinned() {
    assert_eq!(
        [
            CLARIFY_PROMPT_CHARS,
            BLOCK_CHARS,
            FRAGMENT_CHARS,
            CHAIN_CHARS,
            QUESTION_CHARS,
            ANSWER_CHARS
        ],
        [19_000, 4_000, 1_000, 6_000, 1_000, 1_500]
    );
}

#[test]
fn a_clarification_is_one_call_trimmed_and_written_to_the_ledger() {
    let program = chiptune();
    let stage = voices();
    let data = scratch("one");
    let model = Scripted::new(vec!["\n  Пульс — это волна.\n\n> Врезка\n".to_owned()]);

    let answer = clarify::clarify(
        &online(&Up, &model).unwrap(),
        &data,
        &doubt(&program, paragraph(&stage), &[], None),
        AT,
    )
    .unwrap();

    assert_eq!(answer, "Пульс — это волна.\n\n> Врезка");
    assert_eq!(model.prompts().len(), 1);
    let records = ledger::read(&ledger::path(&data, PROGRAM)).unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].step, Step::Clarify.label());
    assert_eq!(records[0].program.as_deref(), Some(NODE));
    assert_eq!(records[0].stage.as_deref(), Some("voices"));
    assert_eq!(records[0].at, Some(AT));
    let _ = std::fs::remove_dir_all(&data);
}

#[test]
fn an_empty_answer_is_refused_but_still_counted() {
    let program = chiptune();
    let stage = voices();
    let data = scratch("empty");
    let model = Scripted::new(vec![" \n ".to_owned()]);

    let refused = clarify::clarify(
        &online(&Up, &model).unwrap(),
        &data,
        &doubt(&program, paragraph(&stage), &[], None),
        AT,
    )
    .unwrap_err();

    assert_eq!(refused, GenerateError::Unclear);
    assert_eq!(refused.code(), "generate.unclear");
    assert_eq!(
        ledger::read(&ledger::path(&data, PROGRAM)).unwrap().len(),
        1
    );
    let _ = std::fs::remove_dir_all(&data);
}
