#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "app gate: a panic here is the report"
)]

mod support;

use serde_json::json;
use support::snapshot;
use tolearn_app::ipc::{Context, NAMES, call};

const GONE: [&str; 6] = [
    "generate",
    "generate_state",
    "generate_go",
    "generate_stop",
    "generate_accept",
    "generate_draft",
];

const PLANNING: [&str; 2] = ["plan_program", "revise_plan"];

#[test]
fn команды_новой_программы_не_повторяют_имена_v1() {
    for name in PLANNING {
        assert!(NAMES.contains(&name), "{name}");
        assert!(!GONE.contains(&name), "{name}");
    }
}

#[test]
fn черновик_генерации_v1_приложение_не_читает_и_не_удаляет() {
    let data = std::env::temp_dir().join(format!("tolearn-draft-kept-{}", std::process::id()));
    if data.exists() {
        std::fs::remove_dir_all(&data).unwrap();
    }
    let draft = data.join("draft");
    std::fs::create_dir_all(draft.join("bundle/topics")).unwrap();
    std::fs::write(draft.join("job.json"), br#"{"subject":"Rust","done":2}"#).unwrap();
    std::fs::write(draft.join("bundle/roadmap.yaml"), b"id: rust-base\n").unwrap();
    let before = snapshot(&draft);

    let context = Context::new(&data);
    call(&context, "library", &json!({})).unwrap();
    for name in GONE {
        let refused = call(&context, name, &json!({ "today": "2026-09-11" })).unwrap_err();
        assert_eq!(refused.code, "ipc.unknown-command", "{name}");
    }

    assert_eq!(snapshot(&draft), before);
}
