#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "app gate: a panic here is the report"
)]

mod support;

use std::path::PathBuf;

use serde_json::{Value, json};
use support::copied;
use tolearn_app::ipc::{Context, IpcError, call};

struct Case {
    context: Context,
    data: PathBuf,
    bundle: String,
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
        data,
        bundle: copied(&format!("settings-{name}")).display().to_string(),
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

fn note(case: &Case, name: &str, extra: Value) -> Result<Value, IpcError> {
    let mut payload = json!({
        "bundle": case.bundle,
        "topic": "local-runtime",
        "directory": Value::Null,
    });
    let object = payload.as_object_mut().unwrap();
    for (key, value) in extra.as_object().unwrap() {
        object.insert(key.clone(), value.clone());
    }
    call(&case.context, name, &payload)
}

#[test]
fn без_файла_приходят_значения_по_умолчанию() {
    let case = case("default");

    let out = read(&case);

    assert_eq!(out["locale"], "ru");
    assert_eq!(out["theme"], "system");
    assert_eq!(out["notes_directory"], Value::Null);
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
fn каталог_конспектов_действует_без_перезапуска() {
    let case = case("notes");
    let outside = case.data.join("снаружи");

    save(
        &case,
        json!({ "notes_directory": outside.display().to_string() }),
    )
    .unwrap();
    note(
        &case,
        "save_note",
        json!({ "body": "Внешний текст", "stamp": Value::Null }),
    )
    .unwrap();

    let out = note(&case, "note", json!({})).unwrap();
    assert_eq!(out["body"].as_str().unwrap().trim(), "Внешний текст");
    assert!(
        out["path"]
            .as_str()
            .unwrap()
            .starts_with(&outside.display().to_string()),
        "{out}"
    );
    assert!(
        !case.data.join("notes").exists(),
        "конспект лёг во внутренний каталог"
    );
}

#[test]
fn каталог_из_запроса_сильнее_настроек() {
    let case = case("explicit");
    let from_settings = case.data.join("из-настроек");
    let from_request = case.data.join("из-запроса");
    save(
        &case,
        json!({ "notes_directory": from_settings.display().to_string() }),
    )
    .unwrap();

    let out = note(
        &case,
        "save_note",
        json!({
            "body": "Текст",
            "directory": from_request.display().to_string(),
            "stamp": Value::Null,
        }),
    )
    .unwrap();

    assert_eq!(out["saved"], true);
    assert!(from_request.exists(), "каталог из запроса не создан");
    assert!(!from_settings.exists(), "запись ушла в каталог из настроек");
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
