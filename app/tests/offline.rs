#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "app gate: a panic here is the report"
)]

mod support;

use std::path::{Path, PathBuf};

use serde_json::json;
use support::{copied, snapshot};
use tolearn_app::ipc::{Context, call};
use tolearn_offline::store::{Fetched, Store};

const URL: &str = "https://docs.ollama.com/faq";

fn context(name: &str) -> (Context, PathBuf) {
    let data = std::env::temp_dir().join(format!("tolearn-offline-{name}-{}", std::process::id()));
    if data.exists() {
        std::fs::remove_dir_all(&data).unwrap();
    }
    std::fs::create_dir_all(&data).unwrap();
    (Context::new(&data), data.join("offline"))
}

fn saved(root: &Path) {
    let page = vec![b'a'; 2 * 1024 * 1024];
    let mut store = Store::open(root, 64 * 1024 * 1024).unwrap();
    store
        .put(
            URL,
            "llm-agents-base",
            &Fetched {
                kind: "archive",
                bytes: &page,
                etag: None,
                last_modified: None,
            },
            1_700_000_000,
        )
        .unwrap();
}

#[test]
fn офлайн_хранилище_v1_приложение_не_читает_и_не_срезает() {
    let bundle = copied("offline-kept");
    let (context, root) = context("kept");
    saved(&root);
    let before = snapshot(&root);

    call(
        &context,
        "settings",
        &json!({ "save": { "disk_budget_mb": 1, "locale": "ru", "theme": "system" } }),
    )
    .unwrap();
    let topic = call(
        &context,
        "topic",
        &json!({
            "bundle": bundle.display().to_string(),
            "topic": "local-runtime",
            "today": "2026-07-27",
        }),
    )
    .unwrap();

    assert!(!before.is_empty());
    assert_eq!(snapshot(&root), before);
    assert!(topic.get("unload").is_none(), "{topic:#}");
    for material in topic["materials"].as_array().unwrap() {
        assert!(material.get("offline").is_none(), "{material:#}");
    }
}
