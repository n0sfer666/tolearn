#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "app gate: a panic here is the report"
)]

mod support;

use std::path::{Path, PathBuf};

use serde_json::json;
use support::snapshot;
use tolearn_app::ipc::{Context, call};
use tolearn_offline::store::{Fetched, Store};

const URL: &str = "https://docs.ollama.com/faq";

fn context(name: &str) -> (Context, PathBuf) {
    let data = support::scratch::made(&format!("offline-{name}"));
    (Context::new(&data), data.join("offline"))
}

fn saved(root: &Path) {
    let page = vec![b'a'; 2 * 1024 * 1024];
    let mut store = Store::open(root, 64 * 1024 * 1024).unwrap();
    store
        .put(
            URL,
            "rust-base",
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
fn настройки_не_срезают_офлайн_хранилище_v1() {
    let (context, root) = context("kept");
    saved(&root);
    let before = snapshot(&root);

    call(
        &context,
        "settings",
        &json!({ "save": { "disk_budget_mb": 1, "locale": "ru", "theme": "system" } }),
    )
    .unwrap();

    assert!(!before.is_empty());
    assert_eq!(snapshot(&root), before);
}
