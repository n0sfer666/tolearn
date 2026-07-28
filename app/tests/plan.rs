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
    let directory = std::env::temp_dir().join(format!("tolearn-plan-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    Context::new(&directory)
}

fn planned(bundle: &Path, today: &str) -> Value {
    call(
        &context(),
        "plan",
        &json!({ "bundle": bundle.display().to_string(), "today": today }),
    )
    .unwrap()
}

fn passed(bundle: &Path, topic: &str) {
    call(
        &context(),
        "set_status",
        &json!({
            "bundle": bundle.display().to_string(),
            "topic": topic,
            "status": "passed",
            "today": TODAY,
        }),
    )
    .unwrap();
}

#[test]
fn дневная_норма_и_прогноз_считаются_из_шапки_программы() {
    let bundle = copied("plan-budget");

    let ahead = planned(&bundle, TODAY);

    assert_eq!(ahead["weekly_hours"], json!(6));
    let daily = ahead["daily_hours"].as_f64().unwrap();
    assert!((daily - 6.0 / 7.0).abs() < 1e-9, "{ahead}");
    let left = ahead["left"]["min"].as_f64().unwrap();
    assert_eq!(
        ahead["soonest"]["days"].as_u64().unwrap(),
        (left / daily).ceil() as u64,
        "{ahead}"
    );
    assert!(
        ahead["latest"]["days"].as_u64() > ahead["soonest"]["days"].as_u64(),
        "прогноз не интервал: {ahead}"
    );
}

#[test]
fn прогноз_это_две_даты_а_не_одна() {
    let bundle = copied("plan-dates");

    let ahead = planned(&bundle, TODAY);

    let soonest = ahead["soonest"]["date"].as_str().unwrap();
    let latest = ahead["latest"]["date"].as_str().unwrap();
    assert!(soonest > TODAY, "{ahead}");
    assert!(latest > soonest, "{ahead}");
}

#[test]
fn пройденная_тема_приближает_обе_даты() {
    let bundle = copied("plan-passed");
    let before = planned(&bundle, TODAY);

    passed(&bundle, "local-runtime");

    let after = planned(&bundle, TODAY);
    assert!(
        after["left"]["max"].as_u64().unwrap() < before["left"]["max"].as_u64().unwrap(),
        "{after}"
    );
    assert!(
        after["latest"]["date"].as_str().unwrap() < before["latest"]["date"].as_str().unwrap(),
        "{after}"
    );
}

#[test]
fn несгенерированные_темы_названы_отдельно_и_в_интервал_не_входят() {
    let bundle = copied("plan-unknown");
    let ahead = planned(&bundle, TODAY);

    let unknown = ahead["unknown"].as_u64().unwrap();
    assert!(unknown > 0, "в эталоне не все темы написаны: {ahead}");

    std::fs::remove_file(bundle.join("topics/local-runtime.yaml")).unwrap();
    let shorter = planned(&bundle, TODAY);

    assert_eq!(shorter["unknown"].as_u64().unwrap(), unknown + 1);
    assert!(
        shorter["left"]["max"].as_u64().unwrap() < ahead["left"]["max"].as_u64().unwrap(),
        "часы ненаписанной темы попали в интервал: {shorter}"
    );
}

#[test]
fn кривая_дата_отвергается_до_счёта() {
    let bundle = copied("plan-date");

    let refused = call(
        &context(),
        "plan",
        &json!({ "bundle": bundle.display().to_string(), "today": "вчера" }),
    )
    .unwrap_err();

    assert_eq!(refused.code, "date.malformed");
}
