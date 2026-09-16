#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "app gate: a panic here is the report"
)]

mod support;

use std::path::Path;
use std::sync::{Arc, Mutex};

use serde_json::{Value, json};
use support::bucket::Bucket;
use support::shelf::{NES_DEV, Shelf, TOOLS};
use tolearn_app::discard::Bin;
use tolearn_app::ipc::{IpcError, Running};
use tolearn_app::journal::{self, ROOM};
use tolearn_core::library::PROGRAMS;
use tolearn_core::state::STATE;
use tolearn_generate::sources::CACHE;

const FIRST_ROM: &str = "fixtures/v2/valid/nes-dev";
const TITLE: &str = "Разработка игр для NES";
const OTHER: &str = "9f1c0d33-4b7a-4e2f-8c15-6a3d9e0b7f42";
const BINNED_LOG: &str = "data-llm-log";

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

fn remove(shelf: &Shelf, program: &str) -> Result<Value, IpcError> {
    shelf.ask("delete_program", json!({ "program": program }))
}

fn kept(shelf: &Shelf, room: &str, uuid: &str) -> bool {
    shelf.data.join(room).join(uuid).exists()
}

#[derive(Debug)]
struct Racer {
    bin: Bucket,
    running: Running,
    tried: Arc<Mutex<Vec<String>>>,
}

impl Bin for Racer {
    fn discard(&self, path: &Path) -> Result<(), String> {
        let tried = match self.running.claim(Some(NES_DEV)) {
            Ok(_) => "генерация началась".to_owned(),
            Err(refusal) => refusal.code,
        };
        self.tried.lock().unwrap().push(tried);
        self.bin.discard(path)
    }
}

#[test]
fn удаление_уносит_программу_состояние_и_кэш_в_корзину() {
    let shelf = Shelf::new("delete-whole");
    stocked(&shelf);

    let told = remove(&shelf, NES_DEV).unwrap();

    assert_eq!(told["title"], json!(TITLE));
    assert!(!kept(&shelf, PROGRAMS, NES_DEV));
    for uuid in [NES_DEV, TOOLS] {
        assert!(!kept(&shelf, STATE, uuid));
        assert!(!kept(&shelf, CACHE, uuid));
        assert!(shelf.bucket.holds(&format!("{STATE}-{uuid}")));
        assert!(shelf.bucket.holds(&format!("{CACHE}-{uuid}")));
    }
    assert!(shelf.bucket.holds(&format!("{PROGRAMS}-{NES_DEV}")));
    assert_eq!(shelf.library(), json!({ "programs": [], "refused": [] }));
}

#[test]
fn программа_уходит_в_корзину_раньше_своего_состояния() {
    let shelf = Shelf::new("delete-order");
    stocked(&shelf);

    remove(&shelf, NES_DEV).unwrap();

    let taken = shelf.bucket.taken();
    assert_eq!(
        taken.first().map(String::as_str),
        Some(&*format!("{PROGRAMS}-{NES_DEV}"))
    );
}

#[test]
fn журнал_запросов_уходит_в_корзину_целиком() {
    let shelf = Shelf::new("delete-journal");
    stocked(&shelf);
    assert_eq!(journal::records(&shelf.data.join(ROOM)).len(), 1);

    remove(&shelf, NES_DEV).unwrap();

    assert!(journal::records(&shelf.data.join(ROOM)).is_empty());
    assert!(shelf.bucket.holds(BINNED_LOG));
    assert!(shelf.bucket.taken().last().map(String::as_str) == Some(BINNED_LOG));
}

#[test]
fn подпрограмму_отдельно_удалить_нельзя() {
    let shelf = Shelf::new("delete-child");
    stocked(&shelf);

    let refusal = remove(&shelf, TOOLS).unwrap_err();

    assert_eq!(refusal.code, "delete.subprogram");
    assert!(refusal.message.contains(TITLE));
    assert!(kept(&shelf, PROGRAMS, NES_DEV));
    assert!(shelf.bucket.taken().is_empty());
}

#[test]
fn незнакомую_программу_удалить_нельзя() {
    let shelf = Shelf::new("delete-absent");
    stocked(&shelf);

    let refusal = remove(&shelf, OTHER).unwrap_err();

    assert_eq!(refusal.code, "library.absent");
    assert!(kept(&shelf, PROGRAMS, NES_DEV));
    assert!(shelf.bucket.taken().is_empty());
}

#[test]
fn генерация_этой_же_программы_запрещает_удаление() {
    let running = Running::default();
    let mut shelf = Shelf::new("delete-busy");
    shelf.context = shelf.context.clone().with_running(running.clone());
    stocked(&shelf);
    let _claim = running.claim(Some(NES_DEV)).unwrap();

    let refusal = remove(&shelf, NES_DEV).unwrap_err();

    assert_eq!(refusal.code, "delete.busy");
    assert!(kept(&shelf, PROGRAMS, NES_DEV));
    assert!(kept(&shelf, STATE, NES_DEV));
    assert!(shelf.bucket.taken().is_empty());
}

#[test]
fn генерация_чужой_программы_удалению_не_мешает() {
    let running = Running::default();
    let mut shelf = Shelf::new("delete-busy-elsewhere");
    shelf.context = shelf.context.clone().with_running(running.clone());
    stocked(&shelf);
    let _claim = running.claim(Some(OTHER)).unwrap();

    remove(&shelf, NES_DEV).unwrap();

    assert!(!kept(&shelf, PROGRAMS, NES_DEV));
}

#[test]
fn пока_идёт_удаление_генерация_этой_программы_не_начинается() {
    let running = Running::default();
    let tried = Arc::new(Mutex::new(Vec::new()));
    let mut shelf = Shelf::new("delete-race");
    let racer = Racer {
        bin: shelf.bucket.clone(),
        running: running.clone(),
        tried: Arc::clone(&tried),
    };
    shelf.context = shelf
        .context
        .clone()
        .with_running(running.clone())
        .with_bin(Arc::new(racer));
    stocked(&shelf);

    remove(&shelf, NES_DEV).unwrap();

    let tried = tried.lock().unwrap().clone();
    assert!(!tried.is_empty());
    assert!(
        tried.iter().all(|code| code == "generate.erased"),
        "{tried:?}"
    );
    assert!(running.claim(Some(NES_DEV)).is_ok());
}
