#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "provider gate: a panic here is the report"
)]

use tolearn_provider::{Harness, drift, fingerprint, preset};

fn saved(id: &str, args: Vec<String>) -> Harness {
    Harness {
        id: id.to_owned(),
        command: "claude".to_owned(),
        args,
        timeout_secs: 900,
        dismissed_advice: None,
    }
}

#[test]
fn совпадающие_аргументы_не_дают_расхождения() {
    let claude = preset("claude").expect("пресет claude зарегистрирован");

    assert_eq!(drift(&saved("claude", claude.advised())), None);
}

#[test]
fn расхождение_показывает_что_уйдёт_и_что_добавится() {
    let claude = preset("claude").expect("пресет claude зарегистрирован");
    let mut args = claude.advised();
    args.retain(|arg| arg != "--strict-mcp-config");
    args.push("--allowedTools".to_owned());

    let found = drift(&saved("claude", args)).expect("аргументы разошлись");

    assert_eq!(found.removed, vec!["--allowedTools".to_owned()]);
    assert_eq!(found.added, vec!["--strict-mcp-config".to_owned()]);
    assert_eq!(found.fingerprint, fingerprint(&claude.advised()));
}

#[test]
fn пустые_аргументы_у_пресета_с_советом_тоже_расхождение() {
    let claude = preset("claude").expect("пресет claude зарегистрирован");

    let found = drift(&saved("claude", Vec::new())).expect("пустые аргументы — расхождение");

    assert!(found.removed.is_empty());
    assert_eq!(found.added, claude.advised());
}

#[test]
fn у_custom_плашки_не_бывает() {
    assert_eq!(drift(&saved("custom", vec!["--anything".to_owned()])), None);
}

#[test]
fn запомненный_отказ_гасит_расхождение_до_смены_совета() {
    let claude = preset("claude").expect("пресет claude зарегистрирован");
    let mut provider = saved("claude", Vec::new());
    provider.dismissed_advice = Some(fingerprint(&claude.advised()));

    assert_eq!(drift(&provider), None);
}

#[test]
fn смена_совета_возвращает_расхождение_после_прежнего_отказа() {
    let claude = preset("claude").expect("пресет claude зарегистрирован");
    let mut provider = saved("claude", Vec::new());
    provider.dismissed_advice = Some(fingerprint(&["--allowedTools".to_owned()]));

    let found = drift(&provider).expect("отпечаток устарел вместе со старым советом");

    assert_eq!(found.added, claude.advised());
}
