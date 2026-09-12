#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "app gate: a panic here is the report"
)]

mod support;

use serde_json::json;
use support::snapshot;
use tolearn_app::ipc::{Context, call};

#[test]
fn импорт_и_реестр_v1_убраны() {
    let context = Context::new(&std::env::temp_dir().join("tolearn-registry-gone"));
    for name in ["import", "programs"] {
        let payload = json!({ "path": "/nowhere", "today": "2026-09-12" });

        let error = call(&context, name, &payload).unwrap_err();

        assert_eq!(error.code, "ipc.unknown-command", "{name}");
    }
}

#[test]
fn реестр_и_распакованные_бандлы_v1_приложение_не_читает_и_не_удаляет() {
    let data = std::env::temp_dir().join(format!("tolearn-registry-kept-{}", std::process::id()));
    if data.exists() {
        std::fs::remove_dir_all(&data).unwrap();
    }
    let unpacked = data.join("unpacked/rust-base");
    std::fs::create_dir_all(unpacked.join("topics")).unwrap();
    std::fs::write(
        unpacked.join("roadmap.yaml"),
        b"id: rust-base\ntitle: Rust\n",
    )
    .unwrap();
    std::fs::write(
        unpacked.join("topics/ownership.yaml"),
        b"schema: learning-topic/v1\nid: ownership\ntitle: Rust ownership\n",
    )
    .unwrap();
    let registry = data.join("registry.yaml");
    let listed = format!(
        "programs:\n  - id: rust-base\n    title: Rust\n    path: {}\n    opened_at: 2026-09-01\n",
        unpacked.display()
    );
    std::fs::write(&registry, &listed).unwrap();
    let before = snapshot(&data.join("unpacked"));

    let context = Context::new(&data);
    let library = call(&context, "library", &json!({})).unwrap();
    call(&context, "settings", &json!({ "save": null })).unwrap();
    let found = call(&context, "search", &json!({ "query": "rust", "limit": 5 })).unwrap();

    assert_eq!(library, json!({ "programs": [], "refused": [] }));
    assert_eq!(found["hits"], json!([]));
    assert_eq!(snapshot(&data.join("unpacked")), before);
    assert_eq!(std::fs::read_to_string(&registry).unwrap(), listed);
}
