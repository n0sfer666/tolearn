#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "app gate: a panic here is the report"
)]

use serde_json::{Value, json};
use tolearn_app::ipc::{Context, IpcError, call};

struct Case {
    context: Context,
}

fn case(name: &str) -> Case {
    let data = std::env::temp_dir().join(format!(
        "tolearn-settings-ipc-{name}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&data);
    std::fs::create_dir_all(&data).unwrap();
    Case {
        context: Context::new(&data),
    }
}

fn read(case: &Case) -> Value {
    call(&case.context, "settings", &json!({ "save": Value::Null })).unwrap()
}

fn save(case: &Case, extra: Value) -> Result<Value, IpcError> {
    let mut asked = read(case);
    let object = asked.as_object_mut().unwrap();
    for (key, value) in extra.as_object().unwrap() {
        object.insert(key.clone(), value.clone());
    }
    call(&case.context, "settings", &json!({ "save": asked }))
}

#[test]
fn без_файла_приходят_значения_по_умолчанию() {
    let case = case("default");

    let out = read(&case);

    assert_eq!(out["locale"], "ru");
    assert_eq!(out["theme"], "system");
    assert_eq!(out["disk_budget_mb"], 2048);
}

#[test]
fn сохранённые_настройки_читаются_обратно() {
    let case = case("round");

    let saved = save(
        &case,
        json!({ "disk_budget_mb": 512, "locale": "en", "theme": "dark" }),
    )
    .unwrap();

    assert_eq!(saved, read(&case));
    assert_eq!(saved["disk_budget_mb"], 512);
    assert_eq!(saved["locale"], "en");
    assert_eq!(saved["theme"], "dark");
}

#[test]
fn неизвестная_тема_отвергается() {
    let case = case("theme");

    let failure = save(&case, json!({ "theme": "неон" })).unwrap_err();

    assert_eq!(failure.code, "settings.unknown-value");
    assert!(failure.message.contains("неон"), "{failure}");
    assert_eq!(read(&case)["theme"], "system");
}

#[test]
fn неизвестный_язык_отвергается() {
    let case = case("locale");

    let failure = save(&case, json!({ "locale": "fr" })).unwrap_err();

    assert_eq!(failure.code, "settings.unknown-value");
    assert_eq!(read(&case)["locale"], "ru");
}

#[test]
fn нулевой_бюджет_отвергается() {
    let case = case("budget");

    let failure = save(&case, json!({ "disk_budget_mb": 0 })).unwrap_err();

    assert_eq!(failure.code, "settings.unknown-value");
    assert_eq!(read(&case)["disk_budget_mb"], 2048);
}
