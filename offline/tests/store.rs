#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "offline gate: a panic here is the report"
)]

use std::path::{Path, PathBuf};

use tolearn_offline::store::{Fetched, Store};

fn root(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("tolearn-store-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&path);
    path
}

fn fetched(bytes: &[u8]) -> Fetched<'_> {
    Fetched {
        kind: "article",
        bytes,
        etag: None,
        last_modified: None,
    }
}

fn objects(root: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut stack = vec![root.join("objects")];
    while let Some(directory) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                found.push(path);
            }
        }
    }
    found
}

#[test]
fn один_url_из_двух_программ_лежит_одним_объектом() {
    let root = root("shared");
    let mut store = Store::open(&root, 1024 * 1024).unwrap();
    let url = "https://example.test/rfc";

    let first = store
        .put(url, "llm-agents", &fetched(b"one and the same"), 10)
        .unwrap();
    let second = store
        .put(url, "rust-core", &fetched(b"one and the same"), 20)
        .unwrap();

    assert_eq!(first.hash, second.hash);
    assert_eq!(objects(&root).len(), 1, "объект записан дважды");
    assert_eq!(store.size().unwrap(), 16);
}

#[test]
fn разные_адреса_с_одинаковым_содержимым_делят_объект() {
    let root = root("dedup");
    let mut store = Store::open(&root, 1024 * 1024).unwrap();

    store
        .put("https://a.test/x", "p", &fetched(b"same bytes"), 10)
        .unwrap();
    store
        .put("https://b.test/y", "p", &fetched(b"same bytes"), 11)
        .unwrap();

    assert_eq!(objects(&root).len(), 1);
    assert_eq!(store.size().unwrap(), 10);
}

#[test]
fn содержимое_возвращается_по_адресу_вместе_с_валидаторами() {
    let root = root("read");
    let mut store = Store::open(&root, 1024 * 1024).unwrap();
    let saved = Fetched {
        kind: "docs",
        bytes: b"body",
        etag: Some("\"v1\""),
        last_modified: Some("Mon, 27 Jul 2026 10:00:00 GMT"),
    };
    store.put("https://a.test/x", "p", &saved, 10).unwrap();

    let held = store.get("https://a.test/x", 20).unwrap().unwrap();

    assert_eq!(held.kind, "docs");
    assert_eq!(held.etag.as_deref(), Some("\"v1\""));
    assert_eq!(
        held.last_modified.as_deref(),
        Some("Mon, 27 Jul 2026 10:00:00 GMT")
    );
    assert_eq!(std::fs::read(&held.path).unwrap(), b"body");
    assert!(store.get("https://a.test/missing", 20).unwrap().is_none());
}

#[test]
fn повторная_загрузка_обновляет_валидаторы_и_не_плодит_объекты() {
    let root = root("revalidate");
    let mut store = Store::open(&root, 1024 * 1024).unwrap();
    store
        .put("https://a.test/x", "p", &fetched(b"first"), 10)
        .unwrap();

    let next = Fetched {
        kind: "article",
        bytes: b"second",
        etag: Some("\"v2\""),
        last_modified: None,
    };
    store.put("https://a.test/x", "p", &next, 20).unwrap();

    let held = store.get("https://a.test/x", 30).unwrap().unwrap();
    assert_eq!(std::fs::read(&held.path).unwrap(), b"second");
    assert_eq!(held.etag.as_deref(), Some("\"v2\""));
    assert_eq!(objects(&root).len(), 1, "старый объект остался лежать");
}

#[test]
fn вытеснение_сносит_давнее_и_укладывает_в_бюджет() {
    let root = root("lru");
    let mut store = Store::open(&root, 20).unwrap();
    store
        .put("https://a.test/old", "p", &fetched(b"0123456789"), 10)
        .unwrap();
    store
        .put("https://a.test/new", "p", &fetched(b"abcdefghij"), 20)
        .unwrap();
    store.get("https://a.test/old", 30).unwrap();
    store
        .put("https://a.test/third", "p", &fetched(b"klmnopqrst"), 40)
        .unwrap();

    let evicted = store.sweep().unwrap();

    assert_eq!(
        evicted,
        vec!["https://a.test/new".to_string()],
        "вытеснено не самое давнее"
    );
    assert!(
        store.size().unwrap() <= 20,
        "бюджет превышен после вытеснения"
    );
    assert!(store.get("https://a.test/new", 50).unwrap().is_none());
    assert!(store.get("https://a.test/old", 50).unwrap().is_some());
    assert_eq!(
        objects(&root).len(),
        2,
        "файл вытесненного объекта остался на диске"
    );
}

#[test]
fn объект_защищённой_программы_не_вытесняется() {
    let root = root("protected");
    let mut store = Store::open(&root, 10).unwrap();
    store
        .put(
            "https://a.test/kept",
            "offline-on",
            &fetched(b"0123456789"),
            10,
        )
        .unwrap();
    store
        .put(
            "https://a.test/loose",
            "offline-off",
            &fetched(b"abcdefghij"),
            20,
        )
        .unwrap();
    store.protect("offline-on", true).unwrap();

    let evicted = store.sweep().unwrap();

    assert_eq!(evicted, vec!["https://a.test/loose".to_string()]);
    assert!(store.get("https://a.test/kept", 30).unwrap().is_some());
}

#[test]
fn под_бюджетом_не_вытесняется_ничего() {
    let root = root("under");
    let mut store = Store::open(&root, 1024).unwrap();
    store
        .put("https://a.test/x", "p", &fetched(b"small"), 10)
        .unwrap();

    assert!(store.sweep().unwrap().is_empty());
    assert!(store.get("https://a.test/x", 20).unwrap().is_some());
}

#[test]
fn индекс_переживает_аварийное_завершение() {
    let root = root("crash");
    {
        let mut store = Store::open(&root, 1024 * 1024).unwrap();
        store
            .put("https://a.test/x", "p", &fetched(b"survives"), 10)
            .unwrap();
        store.protect("p", true).unwrap();
        std::mem::forget(store);
    }

    let mut reopened = Store::open(&root, 1024 * 1024).unwrap();
    let held = reopened.get("https://a.test/x", 20).unwrap().unwrap();

    assert_eq!(std::fs::read(&held.path).unwrap(), b"survives");
    assert!(
        reopened.sweep().unwrap().is_empty(),
        "защита программы не пережила перезапуск"
    );
}
