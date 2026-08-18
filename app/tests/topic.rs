#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "app gate: a panic here is the report"
)]

mod support;

use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use support::copied;
use tolearn_app::ipc::{Context, call};

fn context() -> Context {
    Context::new(&std::env::temp_dir().join(format!("tolearn-topic-{}", std::process::id())))
}

fn topic(root: &Path, id: &str) -> Value {
    call(
        &context(),
        "topic",
        &json!({ "bundle": root.display().to_string(), "topic": id, "today": "2026-07-27" }),
    )
    .unwrap()
}

fn bundle(name: &str) -> PathBuf {
    copied(&format!("topic-{name}"))
}

#[test]
fn заголовок_несёт_статус_часы_зависимости_и_свежесть() {
    let out = topic(&bundle("header"), "cp-gateway");

    assert_eq!(
        out["title"],
        "Чекпоинт — единый шлюз к трём моделям с замером и фолбэком"
    );
    assert_eq!(out["status"], "blocked");
    assert_eq!(out["hours"]["min"], 2);
    assert!(!out["verified_at"].as_str().unwrap().is_empty());
    assert!(out["outdated"].is_boolean());
    assert!(
        !out["blocked_by"].as_array().unwrap().is_empty(),
        "чекпойнт ждёт свои темы: {out:#}"
    );
}

#[test]
fn ответов_нет_в_ответе_команды() {
    let out = topic(&bundle("hidden"), "local-runtime");
    let text = out.to_string();

    let questions = out["questions"].as_array().unwrap();
    assert!(!questions.is_empty(), "вопросы пропали целиком");
    for question in questions {
        assert!(!question["text"].as_str().unwrap().is_empty());
        assert!(question.get("expected_signals").is_none(), "{question:#}");
        assert!(question.get("red_flags").is_none(), "{question:#}");
        assert!(question.get("follow_up").is_none(), "{question:#}");
    }
    assert!(
        out["exam"].get("traps").is_none(),
        "ловушки уехали на экран"
    );
    assert!(!text.contains("expected_signals"));
    assert!(!text.contains("red_flags"));
}

#[test]
fn вырожденный_чекпойнт_отдаёт_пустые_коллекции_а_не_заглушки() {
    let out = topic(&bundle("checkpoint"), "cp-gateway");

    assert!(out["misconceptions"].as_array().unwrap().is_empty());
    assert!(out["materials"].as_array().unwrap().is_empty());
    assert!(out["questions"].as_array().unwrap().is_empty());
    assert!(!out["outcomes"].as_array().unwrap().is_empty());
    assert!(!out["practice"]["task"].as_str().unwrap().is_empty());
}

#[test]
fn ограничения_и_приёмка_идут_двумя_коллекциями() {
    let out = topic(&bundle("practice"), "local-runtime");
    let practice = &out["practice"];

    let constraints = practice["constraints"].as_array().unwrap();
    let acceptance = practice["acceptance"].as_array().unwrap();
    assert!(!constraints.is_empty());
    assert!(!acceptance.is_empty());
    assert_ne!(constraints, acceptance, "коллекции слиты в одну");
    assert!(!constraints[0]["check"].as_str().unwrap().is_empty());
    assert!(practice["smoke_checked"].is_boolean());
}

#[test]
fn материал_несёт_свежесть_и_офлайн_доступность() {
    let out = topic(&bundle("materials"), "local-runtime");
    let material = &out["materials"].as_array().unwrap()[0];

    assert!(!material["title"].as_str().unwrap().is_empty());
    assert!(material["stale"].is_boolean());
    assert_eq!(material["offline"], "absent");
}

#[test]
fn свежесть_считается_от_verified_at_и_срока_ревалидации() {
    let root = bundle("freshness");
    let today = |day: &str| {
        call(
            &context(),
            "topic",
            &json!({ "bundle": root.display().to_string(), "topic": "local-runtime", "today": day }),
        )
        .unwrap()
    };

    assert_eq!(today("2026-07-27")["outdated"], false);
    assert_eq!(today("2026-10-23")["outdated"], false);
    assert_eq!(today("2026-10-24")["outdated"], true);
    assert_eq!(today("2030-01-01")["outdated"], true);
}

#[test]
fn статус_ставится_вручную_и_виден_при_следующем_чтении() {
    let root = bundle("manual-set");
    let set = call(
        &context(),
        "set_status",
        &json!({
            "bundle": root.display().to_string(),
            "topic": "local-runtime",
            "status": "passed",
            "today": "2026-07-27",
        }),
    )
    .unwrap();

    assert_eq!(set["status"], "passed");
    assert_eq!(topic(&root, "local-runtime")["status"], "passed");
}

#[test]
fn ручная_отметка_записана_как_manual_а_не_как_зачёт() {
    let root = bundle("manual-source");
    call(
        &context(),
        "set_status",
        &json!({
            "bundle": root.display().to_string(),
            "topic": "local-runtime",
            "status": "passed",
            "today": "2026-07-27",
        }),
    )
    .unwrap();

    let written = std::fs::read_to_string(root.join("progress.yaml")).unwrap();
    assert!(written.contains("source: manual"), "{written}");
}

#[test]
fn неизвестный_статус_отвергается_а_не_пишется_в_файл() {
    let root = bundle("refused");
    let before = std::fs::read_to_string(root.join("progress.yaml")).unwrap();
    let refused = call(
        &context(),
        "set_status",
        &json!({
            "bundle": root.display().to_string(),
            "topic": "local-runtime",
            "status": "почти_пройдена",
            "today": "2026-07-27",
        }),
    );

    assert!(refused.is_err(), "неизвестный статус принят");
    assert_eq!(
        std::fs::read_to_string(root.join("progress.yaml")).unwrap(),
        before
    );
}
