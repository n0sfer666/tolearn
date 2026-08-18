#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "link gate: a panic here is the report"
)]

use tolearn_core::link::{LinkError, parse};

#[test]
fn ссылка_на_тему_разбирается() {
    let link = parse("tolearn://topic?roadmap=llm-agents-base&topic=local-runtime").unwrap();

    assert_eq!(link.roadmap, "llm-agents-base");
    assert_eq!(link.topic, "local-runtime");
}

#[test]
fn порядок_параметров_значения_не_имеет() {
    let link = parse("tolearn://topic?topic=local-runtime&roadmap=llm-agents-base").unwrap();

    assert_eq!(link.roadmap, "llm-agents-base");
    assert_eq!(link.topic, "local-runtime");
}

#[test]
fn процентные_последовательности_раскрываются() {
    let link = parse("tolearn://topic?roadmap=llm%2Dagents%2Dbase&topic=local%2Druntime").unwrap();

    assert_eq!(link.roadmap, "llm-agents-base");
    assert_eq!(link.topic, "local-runtime");
}

#[test]
fn чужая_схема_отклоняется() {
    let failed = parse("obsidian://topic?roadmap=llm&topic=local").unwrap_err();

    assert!(matches!(failed, LinkError::Scheme { .. }), "{failed}");
}

#[test]
fn неизвестное_действие_отклоняется() {
    let failed = parse("tolearn://delete?roadmap=llm&topic=local").unwrap_err();

    assert!(matches!(failed, LinkError::Action { .. }), "{failed}");
}

#[test]
fn пропущенный_параметр_отклоняется() {
    let failed = parse("tolearn://topic?roadmap=llm").unwrap_err();

    assert!(matches!(failed, LinkError::Missing { .. }), "{failed}");
}

#[test]
fn пустое_значение_отклоняется() {
    let failed = parse("tolearn://topic?roadmap=llm&topic=").unwrap_err();

    assert!(matches!(failed, LinkError::Missing { .. }), "{failed}");
}

#[test]
fn посторонний_параметр_отклоняется() {
    let failed = parse("tolearn://topic?roadmap=llm&topic=local&run=rm").unwrap_err();

    assert!(matches!(failed, LinkError::Extra { .. }), "{failed}");
}

#[test]
fn повторённый_параметр_отклоняется() {
    let failed = parse("tolearn://topic?roadmap=llm&roadmap=other&topic=local").unwrap_err();

    assert!(matches!(failed, LinkError::Extra { .. }), "{failed}");
}

#[test]
fn путь_в_значении_отклоняется() {
    for value in [
        "tolearn://topic?roadmap=../../etc&topic=local",
        "tolearn://topic?roadmap=llm&topic=%2E%2E%2Fpasswd",
        "tolearn://topic?roadmap=/absolute&topic=local",
        "tolearn://topic?roadmap=llm&topic=a%00b",
    ] {
        let failed = parse(value).unwrap_err();
        assert!(
            matches!(failed, LinkError::Value { .. }),
            "{value}: {failed}"
        );
    }
}

#[test]
fn значение_из_одних_точек_отклоняется() {
    for value in [
        "tolearn://topic?roadmap=..&topic=local",
        "tolearn://topic?roadmap=llm&topic=.hidden",
    ] {
        let failed = parse(value).unwrap_err();
        assert!(
            matches!(failed, LinkError::Value { .. }),
            "{value}: {failed}"
        );
    }
}

#[test]
fn ссылка_без_параметров_отклоняется() {
    let failed = parse("tolearn://topic").unwrap_err();

    assert!(matches!(failed, LinkError::Missing { .. }), "{failed}");
}
