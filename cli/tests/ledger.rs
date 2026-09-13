#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "cli gate: a panic here is the report"
)]

mod generating;

use generating::Desk;
use generating::answers::{flat, forking, told};
use serde_json::Value;

const HEAD: &str = "| этап | шаг | вызовы | проверки | время, с | вход | выход |";

#[test]
fn ledger_сводит_журнал_в_таблицу_по_этапам_и_шагам_без_сети() {
    let model = told(flat());
    let mut desk = Desk::new(&model);
    desk.begun();
    desk.up = false;
    let calls = model.heard().len();

    let shown = desk.run(&["ledger", desk.out()]).unwrap();

    let lines: Vec<&str> = shown.lines().collect();
    assert!(lines.contains(&HEAD), "{shown}");
    for step in ["plan", "sources", "text"] {
        assert!(
            lines
                .iter()
                .any(|line| line.contains(&format!(" | {step} | "))),
            "{shown}"
        );
    }
    assert!(
        lines
            .iter()
            .any(|line| line.starts_with("| tracker | итого | ")),
        "{shown}"
    );
    let total = lines
        .iter()
        .find(|line| line.starts_with("| всё | итого | "))
        .unwrap();
    assert!(
        total.starts_with(&format!("| всё | итого | {calls} | ")),
        "{total}"
    );
    assert!(
        total.ends_with(&format!(" | {} | {} |", 7 * calls, 11 * calls)),
        "{total}"
    );
    assert!(
        lines.contains(&format!("токенов всего: {}", 18 * calls).as_str()),
        "{shown}"
    );
    assert_eq!(model.heard().len(), calls, "сводка звала модель");
}

#[test]
fn ledger_разводит_этапы_а_карту_и_развилку_держит_вне_этапа() {
    let model = told(forking());
    let desk = Desk::new(&model);
    desk.begun();
    desk.run(&["next", desk.out()]).unwrap();
    desk.run(&["next", desk.out(), "--choice", "1"]).unwrap();
    let calls = model.heard().len();

    let shown = desk.run(&["ledger", desk.out()]).unwrap();

    let lines: Vec<&str> = shown.lines().collect();
    for row in [
        "| — | plan | ",
        "| — | fork | ",
        "| — | итого | ",
        "| tracker | text | ",
        "| tracker | итого | ",
        "| voices | text | ",
        "| voices | итого | ",
        &format!("| всё | итого | {calls} | "),
    ] {
        assert!(
            lines.iter().any(|line| line.starts_with(row)),
            "{row}\n{shown}"
        );
    }
}

#[test]
fn ledger_json_отдаёт_строки_и_итог() {
    let model = told(flat());
    let desk = Desk::new(&model);
    desk.begun();
    let calls = model.heard().len();

    let shown = desk.run(&["ledger", desk.out(), "--json"]).unwrap();

    let summary: Value = serde_json::from_str(&shown).unwrap();
    assert_eq!(summary["program"], desk.program().program.uuid.as_str());
    assert_eq!(summary["total"]["calls"], calls);
    assert_eq!(summary["total"]["input"], 7 * calls);
    assert_eq!(summary["total"]["output"], 11 * calls);
    assert_eq!(summary["total"]["unknown"], 0);
    let steps: Vec<&str> = summary["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["step"].as_str().unwrap())
        .collect();
    assert!(
        ["plan", "sources", "text"]
            .iter()
            .all(|step| steps.contains(step)),
        "{steps:?}"
    );
}

#[test]
fn ledger_без_записей_говорит_что_журнал_пуст() {
    let desk = Desk::new(&told(flat()));
    desk.begun();
    let uuid = desk.program().program.uuid;
    std::fs::remove_file(desk.out.join("cache").join(uuid).join("ledger.jsonl")).unwrap();

    let shown = desk.run(&["ledger", desk.out()]).unwrap();

    assert!(shown.starts_with("журнал пуст: "), "{shown}");
}

#[test]
fn ledger_без_программы_отказывает_и_называет_new() {
    let desk = Desk::new(&told(flat()));
    std::fs::create_dir_all(&desk.out).unwrap();

    let refused = desk.run(&["ledger", desk.out()]).unwrap_err();

    assert!(refused.to_string().contains("нет программы"), "{refused}");
}

#[test]
fn ledger_без_каталога_отказывает_с_подсказкой() {
    let desk = Desk::new(&told(flat()));

    let refused = desk.run(&["ledger"]).unwrap_err();

    assert!(
        refused.to_string().contains("`ledger` не назвал каталог"),
        "{refused}"
    );
}
