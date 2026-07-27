#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "app gate: a panic here is the report"
)]

mod support;

fn context() -> Context {
    Context::new(&std::env::temp_dir().join(format!("tolearn-contract-{}", std::process::id())))
}

use std::collections::BTreeSet;
use std::path::PathBuf;

use serde_json::json;
use support::{copied, repository, sources};
use tolearn_app::ipc::{Context, NAMES, call, descriptors, typescript};

#[test]
fn таблица_перечисляет_ровно_обработчики_из_каталога() {
    let mut declared: BTreeSet<String> = NAMES.iter().map(|name| (*name).to_owned()).collect();
    let directory = repository().join("app/src/ipc/handlers");
    let mut files = Vec::new();
    sources(&directory, &mut files);

    let written: BTreeSet<String> = files
        .iter()
        .filter_map(|file| file.file_stem())
        .map(|stem| stem.to_string_lossy().into_owned())
        .filter(|stem| stem != "mod")
        .collect();

    assert_eq!(
        written,
        std::mem::take(&mut declared),
        "обработчик мимо таблицы команд или команда без обработчика"
    );
}

#[test]
fn команда_объявляется_только_таблицей() {
    let mut files = Vec::new();
    sources(&repository().join("app/src"), &mut files);
    let table = repository().join("app/src/ipc/contract.rs");

    let stray: Vec<PathBuf> = files
        .into_iter()
        .filter(|file| *file != table)
        .filter(|file| {
            std::fs::read_to_string(file)
                .unwrap()
                .contains("tauri::command")
        })
        .collect();

    assert!(
        stray.is_empty(),
        "команда объявлена мимо таблицы: {stray:?}"
    );
}

#[test]
fn типы_для_ui_совпадают_с_файлом_в_репозитории() {
    let file = repository().join("ui/src/ipc.d.ts");
    let written =
        std::fs::read_to_string(&file).expect("ui/src/ipc.d.ts должен лежать в репозитории");

    assert_eq!(
        written,
        typescript::emit(),
        "типы UI разошлись с контрактом: `cargo run -p tolearn-app --example ipc-types`"
    );
}

#[test]
fn каждый_тип_поля_объявлен() {
    let shapes = tolearn_app::ipc::shapes();
    let known: BTreeSet<&str> = shapes
        .iter()
        .map(|shape| shape.name.as_str())
        .chain(["string", "number", "boolean"])
        .collect();

    for shape in &shapes {
        for field in &shape.fields {
            let ty = field.ty.trim_end_matches("[]").trim_end_matches(" | null");
            assert!(
                known.contains(ty),
                "поле `{}.{}` ссылается на необъявленный тип `{ty}`",
                shape.name,
                field.name
            );
        }
    }
}

#[test]
fn неизвестная_команда_отвечает_кодом() {
    let error = call(&context(), "fly", &json!({})).unwrap_err();

    assert_eq!(error.code, "ipc.unknown-command");
    assert!(error.message.contains("fly"), "{}", error.message);
}

#[test]
fn битый_ввод_отвечает_кодом_а_не_паникой() {
    let error = call(&context(), "validate", &json!({})).unwrap_err();

    assert_eq!(error.code, "ipc.malformed-payload");
    assert!(!error.message.is_empty());
}

#[test]
fn ошибка_ядра_доезжает_кодом_и_сообщением() {
    let error = call(
        &context(),
        "validate",
        &json!({ "bundle": "/nowhere-at-all" }),
    )
    .unwrap_err();

    assert_eq!(error.code, "scan.no-roadmap");
    assert!(
        error.message.contains("nowhere-at-all"),
        "{}",
        error.message
    );
}

#[test]
fn два_формата_в_бандле_отличаются_кодом() {
    let root = repository().join("examples/llm-agents-base");
    let error = call(&context(), "validate", &json!({ "bundle": root })).unwrap_err();

    assert_eq!(error.code, "scan.ambiguous-format");
}

#[test]
fn эталонный_бандл_проходит_валидацию() {
    let root = copied("validate");
    let out = call(&context(), "validate", &json!({ "bundle": root })).unwrap();

    assert_eq!(out["ok"], json!(true), "{out:#}");
    assert_eq!(out["violations"], json!([]));
}

#[test]
fn бандл_с_нарушением_отвечает_нет_и_перечисляет_коды() {
    let root = copied("broken");
    let file = root.join("roadmap.yaml");
    let source = std::fs::read_to_string(&file).unwrap();
    std::fs::write(
        &file,
        source.replacen("est_hours: [4, 6]", "est_hours: [6, 4]", 1),
    )
    .unwrap();

    let out = call(&context(), "validate", &json!({ "bundle": root })).unwrap();

    assert_eq!(out["ok"], json!(false), "{out:#}");
    assert_eq!(out["violations"][0]["code"], json!("bundle.hours-reversed"));
    assert!(!out["violations"][0]["message"].as_str().unwrap().is_empty());
}

#[test]
fn обзор_бандла_называет_формат_и_темы() {
    let root = copied("scan");
    let out = call(&context(), "scan", &json!({ "bundle": root })).unwrap();

    assert_eq!(out["format"], json!("yaml"));
    assert!(!out["topics"].as_array().unwrap().is_empty());
    assert_eq!(out["broken"], json!([]));
}

#[test]
fn сводка_считает_этапы_и_статусы() {
    let root = copied("program");
    let out = call(
        &context(),
        "program",
        &json!({ "bundle": root, "today": "2026-07-27" }),
    )
    .unwrap();

    assert!(out["program"]["total"].as_u64().unwrap() > 0, "{out:#}");
    assert!(!out["stages"].as_array().unwrap().is_empty());
    assert!(!out["topics"].as_array().unwrap().is_empty());
}

#[test]
fn кривая_дата_отвечает_кодом() {
    let root = copied("bad-date");
    let error = call(
        &context(),
        "program",
        &json!({ "bundle": root, "today": "вчера" }),
    )
    .unwrap_err();

    assert_eq!(error.code, "date.malformed");
}

#[test]
fn промпт_темы_собирается_из_шаблона() {
    let root = copied("prompt");
    let topics = call(&context(), "scan", &json!({ "bundle": &root })).unwrap();
    let first = topics["topics"][0].as_str().unwrap().to_owned();

    let out = call(
        &context(),
        "prompt",
        &json!({ "bundle": root, "topic": first }),
    )
    .unwrap();

    assert!(!out["text"].as_str().unwrap().is_empty());
}

#[test]
fn неизвестная_тема_отвечает_кодом() {
    let root = copied("unknown-topic");
    let error = call(
        &context(),
        "prompt",
        &json!({ "bundle": root, "topic": "нет-такой" }),
    )
    .unwrap_err();

    assert_eq!(error.code, "topic.unknown");
}

#[test]
fn описание_команд_держит_обе_формы() {
    for descriptor in descriptors() {
        assert!(!descriptor.input.fields.is_empty(), "{}", descriptor.name);
        assert!(!descriptor.output.fields.is_empty(), "{}", descriptor.name);
    }
}
