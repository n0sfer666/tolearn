#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "verdict gate: a panic here is the report"
)]

mod support;

use support::states::PROGRAM;
use tolearn_core::stage::{Stage, parse as stage};
use tolearn_core::state::{Attempt, Sitting, State, parse, render};
use tolearn_core::verdict::read;

fn voices() -> Stage {
    stage(&support::read("examples/chiptune/stages/voices.yaml")).unwrap()
}

#[test]
fn принятый_вердикт_переживает_запись_и_чтение_state() {
    let text = r#"{"stage": "voices", "per_question": [
        {"id": "q1", "result": "partial", "missed": [" отступ", "хвост ", "a\n", "строка\r\nвторая"], "added": " x "},
        {"id": "q2", "result": "ok", "missed": [], "added": "  "},
        {"id": "q3", "result": "miss", "missed": ["{скобки}: и #решётка"]},
        {"id": "q4", "result": "ok"}]}"#;
    let per_question = read(text, &voices()).unwrap();
    let mut state = State::new(PROGRAM);
    state.attempt(
        "chiptune",
        "voices",
        Attempt {
            on: "2026-10-05".to_owned(),
            by: Sitting::Copypaste,
            model: None,
            per_question: per_question.clone(),
        },
    );

    let back = parse(&render(&state).unwrap()).unwrap();

    assert_eq!(back, state);
}
