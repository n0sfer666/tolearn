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

const TODAY: &str = "2026-07-29";

struct Case {
    context: Context,
    data: PathBuf,
    bundle: PathBuf,
}

fn case(name: &str) -> Case {
    let data = std::env::temp_dir().join(format!("tolearn-export-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&data);
    std::fs::create_dir_all(&data).unwrap();
    Case {
        context: Context::new(&data),
        data,
        bundle: copied(&format!("export-{name}")),
    }
}

fn export(case: &Case, extra: Value) -> Result<Value, IpcError> {
    let mut payload = json!({
        "bundle": case.bundle.display().to_string(),
        "today": TODAY,
        "path": case.data.join("программа.md").display().to_string(),
    });
    let object = payload.as_object_mut().unwrap();
    for (key, value) in extra.as_object().unwrap() {
        object.insert(key.clone(), value.clone());
    }
    call(&case.context, "export", &payload)
}

#[test]
fn экспорт_кладёт_один_файл_с_программой() {
    let case = case("file");

    let out = export(&case, json!({})).unwrap();

    let path = PathBuf::from(out["path"].as_str().unwrap());
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.starts_with("# "), "{}", &text[..40.min(text.len())]);
    assert!(text.contains("## Оглавление"), "нет оглавления");
    assert_eq!(out["bytes"].as_u64().unwrap(), text.len() as u64);
}

#[test]
fn экспорт_в_каталог_бандла_отклонён() {
    let case = case("inside");
    let path = case.bundle.join("программа.md");

    let refused = export(&case, json!({ "path": path.display().to_string() })).unwrap_err();

    assert_eq!(refused.code, "export.inside-bundle");
    assert!(!path.exists(), "файл всё же записан в бандл");
}

#[test]
fn кривая_дата_экспорт_не_запускает() {
    let case = case("date");

    let refused = export(&case, json!({ "today": "29.07.2026" })).unwrap_err();

    assert!(refused.code.contains("date"), "{}", refused.code);
    assert!(!case.data.join("программа.md").exists());
}

#[test]
fn конспекты_из_указанного_каталога_попадают_в_файл() {
    let case = case("notes");
    let outside = case.data.join("конспекты");
    let directory = json!({ "directory": outside.display().to_string() });
    call(
        &case.context,
        "save_note",
        &json!({
            "bundle": case.bundle.display().to_string(),
            "topic": "local-runtime",
            "body": "Мой текст конспекта",
            "directory": outside.display().to_string(),
        }),
    )
    .unwrap();

    let out = export(&case, directory).unwrap();

    let text = std::fs::read_to_string(out["path"].as_str().unwrap()).unwrap();
    assert!(text.contains("#### Конспект"), "конспекта нет");
    assert!(text.contains("Мой текст конспекта"), "текст не попал");
}

#[test]
fn экспорт_бандл_не_трогает() {
    let case = case("intact");
    let before = snapshot(&case.bundle);

    export(&case, json!({})).unwrap();

    assert_eq!(before, snapshot(&case.bundle), "бандл изменён экспортом");
}

fn snapshot(bundle: &std::path::Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut found: Vec<(PathBuf, Vec<u8>)> = Vec::new();
    walk(bundle, &mut found);
    found.sort_by(|left, right| left.0.cmp(&right.0));
    found
}

fn walk(room: &std::path::Path, found: &mut Vec<(PathBuf, Vec<u8>)>) {
    for entry in std::fs::read_dir(room).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            walk(&path, found);
            continue;
        }
        let body = std::fs::read(&path).unwrap();
        found.push((path, body));
    }
}
