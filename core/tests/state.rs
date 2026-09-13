#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "state gate: a panic here is the report"
)]

mod support;

use support::states::{PROGRAM, file, full, placed, scratch};
use tolearn_core::state::{State, StateError};
use tolearn_core::yaml::ParseFailure;

const OTHER: &str = "9b2e4f60-1c3d-4a5b-8e7f-0a1b2c3d4e5f";

#[test]
fn нет_файла_значит_пустое_состояние_и_ничего_не_создано() {
    let data = scratch("absent");

    let state = State::read(&data, PROGRAM).unwrap();

    assert_eq!(state, State::new(PROGRAM));
    assert_eq!(state.program(), PROGRAM);
    assert_eq!(state.workdir, None);
    assert!(state.stages.is_empty() && state.clarifications.is_empty());
    assert!(
        !data.join("state").exists(),
        "reading created the state directory"
    );
}

#[test]
fn записанное_читается_тем_же() {
    let data = scratch("round");
    let expected = full();

    State::update(&data, PROGRAM, |state| *state = expected.clone()).unwrap();

    assert!(file(&data).is_file());
    assert_eq!(State::read(&data, PROGRAM).unwrap(), expected);
}

#[test]
fn правка_получает_прочитанное_и_возвращает_своё() {
    let data = scratch("change");
    State::update(&data, PROGRAM, |state| *state = full()).unwrap();

    let seen = State::update(&data, PROGRAM, |state| {
        state.workdir = None;
        state.stages.len()
    })
    .unwrap();

    assert_eq!(seen, 2);
    let kept = State::read(&data, PROGRAM).unwrap();
    assert_eq!(kept.workdir, None);
    assert_eq!(kept.stages, full().stages);
}

#[test]
fn чужая_схема_отказывает_с_местом_и_файл_не_трогается() {
    let data = scratch("foreign-schema");
    let body =
        format!("schema: tolearn/state/2\nprogram: {PROGRAM}\nstages: {{}}\nclarifications: []\n");
    let path = placed(&data, &body);

    let error = State::update(&data, PROGRAM, |_| {
        panic!("the change ran over a refused file")
    })
    .unwrap_err();

    let StateError::Malformed(parse) = &error else {
        panic!("expected a parse refusal, got {error}");
    };
    assert_eq!(
        (parse.failure(), parse.path(), parse.line()),
        (ParseFailure::UnknownValue, "schema", 1)
    );
    assert_eq!(std::fs::read_to_string(&path).unwrap(), body);
}

#[test]
fn битый_yaml_отказывает_с_местом_и_файл_не_трогается() {
    let data = scratch("broken-yaml");
    let body = format!("schema: tolearn/state/1\nprogram: {PROGRAM}\nstages: [\n");
    let path = placed(&data, &body);

    let error = State::update(&data, PROGRAM, |_| ()).unwrap_err();

    let StateError::Malformed(parse) = &error else {
        panic!("expected a parse refusal, got {error}");
    };
    assert_eq!(parse.failure(), ParseFailure::Syntax);
    assert!(
        parse.line() >= 3,
        "the refusal points at line {}",
        parse.line()
    );
    assert_eq!(std::fs::read_to_string(&path).unwrap(), body);
}

#[test]
fn состояние_другой_программы_отказывает_и_файл_не_трогается() {
    let data = scratch("other-program");
    let body =
        format!("schema: tolearn/state/1\nprogram: {OTHER}\nstages: {{}}\nclarifications: []\n");
    let path = placed(&data, &body);

    let read = State::read(&data, PROGRAM);
    let updated = State::update(&data, PROGRAM, |_| ());

    assert!(matches!(read, Err(StateError::Foreign(found)) if found == OTHER));
    assert!(matches!(updated, Err(StateError::Foreign(_))));
    assert_eq!(std::fs::read_to_string(&path).unwrap(), body);
}

#[test]
fn не_uuid_не_становится_путём() {
    let data = scratch("stray");

    for program in [
        "../programs/3f6c2a1e-8b4d-4c7a-9e21-5d0f7b3a6c84",
        "",
        "3F6C2A1E-8B4D-4C7A-9E21-5D0F7B3A6C84",
    ] {
        let read = State::read(&data, program);
        let updated = State::update(&data, program, |_| ());
        assert!(
            matches!(read, Err(StateError::Stray(_))),
            "read `{program}`"
        );
        assert!(
            matches!(updated, Err(StateError::Stray(_))),
            "update `{program}`"
        );
    }
    assert_eq!(std::fs::read_dir(&data).unwrap().count(), 0);
}

#[test]
fn осиротевшие_этапы_и_врезки_переживают_чтение_и_запись() {
    let data = scratch("orphans");
    placed(&data, &orphans());
    let before = State::read(&data, PROGRAM).unwrap();

    State::update(&data, PROGRAM, |state| {
        state.workdir = Some("/tmp/practice".to_owned());
    })
    .unwrap();

    let after = State::read(&data, PROGRAM).unwrap();
    assert!(after.stages.contains_key("gone"));
    assert_eq!(after.stages, before.stages);
    assert_eq!(after.clarifications, before.clarifications);
    assert_eq!(after.clarifications[0].block, "deadbeef");
}

fn orphans() -> String {
    let lines = [
        "schema: tolearn/state/1".to_owned(),
        format!("program: {PROGRAM}"),
        "stages:".to_owned(),
        "  gone:".to_owned(),
        "    opened: 2026-09-01".to_owned(),
        "    ticks: [a1]".to_owned(),
        "clarifications:".to_owned(),
        "  - stage: voices".to_owned(),
        "    block: deadbeef".to_owned(),
        "    excerpt: Абзац, которого больше нет".to_owned(),
        "    turns:".to_owned(),
        "      - answer: Разъяснение.".to_owned(),
        "    clear: false".to_owned(),
    ];
    lines.join("\n") + "\n"
}
