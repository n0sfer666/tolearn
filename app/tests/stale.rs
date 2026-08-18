#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "app gate: a panic here is the report"
)]

mod support;

use std::path::Path;

use serde_json::{Value, json};
use support::copied;
use tolearn_app::ipc::{Context, call};

const TODAY: &str = "2026-07-28";

fn context() -> Context {
    let directory = std::env::temp_dir().join(format!("tolearn-stale-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    Context::new(&directory)
}

fn aging(bundle: &Path, today: &str) -> Value {
    call(
        &context(),
        "stale",
        &json!({ "bundle": bundle.display().to_string(), "today": today }),
    )
    .unwrap()
}

#[test]
fn дайджест_называет_и_темы_и_материалы() {
    let bundle = copied("stale-digest");

    let taken = aging(&bundle, "2030-01-01");

    let topics = taken["topics"].as_array().unwrap();
    let materials = taken["materials"].as_array().unwrap();
    assert!(!topics.is_empty(), "{taken}");
    assert!(!materials.is_empty(), "{taken}");
    assert!(topics.iter().all(|item| item["expired_at"].is_string()));
}

#[test]
fn до_срока_годности_тем_в_дайджесте_нет() {
    let bundle = copied("stale-early");

    let early = aging(&bundle, "2026-01-01");
    let late = aging(&bundle, "2030-01-01");

    assert_eq!(early["topics"].as_array().unwrap().len(), 0, "{early}");
    assert!(!late["topics"].as_array().unwrap().is_empty(), "{late}");
}

#[test]
fn материал_несёт_причину_а_не_голый_флаг() {
    let bundle = copied("stale-reason");

    let taken = aging(&bundle, TODAY);

    let materials = taken["materials"].as_array().unwrap();
    assert!(
        materials.iter().all(|item| {
            item["stale"].as_bool().unwrap()
                || item["delta"].is_string()
                || item["pin"] == json!("unknown")
        }),
        "{taken}"
    );
    assert!(
        materials.iter().any(|item| item["delta"].is_string()),
        "{taken}"
    );
    assert!(
        materials.iter().all(|item| item["topic_title"].is_string()),
        "{taken}"
    );
}

#[test]
fn версия_мимо_пинов_названа_неизвестной() {
    let bundle = copied("stale-pin");

    let taken = aging(&bundle, TODAY);

    let unknown: Vec<&Value> = taken["materials"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|item| item["pin"] == json!("unknown"))
        .collect();
    assert!(!unknown.is_empty(), "{taken}");
    assert!(
        unknown
            .iter()
            .all(|item| item["covers_version"].is_string()),
        "{unknown:?}"
    );
    assert!(
        taken["materials"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|item| item["covers_version"].is_null())
            .all(|item| item["pin"] == json!("absent")),
        "{taken}"
    );
}

#[test]
fn кривая_дата_отвергается_до_счёта() {
    let bundle = copied("stale-date");

    let refused = call(
        &context(),
        "stale",
        &json!({ "bundle": bundle.display().to_string(), "today": "позавчера" }),
    )
    .unwrap_err();

    assert_eq!(refused.code, "date.malformed");
}
