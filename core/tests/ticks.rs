#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "ticks gate: a panic here is the report"
)]

mod support;

use support::states::{PROGRAM, scratch};
use tolearn_core::state::{Pass, Passed, State, Status, key};

const LEAF: &str = "e4c90b7a-15f2-4d6e-8b38-0a2c6f9d1e47";

#[test]
fn галочка_ставится_один_раз_и_снимается() {
    let mut state = State::new(PROGRAM);

    assert!(state.tick(PROGRAM, "voices", "c1", true));
    assert!(state.tick(PROGRAM, "voices", "a1", true));
    assert!(!state.tick(PROGRAM, "voices", "c1", true));
    assert_eq!(state.ticks(PROGRAM, "voices"), ["c1", "a1"]);

    assert!(state.tick(PROGRAM, "voices", "c1", false));
    assert!(!state.tick(PROGRAM, "voices", "c1", false));
    assert_eq!(state.ticks(PROGRAM, "voices"), ["a1"]);
}

#[test]
fn галочка_не_меняет_статус_этапа() {
    let mut state = State::new(PROGRAM);

    state.tick(PROGRAM, "voices", "c1", true);
    assert_eq!(state.status(PROGRAM, "voices"), Status::Fresh);

    state.open(PROGRAM, "voices", "2026-09-10");
    state.tick(PROGRAM, "voices", "a1", true);
    assert_eq!(state.status(PROGRAM, "voices"), Status::Opened);

    state
        .stages
        .get_mut(&key(PROGRAM, "voices"))
        .unwrap()
        .passed = Some(Passed {
        on: "2026-09-11".to_owned(),
        by: Pass::Skip,
    });
    state.tick(PROGRAM, "voices", "a1", false);
    assert_eq!(state.status(PROGRAM, "voices"), Status::Passed(Pass::Skip));
}

#[test]
fn галочки_одноимённых_этапов_в_двух_узлах_раздельны() {
    let mut state = State::new(PROGRAM);

    state.tick(LEAF, "setup", "c1", true);

    assert_eq!(state.ticks(LEAF, "setup"), ["c1"]);
    assert!(state.ticks(PROGRAM, "setup").is_empty());
}

#[test]
fn галочка_переживает_перечитывание() {
    let data = scratch("ticks");

    State::update(&data, PROGRAM, |state| {
        state.tick(PROGRAM, "voices", "a1", true)
    })
    .unwrap();

    let again = State::read(&data, PROGRAM).unwrap();
    assert_eq!(again.ticks(PROGRAM, "voices"), ["a1"]);
}
