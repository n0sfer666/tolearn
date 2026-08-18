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

const TODAY: &str = "2026-07-28";

fn context(name: &str) -> Context {
    let directory =
        std::env::temp_dir().join(format!("tolearn-history-{name}-{}", std::process::id()));
    if directory.exists() {
        std::fs::remove_dir_all(&directory).unwrap();
    }
    std::fs::create_dir_all(&directory).unwrap();
    Context::new(&directory)
}

fn imported(context: &Context, bundle: &Path) -> Value {
    let taken = call(
        context,
        "import",
        &json!({ "path": bundle.display().to_string(), "today": TODAY }),
    )
    .unwrap();
    assert_eq!(taken["ok"], true, "импорт отвергнут: {taken}");
    taken
}

fn versions(context: &Context, bundle: &Path) -> Vec<Value> {
    call(
        context,
        "history",
        &json!({ "bundle": bundle.display().to_string() }),
    )
    .unwrap()["versions"]
        .as_array()
        .unwrap()
        .clone()
}

fn difference(context: &Context, bundle: &Path, version: u32) -> Value {
    call(
        context,
        "history_diff",
        &json!({ "bundle": bundle.display().to_string(), "version": version }),
    )
    .unwrap()
}

fn ids(diff: &Value, field: &str) -> Vec<String> {
    diff[field]
        .as_array()
        .unwrap()
        .iter()
        .map(|link| link["id"].as_str().unwrap().to_owned())
        .collect()
}

fn topic(bundle: &Path, id: &str) -> PathBuf {
    bundle.join("topics").join(format!("{id}.yaml"))
}

fn reworded(bundle: &Path, id: &str) {
    let file = topic(bundle, id);
    let body = std::fs::read_to_string(&file).unwrap();
    std::fs::write(
        &file,
        body.replace(
            "\nmisconceptions:",
            "  - Ещё один результат, которого раньше не было.\n\nmisconceptions:",
        ),
    )
    .unwrap();
}

fn dropped(bundle: &Path, id: &str) {
    let file = topic(bundle, id);
    if file.exists() {
        std::fs::remove_file(&file).unwrap();
    }
    let file = bundle.join("roadmap.yaml");
    let body = std::fs::read_to_string(&file).unwrap();
    let kept: Vec<&str> = body
        .split("\n  - id: ")
        .filter(|block| !block.starts_with(id))
        .collect();
    std::fs::write(&file, kept.join("\n  - id: ")).unwrap();
}

fn depth(context: &Context, keep: u32) {
    let stored = call(context, "settings", &json!({})).unwrap();
    let mut save = stored.clone();
    save["history_depth"] = json!(keep);
    call(context, "settings", &json!({ "save": save })).unwrap();
}

#[test]
fn первый_импорт_истории_не_заводит() {
    let context = context("first");
    let bundle = copied("history-first");

    imported(&context, &bundle);

    assert!(versions(&context, &bundle).is_empty());
}

#[test]
fn повторный_импорт_сохраняет_прошлую_версию() {
    let context = context("kept");
    let first = copied("history-kept-a");
    let second = copied("history-kept-b");
    imported(&context, &first);

    imported(&context, &second);

    let kept = versions(&context, &second);
    assert_eq!(kept.len(), 1, "{kept:?}");
    assert_eq!(kept[0]["n"], 1);
    assert_eq!(kept[0]["saved_at"], TODAY);
    assert!(kept[0]["bytes"].as_u64().unwrap() > 0, "{kept:?}");
}

#[test]
fn утраченная_прошлая_папка_импорту_не_мешает() {
    let context = context("lost");
    let first = copied("history-lost-a");
    let second = copied("history-lost-b");
    imported(&context, &first);
    std::fs::remove_dir_all(&first).unwrap();

    imported(&context, &second);

    assert!(versions(&context, &second).is_empty());
}

#[test]
fn прогресс_в_снимок_не_попадает() {
    let context = context("progress");
    let first = copied("history-progress-a");
    let second = copied("history-progress-b");
    imported(&context, &first);

    imported(&context, &second);

    let store =
        std::env::temp_dir().join(format!("tolearn-history-progress-{}", std::process::id()));
    let room = store.join("history/llm-agents-base/1");
    assert!(room.join("roadmap.yaml").is_file(), "снимка нет: {room:?}");
    assert!(!room.join("progress.yaml").exists(), "прогресс скопирован");
}

#[test]
fn различие_называет_переписанную_тему() {
    let context = context("rewritten");
    let first = copied("history-rewritten-a");
    let second = copied("history-rewritten-b");
    reworded(&second, "local-runtime");
    imported(&context, &first);
    imported(&context, &second);

    let diff = difference(&context, &second, 1);

    let rewritten = diff["rewritten"].as_array().unwrap();
    assert_eq!(rewritten.len(), 1, "{rewritten:?}");
    assert_eq!(rewritten[0]["id"], "local-runtime");
    assert!(!rewritten[0]["title"].as_str().unwrap().is_empty());
    assert_eq!(rewritten[0]["changed"], json!(["outcomes"]));
    assert_eq!(rewritten[0]["demoted"], false);
    assert!(ids(&diff, "added").is_empty());
    assert!(ids(&diff, "removed").is_empty());
}

#[test]
fn различие_называет_исчезнувшую_тему() {
    let context = context("removed");
    let first = copied("history-removed-a");
    let second = copied("history-removed-b");
    dropped(&second, "dev-workflow-integration");
    imported(&context, &first);
    imported(&context, &second);

    let diff = difference(&context, &second, 1);

    assert_eq!(ids(&diff, "removed"), ["dev-workflow-integration"]);
    assert!(ids(&diff, "added").is_empty());
}

#[test]
fn понижение_зачёта_названо_в_различии() {
    let context = context("demoted");
    let first = copied("history-demoted-a");
    let second = copied("history-demoted-b");
    reworded(&second, "local-runtime");
    imported(&context, &first);
    call(
        &context,
        "set_status",
        &json!({
            "bundle": first.display().to_string(),
            "topic": "local-runtime",
            "status": "passed",
            "today": TODAY,
        }),
    )
    .unwrap();
    imported(&context, &second);

    let diff = difference(&context, &second, 1);

    assert_eq!(diff["rewritten"][0]["demoted"], true, "{diff}");
}

#[test]
fn глубина_истории_режет_старые_версии() {
    let context = context("depth");
    depth(&context, 1);
    let first = copied("history-depth-a");
    let second = copied("history-depth-b");
    let third = copied("history-depth-c");
    imported(&context, &first);
    imported(&context, &second);

    imported(&context, &third);

    let kept = versions(&context, &third);
    assert_eq!(kept.len(), 1, "{kept:?}");
    assert_eq!(kept[0]["n"], 2);
}

#[test]
fn выключенная_история_снимков_не_держит() {
    let context = context("off");
    depth(&context, 0);
    let first = copied("history-off-a");
    let second = copied("history-off-b");
    imported(&context, &first);

    imported(&context, &second);

    assert!(versions(&context, &second).is_empty());
}

#[test]
fn чтение_истории_в_бандл_не_пишет() {
    let context = context("quiet");
    let first = copied("history-quiet-a");
    let second = copied("history-quiet-b");
    imported(&context, &first);
    imported(&context, &second);
    let before = std::fs::read_to_string(second.join("progress.yaml")).unwrap();

    versions(&context, &second);
    difference(&context, &second, 1);

    assert_eq!(
        std::fs::read_to_string(second.join("progress.yaml")).unwrap(),
        before
    );
}

#[test]
fn несуществующая_версия_не_читается_молча() {
    let context = context("absent");
    let bundle = copied("history-absent");
    imported(&context, &bundle);

    let refused = call(
        &context,
        "history_diff",
        &json!({ "bundle": bundle.display().to_string(), "version": 7 }),
    );

    assert_eq!(
        refused.unwrap_err().code,
        "history.unknown-version",
        "отказ не про отсутствующую версию"
    );
}

#[test]
fn версии_отдаются_по_возрасту_а_не_по_имени_каталога() {
    let context = context("order");
    let bundle = copied("history-order");
    imported(&context, &bundle);
    let store = std::env::temp_dir()
        .join(format!("tolearn-history-order-{}", std::process::id()))
        .join("history/llm-agents-base");
    for n in ["10", "9"] {
        let room = store.join(n);
        std::fs::create_dir_all(&room).unwrap();
        std::fs::write(room.join("saved-at"), TODAY).unwrap();
    }

    let kept = versions(&context, &bundle);

    assert_eq!(
        kept.iter()
            .map(|it| it["n"].as_u64().unwrap())
            .collect::<Vec<u64>>(),
        [9, 10]
    );
}

#[test]
fn вес_снимка_считает_и_вложенные_каталоги() {
    let context = context("weight");
    let first = copied("history-weight-a");
    let second = copied("history-weight-b");
    imported(&context, &first);
    imported(&context, &second);
    let room = std::env::temp_dir()
        .join(format!("tolearn-history-weight-{}", std::process::id()))
        .join("history/llm-agents-base/1");
    let shallow: u64 = std::fs::read_dir(&room)
        .unwrap()
        .flatten()
        .filter(|entry| entry.path().is_file())
        .map(|entry| entry.metadata().unwrap().len())
        .sum();

    let kept = versions(&context, &second);

    assert!(
        kept[0]["bytes"].as_u64().unwrap() > shallow,
        "темы снимка в вес не попали: {} против {shallow}",
        kept[0]["bytes"]
    );
}
