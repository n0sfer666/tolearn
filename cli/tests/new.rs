#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "cli gate: a panic here is the report"
)]

mod generating;

use generating::answers::{flat, said, told};
use generating::{Desk, LEVEL, REQUEST, provided};
use serde_json::{Value, json};
use tolearn_core::block::Kind;
use tolearn_core::library::Library;
use tolearn_core::package;

#[test]
fn new_строит_карту_и_первый_этап_в_своей_папке_данных() {
    let model = told(flat());
    let desk = Desk::new(&model);

    let shown = desk.begun();

    let tree = desk.program();
    let path = desk.path();
    assert!(path.join("program.yaml").is_file());
    assert_eq!(tree.stages.keys().collect::<Vec<_>>(), ["tracker"]);
    assert!(
        shown.contains(&format!("программа: {}", path.display())),
        "{shown}"
    );
    assert!(shown.contains("этап: tracker · "), "{shown}");
    assert!(
        shown.contains(&format!("дальше: tolearn next {}", desk.out())),
        "{shown}"
    );
    assert_eq!(model.heard().len(), 3);
    let cache = desk.out.join("cache").join(&tree.program.uuid);
    assert!(!cache.join("build").exists());
    assert!(cache.join("ledger.jsonl").is_file());
}

#[test]
fn каждый_шаг_печатает_время_а_шаги_модели_ещё_и_токены() {
    let desk = Desk::new(&told(flat()));

    desk.begun();

    assert_eq!(
        desk.log.names(),
        ["карта", "источники", "текст", "схемы", "запись"]
    );
    for name in ["карта", "источники", "текст"] {
        let line = desk.log.line(name);
        assert!(line.ends_with(" с · 7 → 11 ток."), "{line}");
    }
    let written = desk.log.line("запись");
    let secs = written
        .strip_prefix("запись · ")
        .and_then(|rest| rest.strip_suffix(" с"))
        .unwrap_or_else(|| panic!("{written}"));
    assert!(secs.parse::<f64>().is_ok(), "{written}");
}

#[test]
fn большие_числа_токенов_разбиты_на_разряды() {
    let desk = Desk::new(&said((3104, 1870), flat()));

    let shown = desk.begun();

    let plan = desk.log.line("карта");
    assert!(
        plan.ends_with(" с · 3\u{a0}104 → 1\u{a0}870 ток."),
        "{plan}"
    );
    assert!(
        shown.contains(" с · 9\u{a0}312 → 5\u{a0}610 ток."),
        "{shown}"
    );
}

#[test]
fn без_окна_схема_остаётся_исходником_mermaid_и_вывод_это_говорит() {
    let desk = Desk::new(&told(flat()));

    desk.begun();

    let diagrams = desk.log.line("схемы");
    assert!(
        diagrams.ends_with(" с · схемы остались исходником Mermaid: окна для схем нет"),
        "{diagrams}"
    );
    let stage = &desk.program().stages["tracker"];
    let code = stage
        .blocks
        .iter()
        .find(|block| block.lang.as_deref() == Some("mermaid"))
        .unwrap();
    assert_eq!(code.kind, Kind::Code);
    assert!(code.text.starts_with("flowchart LR"), "{}", code.text);
    assert!(stage.blocks.iter().all(|block| block.kind != Kind::Diagram));
}

#[test]
fn json_отдаёт_итог_объектом() {
    let desk = Desk::new(&told(flat()));

    let shown = desk
        .run(&[
            "new",
            REQUEST,
            "--level",
            LEVEL,
            "--out",
            desk.out(),
            "--json",
        ])
        .unwrap();

    let summary: Value = serde_json::from_str(&shown).unwrap();
    let tree = desk.program();
    let uuid = &tree.program.uuid;
    assert_eq!(summary["program"], json!(uuid));
    assert_eq!(summary["node"], json!(uuid));
    assert_eq!(summary["path"], json!(desk.path().display().to_string()));
    assert_eq!(summary["stage"], json!("tracker"));
    assert_eq!(summary["title"], json!(tree.program.map.stages[0].title));
    assert_eq!(summary["calls"], json!(3));
    assert_eq!(summary["input"], json!(21));
    assert_eq!(summary["output"], json!(33));
    assert_eq!(summary["unknown"], json!(0));
    assert_eq!(summary["mermaid"], json!(1));
    assert!(summary["ms"].is_u64(), "{summary}");
}

#[test]
fn созданная_программа_проходит_pack_и_импорт() {
    let desk = Desk::new(&told(flat()));
    desk.begun();
    let tree = desk.program();
    let file = desk.root.join("chiptune.tolearn");

    desk.run(&[
        "pack",
        desk.path().to_str().unwrap(),
        file.to_str().unwrap(),
    ])
    .unwrap();
    let shelf = Library::at(&desk.root.join("shelf"));
    let imported = package::import(&file, &shelf).unwrap();

    assert_eq!(imported.uuid, tree.program.uuid);
    assert_eq!(imported.copy_of, None);
    assert_eq!(shelf.open(&imported.uuid).unwrap().stages, tree.stages);
}

#[test]
fn провайдер_берётся_из_настроек_приложения_а_флаг_provider_его_подменяет() {
    let kept = told(flat());
    let other = told(flat());
    let desk = Desk::new(&kept);
    let file = desk.root.join("other.yaml");
    provided(&other.endpoint).save(&file).unwrap();

    desk.run(&[
        "new",
        REQUEST,
        "--level",
        LEVEL,
        "--out",
        desk.out(),
        "--provider",
        file.to_str().unwrap(),
    ])
    .unwrap();

    assert!(kept.heard().is_empty());
    assert_eq!(other.heard().len(), 3);
}
