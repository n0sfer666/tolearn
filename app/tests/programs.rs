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

fn data(name: &str) -> PathBuf {
    let directory =
        std::env::temp_dir().join(format!("tolearn-data-{name}-{}", std::process::id()));
    if directory.exists() {
        std::fs::remove_dir_all(&directory).unwrap();
    }
    std::fs::create_dir_all(&directory).unwrap();
    directory
}

fn import(context: &Context, bundle: &Path) -> Value {
    call(
        context,
        "import",
        &json!({ "path": bundle.display().to_string(), "today": "2026-07-27" }),
    )
    .unwrap()
}

fn programs(context: &Context) -> Value {
    call(context, "programs", &json!({ "today": "2026-07-27" })).unwrap()
}

fn edit(file: &Path, from: &str, to: &str) {
    let text = std::fs::read_to_string(file).unwrap();
    assert!(text.contains(from), "нечего менять в {}", file.display());
    std::fs::write(file, text.replacen(from, to, 1)).unwrap();
}

#[test]
fn импорт_регистрирует_программу_и_список_переживает_перезапуск() {
    let root = data("import");
    let bundle = copied("import");
    let imported = import(&Context::new(&root), &bundle);

    assert_eq!(imported["ok"], json!(true), "{imported}");
    assert_eq!(imported["id"], json!("llm-agents-base"));

    let listed = programs(&Context::new(&root));
    let programs = listed["programs"].as_array().unwrap();
    assert_eq!(programs.len(), 1);
    assert_eq!(programs[0]["id"], json!("llm-agents-base"));
    assert_eq!(programs[0]["reachable"], json!(true));
    assert!(programs[0]["title"].as_str().unwrap().contains("агентами"));
    assert!(
        programs[0]["tally"]["total"].as_u64().unwrap() > 0,
        "прогресс не виден"
    );
}

#[test]
fn импорт_битой_папки_объясняет_причину_и_ничего_не_регистрирует() {
    let root = data("broken");
    let bundle = copied("broken");
    edit(&bundle.join("roadmap.yaml"), "id: llm-agents-base", "id: ");

    let refused = import(&Context::new(&root), &bundle);

    assert_eq!(refused["ok"], json!(false));
    let violations = refused["violations"].as_array().unwrap();
    assert!(!violations.is_empty(), "причина не названа");
    assert!(
        violations[0]["message"].as_str().unwrap().len() > 10,
        "сообщение не объясняет: {}",
        violations[0]
    );
    assert!(
        programs(&Context::new(&root))["programs"]
            .as_array()
            .unwrap()
            .is_empty(),
        "битая программа попала в реестр"
    );
}

#[test]
fn неполный_бандл_отвергается_по_коду_нарушения_а_не_по_падению_скана() {
    let root = data("incomplete");
    let bundle = copied("incomplete");
    std::fs::remove_file(bundle.join("topics/local-runtime.yaml")).unwrap();

    let refused = import(&Context::new(&root), &bundle);

    assert_eq!(refused["ok"], json!(false), "{refused}");
    let codes: Vec<&str> = refused["violations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|violation| violation["code"].as_str().unwrap())
        .collect();
    assert!(
        codes.iter().any(|code| code.starts_with("bundle.")),
        "нарушение бандла не названо: {codes:?}"
    );
    assert!(
        programs(&Context::new(&root))["programs"]
            .as_array()
            .unwrap()
            .is_empty(),
        "неполная программа попала в реестр"
    );
}

#[test]
fn повторный_импорт_показывает_отчёт_о_слиянии_и_переносит_прогресс() {
    let root = data("merge");
    let context = Context::new(&root);
    let before = copied("merge-before");
    import(&context, &before);
    edit(
        &before.join("progress.yaml"),
        "status: todo",
        "status: passed",
    );

    let after = copied("merge-after");
    edit(
        &after.join("topics/local-runtime.yaml"),
        "outcomes:",
        "outcomes:\n  - Новый результат, которого раньше не было\n",
    );
    let merged = import(&context, &after);

    assert_eq!(merged["ok"], json!(true), "{merged}");
    let report = &merged["report"];
    assert!(
        !report["kept"].as_array().unwrap().is_empty(),
        "нет сохранённых тем"
    );
    let stale = report["stale"].as_array().unwrap();
    assert_eq!(stale.len(), 1, "изменившаяся тема не понижена: {report}");
    assert_eq!(stale[0]["id"], json!("local-runtime"));
    assert_eq!(stale[0]["changed"], json!(["outcomes"]));

    let listed = programs(&context);
    let program = &listed["programs"].as_array().unwrap()[0];
    assert_eq!(
        program["path"],
        json!(after.display().to_string()),
        "реестр остался на старой папке"
    );
    assert_eq!(
        program["tally"]["stale"],
        json!(1),
        "перенесённый прогресс потерян"
    );
}

#[test]
fn повторный_импорт_той_же_папки_ничего_не_ломает() {
    let root = data("same");
    let context = Context::new(&root);
    let bundle = copied("same");
    import(&context, &bundle);
    let again = import(&context, &bundle);

    assert_eq!(again["ok"], json!(true), "{again}");
    let report = &again["report"];
    assert!(report["stale"].as_array().unwrap().is_empty(), "{report}");
    assert!(report["added"].as_array().unwrap().is_empty(), "{report}");
    assert!(
        report["orphaned"].as_array().unwrap().is_empty(),
        "{report}"
    );
    assert_eq!(programs(&context)["programs"].as_array().unwrap().len(), 1);
}

#[test]
fn недоступный_путь_помечается_а_не_удаляется() {
    let root = data("gone");
    let context = Context::new(&root);
    let bundle = copied("gone");
    import(&context, &bundle);
    std::fs::remove_dir_all(&bundle).unwrap();

    let listed = programs(&context);
    let programs = listed["programs"].as_array().unwrap();
    assert_eq!(programs.len(), 1, "запись пропала вместо пометки");
    assert_eq!(programs[0]["reachable"], json!(false));
    assert_eq!(
        programs[0]["tally"],
        Value::Null,
        "прогресс недоступной программы выдуман"
    );
}
