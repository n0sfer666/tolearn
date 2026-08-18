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
use tolearn_app::ipc::{Context, IpcError, call};

const BOX_MIN: u32 = 75;

fn context() -> Context {
    Context::new(&std::env::temp_dir().join(format!("tolearn-timer-{}", std::process::id())))
}

fn bundle(name: &str) -> PathBuf {
    copied(&format!("timer-{name}"))
}

fn step(root: &Path, step: &str, now: &str) -> Result<Value, IpcError> {
    call(
        &context(),
        "practice",
        &json!({
            "bundle": root.display().to_string(),
            "topic": "local-runtime",
            "step": step,
            "now": now,
        }),
    )
}

fn written(root: &Path) -> String {
    std::fs::read_to_string(root.join("progress.yaml")).unwrap()
}

#[test]
fn отсчёт_идёт_от_таймбокса_темы() {
    let root = bundle("box");

    let out = step(&root, "peek", "2026-07-28T10:00:00+03:00").unwrap();

    assert_eq!(out["box_min"], BOX_MIN);
    assert_eq!(out["left_sec"], i64::from(BOX_MIN) * 60);
    assert_eq!(out["running"], false);
}

#[test]
fn запуск_записан_в_прогресс_и_переживает_перечитывание() {
    let root = bundle("survives");

    step(&root, "start", "2026-07-28T10:00:00+03:00").unwrap();
    let later = step(&root, "peek", "2026-07-28T10:20:00+03:00").unwrap();

    assert!(written(&root).contains("practice"), "{}", written(&root));
    assert_eq!(later["spent_sec"], 1200);
    assert_eq!(later["running"], true);
}

#[test]
fn пауза_и_продолжение_копят_время() {
    let root = bundle("pause");

    step(&root, "start", "2026-07-28T10:00:00+03:00").unwrap();
    let paused = step(&root, "pause", "2026-07-28T10:10:00+03:00").unwrap();
    step(&root, "start", "2026-07-28T11:00:00+03:00").unwrap();
    let again = step(&root, "peek", "2026-07-28T11:05:00+03:00").unwrap();

    assert_eq!(paused["spent_sec"], 600);
    assert_eq!(paused["running"], false);
    assert_eq!(again["spent_sec"], 900);
}

#[test]
fn сброс_возвращает_полный_таймбокс() {
    let root = bundle("reset");

    step(&root, "start", "2026-07-28T10:00:00+03:00").unwrap();
    let out = step(&root, "reset", "2026-07-28T10:30:00+03:00").unwrap();

    assert_eq!(out["spent_sec"], 0);
    assert_eq!(out["left_sec"], i64::from(BOX_MIN) * 60);
    assert_eq!(out["expired"], false);
}

#[test]
fn истечение_записано_и_ничего_не_блокирует() {
    let root = bundle("expired");

    step(&root, "start", "2026-07-28T10:00:00+03:00").unwrap();
    let over = step(&root, "peek", "2026-07-28T12:00:00+03:00").unwrap();
    let after = step(&root, "pause", "2026-07-28T12:00:00+03:00").unwrap();

    assert_eq!(over["expired"], true);
    assert_eq!(over["running"], true);
    assert!(over["left_sec"].as_i64().unwrap() < 0);
    assert_eq!(after["expired"], true);
    assert_eq!(after["spent_sec"], 7200);
}

#[test]
fn взгляд_на_таймер_в_файл_не_пишет() {
    let root = bundle("peek");
    let before = written(&root);

    step(&root, "peek", "2026-07-28T10:00:00+03:00").unwrap();

    assert_eq!(written(&root), before);
}

#[test]
fn таймер_пишет_только_в_прогресс() {
    let root = bundle("only-progress");
    let topic = std::fs::read_to_string(root.join("topics/local-runtime.yaml")).unwrap();

    step(&root, "start", "2026-07-28T10:00:00+03:00").unwrap();

    assert_eq!(
        std::fs::read_to_string(root.join("topics/local-runtime.yaml")).unwrap(),
        topic
    );
}

#[test]
fn неизвестный_шаг_отвергается() {
    let root = bundle("step");

    let refused = step(&root, "перемотать", "2026-07-28T10:00:00+03:00");

    assert!(refused.is_err(), "принят шаг, которого нет");
}

#[test]
fn неизвестная_тема_отвергается() {
    let root = bundle("topic");

    let refused = call(
        &context(),
        "practice",
        &json!({
            "bundle": root.display().to_string(),
            "topic": "нет-такой-темы",
            "step": "start",
            "now": "2026-07-28T10:00:00+03:00",
        }),
    );

    assert!(refused.is_err(), "таймер запущен у неизвестной темы");
}
