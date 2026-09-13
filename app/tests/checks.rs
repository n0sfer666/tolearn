#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "app gate: a panic here is the report"
)]

mod support;

use std::path::PathBuf;

use serde_json::{Value, json};
use support::shelf::{CHIPTUNE, NES_DEV, ROM, Shelf, TOOLS};
use tolearn_core::state::State;

const FIRST_ROM: &str = "fixtures/v2/valid/nes-dev";

fn check(shelf: &Shelf, claim: &str) -> Result<Value, tolearn_app::ipc::IpcError> {
    shelf.ask(
        "check_claim",
        json!({ "program": NES_DEV, "node": ROM, "stage": "first-rom", "claim": claim }),
    )
}

fn rewrite(shelf: &Shelf, command: &str) {
    let file = shelf
        .data
        .join("programs")
        .join(NES_DEV)
        .join("children")
        .join(TOOLS)
        .join("children")
        .join(ROM)
        .join("stages/first-rom.yaml");
    let source = std::fs::read_to_string(&file).unwrap();
    let replaced = source.replace("check: ca65 --version", &format!("check: {command:?}"));
    assert_ne!(source, replaced, "the fixture lost its check");
    std::fs::write(&file, replaced).unwrap();
}

fn folder(shelf: &Shelf, name: &str) -> PathBuf {
    let path = shelf.incoming.join(name);
    std::fs::create_dir_all(&path).unwrap();
    path
}

fn choose(shelf: &Shelf, path: &std::path::Path) -> Result<Value, tolearn_app::ipc::IpcError> {
    shelf.ask(
        "workdir",
        json!({ "program": NES_DEV, "path": path.display().to_string() }),
    )
}

#[test]
fn без_папки_проверка_не_запускается_а_пункт_без_команды_не_запускается_вовсе() {
    let shelf = Shelf::new("practice-unset");
    shelf.shelved(FIRST_ROM);
    let stage = shelf
        .ask(
            "stage",
            json!({ "program": NES_DEV, "node": ROM, "stage": "first-rom" }),
        )
        .unwrap();
    assert_eq!(stage["workdir"], Value::Null);

    assert_eq!(check(&shelf, "c1").unwrap_err().code, "workdir.unset");

    let chip = Shelf::new("practice-plain");
    chip.shelved("examples/chiptune");
    let plain = chip
        .ask(
            "check_claim",
            json!({ "program": CHIPTUNE, "node": "", "stage": "voices", "claim": "c1" }),
        )
        .unwrap_err();
    assert_eq!(plain.code, "claim.unchecked");
}

#[test]
fn проверка_идёт_в_выбранной_папке_и_отдаёт_вывод_и_код() {
    let shelf = Shelf::new("practice-run");
    shelf.shelved(FIRST_ROM);
    rewrite(&shelf, "pwd; echo said; echo moaned >&2; exit 3");
    let place = folder(&shelf, "practice-dir");

    let chosen = choose(&shelf, &place).unwrap();
    assert_eq!(chosen["workdir"], place.display().to_string());
    assert_eq!(
        State::read(&shelf.data, NES_DEV).unwrap().workdir,
        Some(place.display().to_string())
    );

    let ran = check(&shelf, "c1").unwrap();

    assert_eq!(ran["outcome"], "finished");
    assert_eq!(ran["code"], 3);
    let stdout = ran["stdout"].as_str().unwrap();
    assert!(stdout.contains("practice-dir"), "{stdout}");
    assert!(stdout.contains("said"), "{stdout}");
    assert!(ran["stderr"].as_str().unwrap().contains("moaned"));
    assert_eq!(ran["truncated"], false);
    let stage = shelf
        .ask(
            "stage",
            json!({ "program": NES_DEV, "node": ROM, "stage": "first-rom" }),
        )
        .unwrap();
    assert_eq!(stage["workdir"], place.display().to_string());
}

#[test]
fn длинный_вывод_упирается_в_лимит_и_это_видно() {
    let shelf = Shelf::new("practice-flood");
    shelf.shelved(FIRST_ROM);
    rewrite(&shelf, "head -c 400000 /dev/zero | tr '\\0' x");
    choose(&shelf, &folder(&shelf, "flood")).unwrap();

    let ran = check(&shelf, "c1").unwrap();

    assert_eq!(ran["outcome"], "finished");
    assert_eq!(ran["truncated"], true);
    assert!(ran["stdout"].as_str().unwrap().len() < 400_000);
}

#[test]
fn папка_практики_должна_существовать_и_лежать_вне_данных() {
    let shelf = Shelf::new("practice-folder");
    shelf.shelved(FIRST_ROM);

    let missing = choose(&shelf, &shelf.incoming.join("nowhere")).unwrap_err();
    assert_eq!(missing.code, "workdir.absent");
    let inside = choose(&shelf, &shelf.data.join("programs")).unwrap_err();
    assert_eq!(inside.code, "workdir.inside");
    let relative = shelf
        .ask("workdir", json!({ "program": NES_DEV, "path": "incoming" }))
        .unwrap_err();
    assert_eq!(relative.code, "workdir.absent");

    assert_eq!(State::read(&shelf.data, NES_DEV).unwrap().workdir, None);
}

#[test]
fn исчезнувшая_папка_отказывает_при_запуске() {
    let shelf = Shelf::new("practice-gone");
    shelf.shelved(FIRST_ROM);
    let place = folder(&shelf, "gone");
    choose(&shelf, &place).unwrap();
    std::fs::remove_dir(&place).unwrap();

    assert_eq!(check(&shelf, "c1").unwrap_err().code, "workdir.absent");
}
