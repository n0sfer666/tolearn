#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "clarifications gate: a panic here is the report"
)]

mod support;

use support::read;
use support::states::PROGRAM;
use tolearn_core::state::{EXCERPT_CHARS, State, Turn, excerpt, parse, render};

const NODE: &str = "9d1e7b40-2c6a-4f8e-b3d5-71a0c4e2f9b6";

fn turn(asked: Option<&str>, answer: &str) -> Turn {
    Turn {
        asked: asked.map(str::to_owned),
        answer: answer.to_owned(),
    }
}

fn chained() -> State {
    let mut state = State::new(PROGRAM);
    state.clarify(
        PROGRAM,
        "voices",
        "a1b2c3d4",
        "Пульс",
        None,
        turn(None, "Прямоугольная волна."),
    );
    state.clarify(
        NODE,
        "voices",
        "a1b2c3d4",
        "Пульс",
        None,
        turn(None, "Другой узел."),
    );
    state.clarify(
        PROGRAM,
        "tracker",
        "e5f6a7b8",
        "Трекер",
        Some("скважность"),
        turn(None, "Таблица."),
    );
    state.clarify(
        PROGRAM,
        "voices",
        "c9d0e1f2",
        "Шум",
        None,
        turn(Some("Зачем?"), "Для ударных."),
    );
    state
}

#[test]
fn a_clarification_opens_a_chain_under_its_stage_and_block() {
    let mut state = State::new(PROGRAM);

    let index = state.clarify(
        PROGRAM,
        "voices",
        "a1b2c3d4",
        "Пульс",
        None,
        turn(None, "Волна."),
    );

    assert_eq!(index, 0);
    let chain = state.chain(PROGRAM, "voices", 0).unwrap();
    assert_eq!(chain.stage, format!("{PROGRAM}/voices"));
    assert_eq!(chain.block, "a1b2c3d4");
    assert_eq!(chain.excerpt, "Пульс");
    assert_eq!(chain.fragment, None);
    assert_eq!(chain.turns, [turn(None, "Волна.")]);
    assert!(!chain.clear);
}

#[test]
fn a_stage_sees_only_its_own_chains_with_their_places_in_the_file() {
    let state = chained();

    let indexes: Vec<usize> = state
        .clarifications_of(PROGRAM, "voices")
        .into_iter()
        .map(|(index, _)| index)
        .collect();

    assert_eq!(indexes, [0, 3]);
    assert!(state.chain(PROGRAM, "voices", 1).is_none());
    assert!(state.chain(PROGRAM, "voices", 2).is_none());
    assert!(state.chain(PROGRAM, "voices", 9).is_none());
}

#[test]
fn a_chain_grows_and_closes_only_from_its_own_stage() {
    let mut state = chained();

    assert!(state.chain_mut(NODE, "tracker", 0).is_none());
    let chain = state.chain_mut(PROGRAM, "voices", 0).unwrap();
    chain
        .turns
        .push(turn(Some("А скважность?"), "Доля верхней части."));
    chain.clear = true;

    let chain = state.chain(PROGRAM, "voices", 0).unwrap();
    assert_eq!(chain.turns.len(), 2);
    assert!(chain.clear);
}

#[test]
fn removal_takes_only_a_chain_of_the_stage() {
    let mut state = chained();

    assert!(!state.unclarify(PROGRAM, "voices", 1));
    assert_eq!(state.clarifications.len(), 4);
    assert!(state.unclarify(PROGRAM, "voices", 0));

    assert_eq!(state.clarifications.len(), 3);
    assert_eq!(state.clarifications[0].stage, format!("{NODE}/voices"));
}

#[test]
fn chains_survive_the_state_file() {
    let state = chained();

    assert_eq!(parse(&render(&state).unwrap()).unwrap(), state);
}

#[test]
fn the_excerpt_is_the_start_of_the_block_on_one_line() {
    assert_eq!(
        excerpt("Чип держит\n\n  пять   каналов"),
        "Чип держит пять каналов"
    );
    assert_eq!(excerpt(&"я".repeat(500)).chars().count(), EXCERPT_CHARS);
    assert_eq!(EXCERPT_CHARS, 80);
}

#[test]
fn the_picked_fragment_lives_next_to_the_excerpt_and_survives_the_file() {
    let mut state = State::new(PROGRAM);

    state.clarify(
        PROGRAM,
        "voices",
        "a1b2c3d4",
        "Чип держит пять каналов",
        Some("пять каналов"),
        turn(None, "Волна."),
    );

    let written = render(&state).unwrap();
    assert!(written.contains("fragment: пять каналов"), "{written}");
    assert_eq!(parse(&written).unwrap(), state);
}

#[test]
fn a_chain_written_before_the_fragment_reads_without_it() {
    let before = read("fixtures/v2/states/before-fragment.yaml");

    let state = parse(&before).unwrap();

    let chain = state.chain(PROGRAM, "voices", 0).unwrap();
    assert_eq!(chain.excerpt, "Звуковой чип NES");
    assert_eq!(chain.fragment, None);
    assert_eq!(render(&state).unwrap(), before);
}
