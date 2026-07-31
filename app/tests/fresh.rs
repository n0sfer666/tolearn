#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "app gate: a panic here is the report"
)]

mod support;

use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use support::copied;
use tolearn_app::ipc::{Context, call};
use tolearn_offline::store::{Checked, Fetched, Store};

const PAGE: &str = "<html><head><title>Ollama FAQ</title></head><body><p>тело</p></body></html>";

fn context(name: &str) -> (Context, PathBuf) {
    let data = std::env::temp_dir().join(format!("tolearn-fresh-{name}-{}", std::process::id()));
    if data.exists() {
        std::fs::remove_dir_all(&data).unwrap();
    }
    std::fs::create_dir_all(&data).unwrap();
    (Context::new(&data), data.join("offline"))
}

fn topic(context: &Context, bundle: &Path) -> Value {
    call(
        context,
        "topic",
        &json!({
            "bundle": bundle.display().to_string(),
            "topic": "local-runtime",
            "today": "2026-07-27",
        }),
    )
    .unwrap()
}

fn urls(topic: &Value) -> Vec<String> {
    topic["materials"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["url"].as_str().unwrap().to_owned())
        .collect()
}

fn everything(root: &Path, program: &str, urls: &[String], at: Option<i64>) {
    let mut store = Store::open(root, 64 * 1024 * 1024).unwrap();
    for url in urls {
        store
            .put(
                url,
                program,
                &Fetched {
                    kind: "archive",
                    bytes: PAGE.as_bytes(),
                    etag: None,
                    last_modified: None,
                },
                1_700_000_000,
            )
            .unwrap();
        let Some(at) = at else { continue };
        store
            .stamp(
                url,
                &Checked {
                    body_hash: Some("hash-1".to_owned()),
                    etag: None,
                    last_modified: None,
                    at,
                },
            )
            .unwrap();
    }
}

fn сейчас() -> i64 {
    i64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    )
    .unwrap()
}

#[test]
fn кнопка_темы_проходит_путь_от_сохранения_до_свежести() {
    let bundle = copied("fresh-unload");
    let (context, root) = context("unload");

    let empty = topic(&context, &bundle);
    assert_eq!(empty["unload"]["state"], "missing");

    let addresses = urls(&empty);
    everything(&root, "llm-agents-base", &addresses, None);
    let taken = topic(&context, &bundle);
    assert_eq!(
        taken["unload"]["state"], "stale",
        "непроверенное сочли свежим"
    );

    let at = сейчас();
    everything(&root, "llm-agents-base", &addresses, Some(at));
    let checked = topic(&context, &bundle);
    assert_eq!(checked["unload"]["state"], "fresh");
    assert_eq!(checked["unload"]["checked_at"], json!(at));
}
