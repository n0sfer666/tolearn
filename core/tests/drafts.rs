#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "drafts gate: a panic here is the report"
)]

mod support;

use std::collections::BTreeMap;

use support::states::{PROGRAM, file, scratch};
use tolearn_core::state::State;

fn kept(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|&(id, text)| (id.to_owned(), text.to_owned()))
        .collect()
}

#[test]
fn черновик_ответа_переживает_закрытие_окна() {
    let data = scratch("drafts-kept");
    let text = "Пять каналов:\n- два пульса\n- треугольник: «тише», # не комментарий";
    State::update(&data, PROGRAM, |state| {
        state.draft(PROGRAM, "voices", "q1", text);
        state.draft(PROGRAM, "voices", "q2", "шум");
    })
    .unwrap();

    let read = State::read(&data, PROGRAM).unwrap();
    assert_eq!(
        read.drafts(PROGRAM, "voices"),
        kept(&[("q1", text), ("q2", "шум")])
    );
    assert!(read.drafts(PROGRAM, "tracker").is_empty());
}

#[test]
fn стёртый_ответ_убирает_черновик_и_не_заводит_этап() {
    let data = scratch("drafts-cleared");
    State::update(&data, PROGRAM, |state| {
        state.draft(PROGRAM, "voices", "q1", "  \n");
    })
    .unwrap();
    assert!(!file(&data).exists(), "пустой черновик записал состояние");

    State::update(&data, PROGRAM, |state| {
        state.draft(PROGRAM, "voices", "q1", "пять");
        state.draft(PROGRAM, "voices", "q2", "шум");
    })
    .unwrap();
    State::update(&data, PROGRAM, |state| {
        state.draft(PROGRAM, "voices", "q1", "");
    })
    .unwrap();

    let read = State::read(&data, PROGRAM).unwrap();
    assert_eq!(read.drafts(PROGRAM, "voices"), kept(&[("q2", "шум")]));
}
