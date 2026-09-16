#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "app gate: a panic here is the report"
)]

mod support;

use std::path::Path;

use serde_json::{Value, json};
use support::shelf::{NES_DEV, SOUND, Shelf, TOOLS};
use tolearn_app::ipc::IpcError;
use tolearn_app::journal::ROOM;
use tolearn_core::library::PROGRAMS;
use tolearn_core::state::STATE;
use tolearn_generate::sources::CACHE;
use tolearn_offline::store::{Fetched, Store};

const FIRST_ROM: &str = "fixtures/v2/valid/nes-dev";
const TWIN: &str = "9f3b7c21-4d5e-4a80-b612-7c8d9e0f1a2b";

fn stocked(shelf: &Shelf) {
    shelf.shelved(FIRST_ROM);
    for uuid in [NES_DEV, TOOLS] {
        made(&shelf.data.join(STATE).join(uuid), "state.yaml");
        made(&shelf.data.join(CACHE).join(uuid), "page.html");
    }
    made(&shelf.data.join(ROOM), "0000000001-0001.md");
}

fn made(room: &Path, name: &str) {
    std::fs::create_dir_all(room).unwrap();
    std::fs::write(room.join(name), b"x").unwrap();
}

fn fetched(bytes: &[u8]) -> Fetched<'_> {
    Fetched {
        kind: "article",
        bytes,
        etag: None,
        last_modified: None,
    }
}

fn remove(shelf: &Shelf, program: &str) -> Result<Value, IpcError> {
    shelf.ask("delete_program", json!({ "program": program }))
}

fn kept(shelf: &Shelf, room: &str, uuid: &str) -> bool {
    shelf.data.join(room).join(uuid).exists()
}

#[test]
fn корзина_не_приняла_программу_значит_ничего_не_тронуто() {
    let shelf = Shelf::jammed("delete-jam-programs", PROGRAMS);
    stocked(&shelf);

    let refusal = remove(&shelf, NES_DEV).unwrap_err();

    assert_eq!(refusal.code, "delete.left");
    assert!(kept(&shelf, PROGRAMS, NES_DEV));
    assert!(kept(&shelf, STATE, NES_DEV));
    assert!(kept(&shelf, CACHE, NES_DEV));
    assert!(shelf.bucket.taken().is_empty());
}

#[test]
fn ранний_отказ_перечисляет_всё_что_осталось_на_месте() {
    let shelf = Shelf::jammed("delete-jam-list", PROGRAMS);
    stocked(&shelf);

    let refusal = remove(&shelf, NES_DEV).unwrap_err();

    for room in [
        format!("{PROGRAMS}/{NES_DEV}"),
        format!("{STATE}/{NES_DEV}"),
        format!("{STATE}/{TOOLS}"),
        format!("{CACHE}/{NES_DEV}"),
        format!("{CACHE}/{TOOLS}"),
        ROOM.to_owned(),
    ] {
        assert!(refusal.message.contains(&room), "{}", refusal.message);
    }
}

#[test]
fn отказ_корзины_называет_причину() {
    let shelf = Shelf::jammed("delete-jam-why", PROGRAMS);
    stocked(&shelf);

    let refusal = remove(&shelf, NES_DEV).unwrap_err();

    assert!(
        refusal.message.contains("подставная корзина не принимает"),
        "{}",
        refusal.message
    );
}

#[test]
fn корзина_не_приняла_состояние_значит_отказ_перечисляет_оставшееся() {
    let shelf = Shelf::jammed("delete-jam-state", STATE);
    stocked(&shelf);

    let refusal = remove(&shelf, NES_DEV).unwrap_err();

    assert_eq!(refusal.code, "delete.left");
    assert!(refusal.message.contains(&format!("{STATE}/{NES_DEV}")));
    assert!(refusal.message.contains(&format!("{STATE}/{TOOLS}")));
    assert!(!kept(&shelf, PROGRAMS, NES_DEV));
    assert!(!kept(&shelf, CACHE, NES_DEV));
    assert!(kept(&shelf, STATE, NES_DEV));
}

#[test]
fn удаление_снимает_привязку_программы_в_офлайн_кэше() {
    let shelf = Shelf::new("delete-binding");
    stocked(&shelf);
    let url = "https://a.test/held";
    let cache = shelf.data.join(CACHE);
    {
        let mut store = Store::open(&cache, 0).unwrap();
        store
            .put(url, NES_DEV, &fetched(b"0123456789"), 10)
            .unwrap();
        store.protect(NES_DEV, true).unwrap();
        assert!(store.sweep().unwrap().is_empty());
    }

    remove(&shelf, NES_DEV).unwrap();

    let mut store = Store::open(&cache, 0).unwrap();
    assert_eq!(store.sweep().unwrap(), vec![url.to_owned()]);
}

#[test]
fn недоступный_индекс_кэша_отменяет_удаление_до_корзины() {
    let shelf = Shelf::new("delete-cache-jam");
    stocked(&shelf);
    std::fs::create_dir_all(shelf.data.join(CACHE).join("index.sqlite")).unwrap();

    let refusal = remove(&shelf, NES_DEV).unwrap_err();

    assert_eq!(refusal.code, "delete.cache");
    assert!(kept(&shelf, PROGRAMS, NES_DEV));
    assert!(shelf.bucket.taken().is_empty());
}

#[test]
fn общий_uuid_у_двух_программ_отменяет_удаление() {
    let shelf = Shelf::new("delete-shared");
    stocked(&shelf);
    let shelved = shelf.data.join(PROGRAMS);
    let yaml = std::fs::read_to_string(shelved.join(NES_DEV).join("program.yaml")).unwrap();
    let twin = shelved.join(TWIN);
    std::fs::create_dir_all(&twin).unwrap();
    std::fs::write(twin.join("program.yaml"), yaml.replace(NES_DEV, TWIN)).unwrap();

    let refusal = remove(&shelf, NES_DEV).unwrap_err();

    assert_eq!(refusal.code, "delete.broken");
    assert!(refusal.message.contains(SOUND) || refusal.message.contains(TOOLS));
    assert!(kept(&shelf, PROGRAMS, NES_DEV));
    assert!(shelf.bucket.taken().is_empty());
}
