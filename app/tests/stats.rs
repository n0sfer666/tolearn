#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "app gate: a panic here is the report"
)]

mod support;

use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use support::{copied, repository};
use tolearn_app::ipc::{Context, call};

fn context() -> Context {
    let directory = std::env::temp_dir().join(format!("tolearn-stats-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    Context::new(&directory)
}

fn sampled(name: &str, fixture: &str) -> PathBuf {
    let bundle = copied(name);
    let source = repository().join(format!("fixtures/valid/progress/{fixture}.yaml"));
    std::fs::copy(source, bundle.join("progress.yaml")).unwrap();
    bundle
}

fn taken(bundle: &Path) -> Value {
    call(
        &context(),
        "stats",
        &json!({ "bundle": bundle.display().to_string() }),
    )
    .unwrap()
}

#[test]
fn статистика_считается_по_всем_попыткам_программы() {
    let bundle = sampled("stats-all", "attempts-sample");

    let stats = taken(&bundle);

    assert_eq!(stats["attempts"], json!(7));
    assert_eq!(stats["enough"], json!(true));
    assert_eq!(stats["hinted"], json!(3));
    let share = stats["hinted_share"].as_f64().unwrap();
    assert!((share - 3.0 / 7.0).abs() < 1e-9, "{stats}");
}

#[test]
fn на_малой_выборке_вывода_нет_а_есть_число_попыток() {
    let bundle = sampled("stats-scarce", "attempts-history");

    let stats = taken(&bundle);

    assert_eq!(stats["attempts"], json!(4));
    assert_eq!(stats["enough"], json!(false));
    assert_eq!(stats["kinds"].as_array().unwrap().len(), 0, "{stats}");
    assert_eq!(stats["actions"].as_array().unwrap().len(), 0, "{stats}");
}

#[test]
fn ответы_разложены_по_типам_вопросов() {
    let bundle = sampled("stats-kinds", "attempts-sample");

    let stats = taken(&bundle);

    let kinds = stats["kinds"].as_array().unwrap();
    assert!(!kinds.is_empty(), "{stats}");
    let diagnose = kinds
        .iter()
        .find(|item| item["kind"] == json!("diagnose"))
        .unwrap_or_else(|| panic!("{stats}"));
    assert_eq!(diagnose["ok"], json!(1));
    assert_eq!(diagnose["partial"], json!(1));
    assert_eq!(diagnose["miss"], json!(1));
}

#[test]
fn серия_провалов_названа_вместе_с_темой() {
    let bundle = sampled("stats-streak", "attempts-sample");

    let stats = taken(&bundle);

    assert_eq!(stats["streak"]["longest"], json!(3));
    assert_eq!(stats["streak"]["topic"], json!("structured-output"));
}

#[test]
fn калибровка_уезжает_прозой_а_не_числом() {
    let bundle = sampled("stats-calibration", "attempts-sample");

    let stats = taken(&bundle);

    let lines = stats["calibration"].as_array().unwrap();
    assert_eq!(lines.len(), 3, "{stats}");
    assert!(
        lines
            .iter()
            .all(|line| line.as_str().unwrap().contains(' ')),
        "{stats}"
    );
}
