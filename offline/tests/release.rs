#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "offline gate: a panic here is the report"
)]

#[path = "../../tests-support/scratch.rs"]
mod scratch;

use std::path::PathBuf;

use tolearn_offline::store::{Fetched, Store};

fn root(name: &str) -> PathBuf {
    scratch::named(&format!("release-{name}"))
}

fn fetched(bytes: &[u8]) -> Fetched<'_> {
    Fetched {
        kind: "article",
        bytes,
        etag: None,
        last_modified: None,
    }
}

#[test]
fn отпущенное_программой_вытесняется_хотя_программа_была_защищена() {
    let root = root("protected");
    let mut store = Store::open(&root, 10).unwrap();
    store
        .put("https://a.test/kept", "gone", &fetched(b"0123456789"), 10)
        .unwrap();
    store
        .put("https://a.test/other", "stays", &fetched(b"abcdefghij"), 20)
        .unwrap();
    store.protect("gone", true).unwrap();
    store.protect("stays", true).unwrap();
    assert!(store.sweep().unwrap().is_empty());

    store.release("gone").unwrap();

    assert_eq!(
        store.sweep().unwrap(),
        vec!["https://a.test/kept".to_string()]
    );
    assert!(store.get("https://a.test/other", 30).unwrap().is_some());
}

#[test]
fn чужое_держит_объект_даже_после_ухода_одной_программы() {
    let root = root("shared");
    let mut store = Store::open(&root, 0).unwrap();
    let url = "https://a.test/shared";
    store.put(url, "gone", &fetched(b"0123456789"), 10).unwrap();
    store
        .put(url, "stays", &fetched(b"0123456789"), 20)
        .unwrap();
    store.protect("stays", true).unwrap();

    store.release("gone").unwrap();

    assert!(store.sweep().unwrap().is_empty());
    assert!(store.get(url, 30).unwrap().is_some());
}

#[test]
fn незнакомая_программа_отпускается_без_ошибки() {
    let root = root("stranger");
    let mut store = Store::open(&root, 1024).unwrap();
    store
        .put("https://a.test/x", "known", &fetched(b"small"), 10)
        .unwrap();

    store
        .release("00000000-0000-4000-8000-000000000000")
        .unwrap();

    assert!(store.get("https://a.test/x", 20).unwrap().is_some());
}

#[test]
fn отпущенное_не_возвращается_после_переоткрытия() {
    let root = root("reopen");
    let mut store = Store::open(&root, 1024).unwrap();
    store
        .put("https://a.test/x", "gone", &fetched(b"0123456789"), 10)
        .unwrap();
    store.protect("gone", true).unwrap();
    store.release("gone").unwrap();
    drop(store);

    let mut store = Store::open(&root, 1).unwrap();

    assert_eq!(store.sweep().unwrap(), vec!["https://a.test/x".to_string()]);
}

#[test]
fn вернувшийся_uuid_не_воскрешает_старые_привязки() {
    let root = root("revived");
    let mut store = Store::open(&root, 1).unwrap();
    store
        .put("https://a.test/old", "same", &fetched(b"0123456789"), 10)
        .unwrap();
    store.protect("same", true).unwrap();
    store.release("same").unwrap();

    store.protect("same", true).unwrap();

    assert_eq!(
        store.sweep().unwrap(),
        vec!["https://a.test/old".to_string()]
    );
}
