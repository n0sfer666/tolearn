#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "app gate: a panic here is the report"
)]

mod support;

use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use support::{copied, gzipped, zipped};
use tolearn_app::ipc::{Context, call};

const TODAY: &str = "2026-07-28";

fn context(name: &str) -> (Context, PathBuf) {
    let data = std::env::temp_dir().join(format!("tolearn-archive-{name}-{}", std::process::id()));
    if data.exists() {
        std::fs::remove_dir_all(&data).unwrap();
    }
    std::fs::create_dir_all(&data).unwrap();
    (Context::new(&data), data)
}

fn imported(context: &Context, path: &Path) -> Value {
    call(
        context,
        "import",
        &json!({ "path": path.display().to_string(), "today": TODAY }),
    )
    .unwrap()
}

fn programs(context: &Context) -> Vec<Value> {
    call(context, "programs", &json!({ "today": TODAY })).unwrap()["programs"]
        .as_array()
        .unwrap()
        .clone()
}

fn leftovers(data: &Path) -> Vec<String> {
    let room = data.join("unpacked");
    if !room.is_dir() {
        return Vec::new();
    }
    std::fs::read_dir(room)
        .unwrap()
        .flatten()
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| name.starts_with(".taking-"))
        .collect()
}

#[test]
fn zip_импортируется_как_папка() {
    let (context, data) = context("zip");
    let bundle = copied("archive-zip");
    let archive = zipped(&bundle);
    std::fs::remove_dir_all(&bundle).unwrap();

    let taken = imported(&context, &archive);

    assert_eq!(taken["ok"], true, "{taken}");
    assert_eq!(taken["id"], "llm-agents-base");
    let programs = programs(&context);
    assert_eq!(programs.len(), 1, "{programs:?}");
    assert_eq!(
        programs[0]["path"].as_str().unwrap(),
        data.join("unpacked")
            .join("llm-agents-base")
            .display()
            .to_string()
    );
    assert!(data.join("unpacked/llm-agents-base/roadmap.yaml").is_file());
    assert!(leftovers(&data).is_empty(), "остался рабочий каталог");
}

#[test]
fn tar_gz_даёт_тот_же_результат_что_и_zip() {
    let (context, data) = context("targz");
    let bundle = copied("archive-targz");
    let archive = gzipped(&bundle);
    std::fs::remove_dir_all(&bundle).unwrap();

    let taken = imported(&context, &archive);

    assert_eq!(taken["ok"], true, "{taken}");
    assert_eq!(taken["id"], "llm-agents-base");
    assert!(data.join("unpacked/llm-agents-base/roadmap.yaml").is_file());
    assert!(leftovers(&data).is_empty(), "остался рабочий каталог");
}

#[test]
fn прогресс_нового_бандла_живёт_рядом_с_ним() {
    let (context, data) = context("progress");
    let bundle = copied("archive-progress");
    let archive = zipped(&bundle);
    std::fs::remove_dir_all(&bundle).unwrap();
    imported(&context, &archive);

    let home = data.join("unpacked/llm-agents-base");
    call(
        &context,
        "set_status",
        &json!({
            "bundle": home.display().to_string(),
            "topic": "local-runtime",
            "status": "passed",
            "today": TODAY,
        }),
    )
    .unwrap();

    let written = std::fs::read_to_string(home.join("progress.yaml")).unwrap();
    assert!(written.contains("local-runtime"), "{written}");
}

#[test]
fn повторный_импорт_архива_обновляет_ту_же_папку() {
    let (context, data) = context("again");
    let bundle = copied("archive-again");
    let archive = zipped(&bundle);
    imported(&context, &archive);

    let taken = imported(&context, &archive);

    assert_eq!(taken["ok"], true, "{taken}");
    assert_eq!(programs(&context).len(), 1);
    assert!(
        data.join("history/llm-agents-base/1").is_dir(),
        "прошлая версия в историю не ушла"
    );
    assert!(leftovers(&data).is_empty(), "остался рабочий каталог");
}

#[test]
fn негодный_бандл_из_архива_распакованным_не_остаётся() {
    let (context, data) = context("broken");
    let bundle = copied("archive-broken");
    std::fs::write(bundle.join("roadmap.yaml"), "id: broken\n").unwrap();
    let archive = zipped(&bundle);
    std::fs::remove_dir_all(&bundle).unwrap();

    let taken = imported(&context, &archive);

    assert_eq!(taken["ok"], false, "{taken}");
    assert!(programs(&context).is_empty());
    assert!(leftovers(&data).is_empty(), "остался рабочий каталог");
    assert!(!data.join("unpacked/broken").exists());
}

#[test]
fn не_архив_отвергается_кодом() {
    let (context, _data) = context("nonsense");
    let file = std::env::temp_dir().join(format!("tolearn-nonsense-{}.zip", std::process::id()));
    std::fs::write(&file, "не архив вовсе").unwrap();

    let refused = call(
        &context,
        "import",
        &json!({ "path": file.display().to_string(), "today": TODAY }),
    );

    assert_eq!(refused.unwrap_err().code, "archive.unknown-format");
}
