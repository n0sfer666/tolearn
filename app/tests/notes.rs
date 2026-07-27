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
    let data =
        std::env::temp_dir().join(format!("tolearn-notes-ipc-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&data);
    std::fs::create_dir_all(&data).unwrap();
    Case {
        context: Context::new(&data),
        data,
        bundle: copied(&format!("notes-{name}")).display().to_string(),
    }
}

fn note(case: &Case, extra: Value) -> Result<Value, IpcError> {
    let mut payload = json!({ "bundle": case.bundle, "topic": "local-runtime" });
    merge(&mut payload, extra);
    call(&case.context, "note", &payload)
}

fn save(case: &Case, body: &str, extra: Value) -> Result<Value, IpcError> {
    let mut payload = json!({
        "bundle": case.bundle,
        "topic": "local-runtime",
        "body": body,
    });
    merge(&mut payload, extra);
    call(&case.context, "save_note", &payload)
}

fn merge(payload: &mut Value, extra: Value) {
    let object = payload.as_object_mut().unwrap();
    for (key, value) in extra.as_object().unwrap() {
        object.insert(key.clone(), value.clone());
    }
}

#[test]
fn пустой_конспект_читается_без_ошибки() {
    let case = case("empty");

    let out = note(&case, json!({})).unwrap();

    assert_eq!(out["body"], "");
    assert_eq!(out["stamp"], Value::Null);
    assert_eq!(out["path"], Value::Null);
}

#[test]
fn записанный_конспект_читается_обратно() {
    let case = case("round");

    let saved = save(&case, "Текст конспекта", json!({})).unwrap();
    let out = note(&case, json!({})).unwrap();

    assert_eq!(saved["saved"], true);
    assert_eq!(out["body"].as_str().unwrap().trim(), "Текст конспекта");
    assert_eq!(out["stamp"], saved["stamp"]);
    assert!(
        out["path"]
            .as_str()
            .unwrap()
            .starts_with(&case.data.join("notes").display().to_string()),
        "{out}"
    );
}

#[test]
fn привязка_идёт_по_идентификатору_программы_а_не_по_пути() {
    let case = case("bound");
    save(&case, "Текст конспекта", json!({})).unwrap();

    let moved = Case {
        context: Context::new(&case.data),
        data: case.data.clone(),
        bundle: copied("notes-bound-другой-путь").display().to_string(),
    };

    assert_eq!(
        note(&moved, json!({})).unwrap()["body"]
            .as_str()
            .unwrap()
            .trim(),
        "Текст конспекта"
    );
}

#[test]
fn внешний_каталог_берётся_вместо_своего() {
    let case = case("external");
    let outside = case.data.join("снаружи");
    let directory = json!({ "directory": outside.display().to_string() });

    save(&case, "Внешний текст", directory.clone()).unwrap();

    let out = note(&case, directory).unwrap();
    assert_eq!(out["body"].as_str().unwrap().trim(), "Внешний текст");
    assert!(
        !case.data.join("notes").exists(),
        "конспект лёг во внутренний каталог"
    );
}

#[test]
fn одновременная_правка_возвращает_обе_версии() {
    let case = case("conflict");
    let first = save(&case, "Первый текст", json!({})).unwrap();
    let path = note(&case, json!({})).unwrap()["path"]
        .as_str()
        .unwrap()
        .to_owned();
    std::fs::write(
        &path,
        "---\ntolearn:\n  roadmap: llm-agents-base\n  topic: local-runtime\n---\n\nВерсия снаружи\n",
    )
    .unwrap();

    let refused = save(&case, "Версия из окна", json!({ "stamp": first["stamp"] })).unwrap();

    assert_eq!(refused["saved"], false);
    assert_eq!(refused["theirs"].as_str().unwrap().trim(), "Версия снаружи");
    assert_eq!(refused["stamp"], Value::Null);
    let kept = std::fs::read_to_string(&path).unwrap();
    assert!(
        kept.contains("Версия снаружи"),
        "чужой текст затёрт: {kept}"
    );
}

#[test]
fn запись_поверх_своего_же_слепка_проходит() {
    let case = case("same");
    let first = save(&case, "Первый текст", json!({})).unwrap();

    let second = save(&case, "Второй текст", json!({ "stamp": first["stamp"] })).unwrap();

    assert_eq!(second["saved"], true);
    assert_ne!(second["stamp"], first["stamp"]);
    assert_eq!(
        note(&case, json!({})).unwrap()["body"]
            .as_str()
            .unwrap()
            .trim(),
        "Второй текст"
    );
}
