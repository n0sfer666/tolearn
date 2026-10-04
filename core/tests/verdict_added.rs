#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "verdict gate: a panic here is the report"
)]

mod support;

use tolearn_core::stage::{Stage, parse};
use tolearn_core::verdict::{VerdictError, read};

fn voices() -> Stage {
    parse(&support::read("examples/chiptune/stages/voices.yaml")).unwrap()
}

fn graded(first: &str) -> String {
    format!(
        r#"{{"stage": "voices", "per_question": [{first}, {{"id": "q2", "result": "ok"}},
        {{"id": "q3", "result": "ok"}}, {{"id": "q4", "result": "ok"}}]}}"#
    )
}

fn added(first: &str) -> Option<String> {
    read(&graded(first), &voices()).unwrap()[0].added.clone()
}

#[test]
fn added_пишется_дословно_у_любого_result() {
    assert_eq!(
        added(r#"{"id": "q1", "result": "ok", "added": "ещё есть {DPCM}"}"#).as_deref(),
        Some("ещё есть {DPCM}")
    );
    assert_eq!(
        added(r#"{"id": "q1", "result": "partial", "missed": ["шум"], "added": "шум — пятый"}"#)
            .as_deref(),
        Some("шум — пятый")
    );
    assert_eq!(
        added(r#"{"id": "q1", "result": "miss", "missed": ["всё"], "added": "пять каналов"}"#)
            .as_deref(),
        Some("пять каналов")
    );
}

#[test]
fn null_пустая_строка_и_отсутствие_значат_что_дополнения_нет() {
    for empty in [
        r#"{"id": "q1", "result": "ok"}"#,
        r#"{"id": "q1", "result": "ok", "added": null}"#,
        r#"{"id": "q1", "result": "ok", "added": ""}"#,
        r#"{"id": "q1", "result": "ok", "added": " "}"#,
        r#"{"id": "q1", "result": "ok", "added": "\n"}"#,
    ] {
        assert_eq!(added(empty), None, "{empty}");
    }
}

#[test]
fn added_не_строкой_отказывает_с_id_вопроса() {
    for broken in [
        r#"{"id": "q1", "result": "ok", "added": 3}"#,
        r#"{"id": "q1", "result": "ok", "added": ["шум"]}"#,
        r#"{"id": "q1", "result": "ok", "added": {"text": "шум"}}"#,
    ] {
        match read(&graded(broken), &voices()) {
            Err(VerdictError::Shape(reason)) => {
                assert!(
                    reason.contains("q1") && reason.contains("added"),
                    "{reason}"
                );
            }
            other => panic!("{broken}: {other:?}"),
        }
    }
}
