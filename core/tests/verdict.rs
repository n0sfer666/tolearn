#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "verdict gate: a panic here is the report"
)]

mod support;

use tolearn_core::stage::{Stage, parse};
use tolearn_core::state::{Answered, Grade};
use tolearn_core::verdict::{VerdictError, read};

fn voices() -> Stage {
    parse(&support::read("examples/chiptune/stages/voices.yaml")).unwrap()
}

fn graded(rows: &str) -> String {
    format!(r#"{{"stage": "voices", "per_question": [{rows}]}}"#)
}

const ALL_OK: &str = r#"{"id": "q1", "result": "ok"}, {"id": "q2", "result": "ok"},
    {"id": "q3", "result": "ok"}, {"id": "q4", "result": "ok"}"#;

const MIXED: &str = r#"{"id": "q1", "result": "partial", "missed": ["нет шумового канала", "{скобки}"], "quote": "пять"},
    {"id": "q2", "result": "ok", "missed": []},
    {"id": "q3", "result": "miss", "missed": ["формула таймера"]},
    {"id": "q4", "result": "ok"}"#;

fn refused(text: &str) -> VerdictError {
    match read(text, &voices()) {
        Ok(rows) => panic!("verdict accepted: {rows:?}"),
        Err(error) => error,
    }
}

fn row(id: &str, result: Grade, missed: &[&str]) -> Answered {
    Answered {
        id: id.to_owned(),
        result,
        missed: missed.iter().map(|&line| line.to_owned()).collect(),
    }
}

#[test]
fn берётся_последний_json_блок_со_всем_текстом_вокруг() {
    let text = format!(
        "Разбор ответов {{по порядку}}.\n\n```json\n{}\n```\n\nИтог:\n\n```json\n{}\n```\nУдачи!",
        graded(r#"{"id": "q1", "result": "miss", "missed": ["старое"]}"#),
        graded(MIXED),
    );

    let rows = read(&text, &voices()).unwrap();

    assert_eq!(
        rows,
        [
            row("q1", Grade::Partial, &["нет шумового канала", "{скобки}"]),
            row("q2", Grade::Ok, &[]),
            row("q3", Grade::Miss, &["формула таймера"]),
            row("q4", Grade::Ok, &[]),
        ]
    );
}

#[test]
fn блок_без_ограды_и_вложенные_объекты_не_путают_разбор() {
    let text = format!(
        r#"Вот вердикт: {{"meta": {{"stage": "envelope"}}, "stage": "voices", "per_question": [{ALL_OK}]}} — конец."#
    );

    let rows = read(&text, &voices()).unwrap();

    assert_eq!(rows.len(), 4);
    assert!(rows.iter().all(|one| one.result == Grade::Ok));
}

#[test]
fn вердикт_чужого_этапа_отказывает_как_вставленный_не_туда() {
    let text = format!(r#"{{"stage": "envelope", "per_question": [{ALL_OK}]}}"#);

    let error = refused(&text);

    assert!(
        matches!(&error, VerdictError::Stage { expected, found } if expected == "voices" && found == "envelope"),
        "{error:?}"
    );
    assert!(error.to_string().contains("не в тот этап"), "{error}");
}

#[test]
fn без_json_без_этапа_или_без_per_question_вердикта_нет() {
    assert!(matches!(
        refused("Молодец, всё верно!"),
        VerdictError::Absent
    ));
    assert!(matches!(refused("{ не json }"), VerdictError::Absent));
    assert!(matches!(
        refused(&format!(r#"{{"per_question": [{ALL_OK}]}}"#)),
        VerdictError::Shape(_)
    ));
    assert!(matches!(
        refused(r#"{"stage": "voices", "verdict": "pass"}"#),
        VerdictError::Shape(_)
    ));
    assert!(matches!(
        refused(r#"{"stage": "voices", "per_question": {"q1": "ok"}}"#),
        VerdictError::Shape(_)
    ));
    assert!(matches!(
        refused(r#"{"stage": "voices", "per_question": []}"#),
        VerdictError::Questions { .. }
    ));
}

#[test]
fn per_question_покрывает_ровно_вопросы_этапа() {
    let short = refused(&graded(
        r#"{"id": "q1", "result": "ok"}, {"id": "q2", "result": "ok"}, {"id": "q3", "result": "ok"}"#,
    ));
    let extra = refused(&graded(&format!(
        r#"{ALL_OK}, {{"id": "q9", "result": "ok"}}"#
    )));
    let twice = refused(&graded(&format!(
        r#"{ALL_OK}, {{"id": "q2", "result": "ok"}}"#
    )));

    assert!(
        matches!(&short, VerdictError::Questions { missing, stray, repeated } if missing == &["q4"] && stray.is_empty() && repeated.is_empty()),
        "{short:?}"
    );
    assert!(
        matches!(&extra, VerdictError::Questions { stray, .. } if stray == &["q9"]),
        "{extra:?}"
    );
    assert!(
        matches!(&twice, VerdictError::Questions { repeated, .. } if repeated == &["q2"]),
        "{twice:?}"
    );
    assert!(short.to_string().contains("q4"), "{short}");
}

#[test]
fn result_из_трёх_значений_а_у_незачтённого_есть_missed() {
    let rows = |first: &str| {
        graded(&format!(
            r#"{first}, {{"id": "q2", "result": "ok"}}, {{"id": "q3", "result": "ok"}}, {{"id": "q4", "result": "ok"}}"#
        ))
    };

    for broken in [
        r#"{"id": "q1", "result": "great"}"#,
        r#"{"id": "q1", "result": "OK"}"#,
        r#"{"id": "q1"}"#,
        r#"{"result": "ok"}"#,
        r#"{"id": "q1", "result": "partial"}"#,
        r#"{"id": "q1", "result": "miss", "missed": []}"#,
        r#"{"id": "q1", "result": "miss", "missed": "всё"}"#,
        r#"{"id": "q1", "result": "miss", "missed": [""]}"#,
        r#"{"id": "q1", "result": "ok", "missed": [3]}"#,
        r#""q1""#,
    ] {
        let error = refused(&rows(broken));
        assert!(
            matches!(error, VerdictError::Shape(_)),
            "{broken}: {error:?}"
        );
    }
}
