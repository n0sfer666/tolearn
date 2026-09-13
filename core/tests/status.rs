#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "status gate: a panic here is the report"
)]

mod support;

use support::states::{PROGRAM, file, scratch};
use tolearn_core::state::{Pass, Passed, State, Status, Summary, key};

const LEAF: &str = "e4c90b7a-15f2-4d6e-8b38-0a2c6f9d1e47";

fn pass(state: &mut State, node: &str, stage: &str, by: Pass) {
    state.open(node, stage, "2026-09-01");
    state.stages.get_mut(&key(node, stage)).unwrap().passed = Some(Passed {
        on: "2026-09-02".to_owned(),
        by,
    });
}

#[test]
fn этап_без_записи_не_начат() {
    let state = State::new(PROGRAM);

    assert_eq!(state.status(PROGRAM, "voices"), Status::Fresh);
}

#[test]
fn первое_открытие_пишет_дату_а_повторное_её_не_меняет() {
    let mut state = State::new(PROGRAM);

    assert!(state.open(PROGRAM, "voices", "2026-09-10"));
    assert!(!state.open(PROGRAM, "voices", "2026-09-13"));

    assert_eq!(state.status(PROGRAM, "voices"), Status::Opened);
    assert_eq!(
        state.stages[&key(PROGRAM, "voices")].opened.as_deref(),
        Some("2026-09-10")
    );
}

#[test]
fn пройденный_этап_помнит_способ() {
    let mut state = State::new(PROGRAM);

    pass(&mut state, PROGRAM, "voices", Pass::Exam);
    pass(&mut state, PROGRAM, "envelope", Pass::Skip);

    assert_eq!(state.status(PROGRAM, "voices"), Status::Passed(Pass::Exam));
    assert_eq!(
        state.status(PROGRAM, "envelope"),
        Status::Passed(Pass::Skip)
    );
}

#[test]
fn одинаковый_id_в_двух_узлах_живёт_отдельно() {
    let mut state = State::new(PROGRAM);

    state.open(LEAF, "setup", "2026-09-10");

    assert_eq!(state.status(LEAF, "setup"), Status::Opened);
    assert_eq!(state.status(PROGRAM, "setup"), Status::Fresh);
}

#[test]
fn сводка_считает_переданные_этапы_и_не_видит_сирот() {
    let mut state = State::new(PROGRAM);
    pass(&mut state, PROGRAM, "voices", Pass::Exam);
    pass(&mut state, PROGRAM, "envelope", Pass::Skip);
    state.open(LEAF, "setup", "2026-09-10");
    pass(&mut state, PROGRAM, "gone", Pass::Exam);

    let summary = state.summary([
        (PROGRAM, "voices"),
        (PROGRAM, "envelope"),
        (LEAF, "setup"),
        (LEAF, "linker"),
    ]);

    assert_eq!(
        summary,
        Summary {
            passed: 2,
            total: 4,
            skipped: 1,
        }
    );
}

#[test]
fn правка_без_изменений_не_создаёт_файл() {
    let data = scratch("untouched");

    State::update(&data, PROGRAM, |_| ()).unwrap();

    assert!(!data.join("state").exists(), "a no-op created the state");
}

#[cfg(unix)]
#[test]
fn повторное_открытие_не_переписывает_файл() {
    use std::os::unix::fs::PermissionsExt;

    let data = scratch("reopen");
    let open = |state: &mut State| state.open(PROGRAM, "voices", "2026-09-10");
    assert!(State::update(&data, PROGRAM, open).unwrap());
    let directory = file(&data).parent().unwrap().to_path_buf();
    let before = std::fs::read(file(&data)).unwrap();
    std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(0o555)).unwrap();

    let again = State::update(&data, PROGRAM, open);

    std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(0o755)).unwrap();
    assert!(!again.unwrap(), "the second open reported a change");
    assert_eq!(std::fs::read(file(&data)).unwrap(), before);
}
