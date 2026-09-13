#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "skip gate: a panic here is the report"
)]

mod support;

use support::states::{PROGRAM, file, scratch};
use tolearn_core::state::{
    Answered, Attempt, Grade, Pass, Passed, Sitting, StageState, State, Status, key,
};

fn sat(on: &str, result: Grade) -> Attempt {
    Attempt {
        on: on.to_owned(),
        by: Sitting::Copypaste,
        model: None,
        per_question: vec![Answered {
            id: "q1".to_owned(),
            result,
            missed: Vec::new(),
        }],
    }
}

fn entry(state: &State) -> &StageState {
    state.stages.get(&key(PROGRAM, "voices")).unwrap()
}

fn passed(on: &str, by: Pass) -> Option<Passed> {
    Some(Passed {
        on: on.to_owned(),
        by,
    })
}

#[test]
fn пропуск_отмечает_этап_пройденным_и_не_пишет_попытку() {
    let mut state = State::new(PROGRAM);

    assert!(state.skip(PROGRAM, "voices", "2026-09-13"));

    assert_eq!(state.status(PROGRAM, "voices"), Status::Passed(Pass::Skip));
    assert_eq!(entry(&state).passed, passed("2026-09-13", Pass::Skip));
    assert_eq!(entry(&state).opened.as_deref(), Some("2026-09-13"));
    assert!(entry(&state).attempts.is_empty());
    assert!(!state.skip(PROGRAM, "voices", "2026-09-14"));
    assert_eq!(entry(&state).passed, passed("2026-09-13", Pass::Skip));
}

#[test]
fn пропуск_не_снимает_сданный_зачёт() {
    let mut state = State::new(PROGRAM);
    state.attempt(PROGRAM, "voices", sat("2026-09-12", Grade::Ok));

    assert!(!state.skip(PROGRAM, "voices", "2026-09-13"));

    assert_eq!(entry(&state).passed, passed("2026-09-12", Pass::Exam));
}

#[test]
fn после_пропуска_проваленный_зачёт_пишет_попытку_а_сданный_меняет_пометку() {
    let mut state = State::new(PROGRAM);
    state.skip(PROGRAM, "voices", "2026-09-10");

    state.attempt(PROGRAM, "voices", sat("2026-09-11", Grade::Miss));
    assert_eq!(state.status(PROGRAM, "voices"), Status::Passed(Pass::Skip));
    assert_eq!(entry(&state).attempts.len(), 1);

    state.attempt(PROGRAM, "voices", sat("2026-09-12", Grade::Ok));
    assert_eq!(entry(&state).passed, passed("2026-09-12", Pass::Exam));
}

#[test]
fn пропуск_ложится_в_файл_без_попыток() {
    let data = scratch("skip");

    State::update(&data, PROGRAM, |state| {
        state.skip(PROGRAM, "voices", "2026-09-13")
    })
    .unwrap();

    let text = std::fs::read_to_string(file(&data)).unwrap();
    assert!(text.contains("by: skip"), "{text}");
    assert!(!text.contains("attempts"), "{text}");
    let read = State::read(&data, PROGRAM).unwrap();
    assert_eq!(read.status(PROGRAM, "voices"), Status::Passed(Pass::Skip));
}
