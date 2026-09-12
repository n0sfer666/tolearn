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
use support::{repository, sources};
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
    let error = call(&context(), "node", &json!({})).unwrap_err();

    assert_eq!(error.code, "ipc.malformed-payload");
    assert!(!error.message.is_empty());
}

#[test]
fn ошибка_ядра_доезжает_кодом_и_сообщением() {
    let error = call(
        &context(),
        "node",
        &json!({ "program": "nowhere-at-all", "node": "" }),
    )
    .unwrap_err();

    assert_eq!(error.code, "library.absent");
    assert!(
        error.message.contains("nowhere-at-all"),
        "{}",
        error.message
    );
}

#[test]
fn команды_экранов_изучения_v1_убраны() {
    for name in [
        "topic",
        "run_check",
        "set_status",
        "parse_verdict",
        "apply_verdict",
        "review",
        "prompt",
        "examine",
        "validate",
        "scan",
        "program",
    ] {
        let error = call(&context(), name, &json!({})).unwrap_err();

        assert_eq!(error.code, "ipc.unknown-command", "{name}");
    }
}

#[test]
fn описание_команд_держит_обе_формы() {
    let mut bare = Vec::new();
    for descriptor in descriptors() {
        assert!(descriptor.input.name.ends_with("In"), "{}", descriptor.name);
        assert!(!descriptor.output.fields.is_empty(), "{}", descriptor.name);
        if descriptor.input.fields.is_empty() {
            bare.push(descriptor.name);
        }
    }
    assert_eq!(bare, ["library"]);
}
