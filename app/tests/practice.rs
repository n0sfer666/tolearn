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

fn context() -> Context {
    Context::new(&std::env::temp_dir().join(format!("tolearn-practice-{}", std::process::id())))
}

fn bundle(name: &str) -> PathBuf {
    copied(&format!("practice-{name}"))
}

fn run_check(root: &Path, id: &str) -> Result<Value, IpcError> {
    call(
        &context(),
        "run_check",
        &json!({
            "bundle": root.display().to_string(),
            "topic": "local-runtime",
            "check": id,
        }),
    )
}

#[test]
fn прогон_возвращает_команду_код_и_вывод() {
    let root = bundle("run");

    let out = run_check(&root, "c2").unwrap();

    assert_eq!(out["id"], "c2");
    assert!(out["command"].as_str().unwrap().contains("gpu-layers"));
    assert_eq!(out["code"], 0);
    assert!(out["stdout"].as_str().unwrap().contains("NO_COMMANDS_FILE"));
    assert_eq!(out["timed_out"], false);
    assert_eq!(out["truncated"], false);
}

#[test]
fn код_возврата_подсказка_а_не_вердикт() {
    let root = bundle("hint");

    let out = run_check(&root, "c2").unwrap();

    assert!(
        out.get("passed").is_none(),
        "вердикт ставит человек, а не команда: {out:#}"
    );
    assert!(out["code"].is_i64() || out["code"].is_null());
}

#[test]
fn проверка_гоняется_в_папке_бандла() {
    let root = bundle("cwd");
    std::fs::write(root.join("commands.md"), "ollama run --gpu-layers 0\n").unwrap();

    let out = run_check(&root, "c2").unwrap();

    assert!(
        out["stdout"].as_str().unwrap().contains("NGL_ZERO_FOUND"),
        "команда не увидела файл бандла: {out:#}"
    );
}

#[test]
fn неизвестная_проверка_отвергается() {
    let root = bundle("unknown");

    let refused = run_check(&root, "нет-такой-проверки");

    assert!(refused.is_err(), "запуск по неизвестному id");
}

#[test]
fn проверка_ищется_в_своей_теме() {
    let root = bundle("foreign");

    let refused = call(
        &context(),
        "run_check",
        &json!({
            "bundle": root.display().to_string(),
            "topic": "нет-такой-темы",
            "check": "c2",
        }),
    );

    assert!(refused.is_err(), "проверка запущена у неизвестной темы");
}

#[test]
fn проверка_приёмки_гоняется_тем_же_путём() {
    let root = bundle("acceptance");

    let out = run_check(&root, "a1").unwrap();

    assert_eq!(out["id"], "a1");
    assert!(!out["command"].as_str().unwrap().is_empty());
    assert!(out["expect"].is_string());
}
