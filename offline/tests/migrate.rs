#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "offline gate: a panic here is the report"
)]

use std::path::{Path, PathBuf};

use tolearn_offline::store::{Checked, Fetched, Store};

fn root(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("tolearn-migrate-{name}-{}", std::process::id()));
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

fn checked(hash: &str, at: i64) -> Checked {
    Checked {
        body_hash: Some(hash.to_string()),
        etag: None,
        last_modified: None,
        at,
    }
}

#[test]
fn отметка_на_чужой_адрес_молча_не_пропадает() {
    let root = root("absent");
    let mut store = Store::open(&root, 1024 * 1024).unwrap();

    let stamped = store.stamp("https://a.test/нет", &checked("hash-1", 7));
    let noted = store.checked("https://a.test/нет", 7);

    assert!(stamped.is_err(), "штамп лёг на несуществующий адрес");
    assert!(noted.is_err(), "момент лёг на несуществующий адрес");
}

#[test]
fn момент_проверки_ложится_отдельно_от_валидаторов() {
    let root = root("noted");
    let mut store = Store::open(&root, 1024 * 1024).unwrap();
    store
        .put("https://a.test/y", "p", &fetched(b"body"), 10)
        .unwrap();
    store
        .stamp("https://a.test/y", &checked("hash-1", 42))
        .unwrap();

    store.checked("https://a.test/y", 100).unwrap();

    let held = store.held("https://a.test/y").unwrap().unwrap();
    assert_eq!(held.checked_at, Some(100));
    assert_eq!(
        held.body_hash.as_deref(),
        Some("hash-1"),
        "момент затёр хэш тела"
    );
}

#[test]
fn отметка_о_проверке_ложится_на_адрес_и_читается_обратно() {
    let root = root("stamp");
    let mut store = Store::open(&root, 1024 * 1024).unwrap();
    store
        .put("https://a.test/x", "p", &fetched(b"body"), 10)
        .unwrap();

    let mark = Checked {
        body_hash: Some("hash-1".to_string()),
        etag: Some("\"v9\"".to_string()),
        last_modified: None,
        at: 42,
    };
    store.stamp("https://a.test/x", &mark).unwrap();

    let held = store.get("https://a.test/x", 50).unwrap().unwrap();
    assert_eq!(held.body_hash.as_deref(), Some("hash-1"));
    assert_eq!(held.checked_at, Some(42));
    assert_eq!(held.etag.as_deref(), Some("\"v9\""));
}

#[test]
fn перекачка_не_роняет_отметку_о_проверке() {
    let root = root("restamp");
    let mut store = Store::open(&root, 1024 * 1024).unwrap();
    store
        .put("https://a.test/x", "p", &fetched(b"first"), 10)
        .unwrap();
    store
        .stamp("https://a.test/x", &checked("hash-1", 42))
        .unwrap();

    store
        .put("https://a.test/x", "p", &fetched(b"second"), 60)
        .unwrap();

    let held = store.get("https://a.test/x", 70).unwrap().unwrap();
    assert_eq!(held.checked_at, Some(42), "отметка стёрта перезаписью");
    assert_eq!(held.body_hash.as_deref(), Some("hash-1"));
}

#[test]
fn база_предыдущей_версии_открывается_и_работает() {
    let root = root("aged");
    std::fs::create_dir_all(&root).unwrap();
    aged(&root.join("index.sqlite"));

    let mut store = Store::open(&root, 1024 * 1024).unwrap();

    let held = store.get("https://a.test/old", 20).unwrap().unwrap();
    assert_eq!(held.kind, "article");
    assert_eq!(held.etag.as_deref(), Some("\"v1\""));
    assert_eq!(held.body_hash, None, "у старой записи взялся хэш ниоткуда");
    assert_eq!(held.checked_at, None);

    store
        .stamp("https://a.test/old", &checked("hash-1", 99))
        .unwrap();
    store
        .put("https://a.test/new", "p", &fetched(b"fresh"), 30)
        .unwrap();

    assert_eq!(
        store
            .get("https://a.test/old", 40)
            .unwrap()
            .unwrap()
            .checked_at,
        Some(99)
    );
    assert!(store.get("https://a.test/new", 40).unwrap().is_some());
}

#[test]
fn повторное_открытие_старую_базу_не_ломает() {
    let root = root("twice");
    std::fs::create_dir_all(&root).unwrap();
    aged(&root.join("index.sqlite"));

    Store::open(&root, 1024 * 1024).unwrap();
    let mut store = Store::open(&root, 1024 * 1024).unwrap();

    store
        .stamp("https://a.test/old", &checked("hash-2", 7))
        .unwrap();
    assert_eq!(
        store
            .get("https://a.test/old", 40)
            .unwrap()
            .unwrap()
            .checked_at,
        Some(7)
    );
}

fn aged(path: &Path) {
    let db = rusqlite::Connection::open(path).unwrap();
    db.execute_batch(
        "create table objects (hash text primary key, size integer not null, used_at integer not null);
         create table urls (
             url text primary key,
             hash text not null,
             kind text not null,
             fetched_at integer not null,
             etag text,
             last_modified text
         );
         create table holders (program text not null, url text not null, primary key (program, url));
         create table programs (program text primary key, protected integer not null);
         insert into objects (hash, size, used_at) values ('abcdef', 5, 10);
         insert into urls (url, hash, kind, fetched_at, etag, last_modified)
             values ('https://a.test/old', 'abcdef', 'article', 10, '\"v1\"', null);
         insert into holders (program, url) values ('p', 'https://a.test/old');",
    )
    .unwrap();
}
