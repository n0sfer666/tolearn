#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "offline gate: a panic here is the report"
)]

use std::path::PathBuf;

use rusqlite::Connection;
use tolearn_offline::store::{Fetched, Store};

const OLD_SCHEMA: &str = "
create table objects (hash text primary key, size integer not null, used_at integer not null);
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
";

fn root(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("tolearn-keep-{name}-{}", std::process::id()));
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

fn cloned(store: &Store, url: &str, bytes: &[u8]) -> PathBuf {
    let path = store.corner(url).unwrap();
    std::fs::create_dir_all(path.join("src")).unwrap();
    std::fs::write(path.join("src").join("main.rs"), bytes).unwrap();
    path
}

#[test]
fn клон_репозитория_учитывается_в_бюджете_и_открывается_по_адресу() {
    let root = root("clone");
    let mut store = Store::open(&root, 1024 * 1024).unwrap();
    let clone = cloned(&store, "https://git.test/llama", b"fn main() {}");

    let kept = store
        .keep("https://git.test/llama", "p", "repo", 10)
        .unwrap();

    assert_eq!(kept.size, 12);
    assert_eq!(store.size().unwrap(), 12, "артефакт мимо бюджета");

    let held = store.get("https://git.test/llama", 20).unwrap().unwrap();
    assert_eq!(held.path, clone);
    assert_eq!(held.kind, "repo");
    assert_eq!(
        std::fs::read(held.path.join("src").join("main.rs")).unwrap(),
        b"fn main() {}"
    );
}

#[test]
fn вытеснение_сносит_каталог_артефакта() {
    let root = root("evict");
    let mut store = Store::open(&root, 10).unwrap();
    let clone = cloned(&store, "https://git.test/big", b"0123456789abcdef");
    store.keep("https://git.test/big", "p", "repo", 10).unwrap();

    let evicted = store.sweep().unwrap();

    assert_eq!(evicted, vec!["https://git.test/big".to_string()]);
    assert!(!clone.exists(), "каталог артефакта остался на диске");
    assert!(store.get("https://git.test/big", 20).unwrap().is_none());
}

#[test]
fn повторное_сохранение_артефакта_не_сносит_его_каталог() {
    let root = root("again");
    let mut store = Store::open(&root, 1024 * 1024).unwrap();
    let clone = cloned(&store, "https://git.test/llama", b"fn main() {}");

    store
        .keep("https://git.test/llama", "p", "repo", 10)
        .unwrap();
    store
        .keep("https://git.test/llama", "p", "repo", 20)
        .unwrap();

    assert!(clone.join("src").join("main.rs").exists());
    assert_eq!(store.size().unwrap(), 12, "артефакт посчитан дважды");
}

#[test]
fn просмотр_без_касания_не_двигает_очередь_вытеснения() {
    let root = root("peek");
    let mut store = Store::open(&root, 20).unwrap();
    store
        .put("https://a.test/old", "p", &fetched(b"0123456789"), 10)
        .unwrap();
    store
        .put("https://a.test/new", "p", &fetched(b"abcdefghij"), 20)
        .unwrap();

    let seen = store.held("https://a.test/old").unwrap().unwrap();
    assert_eq!(seen.kind, "article");
    assert!(store.held("https://a.test/missing").unwrap().is_none());

    store
        .put("https://a.test/third", "p", &fetched(b"klmnopqrst"), 40)
        .unwrap();
    let evicted = store.sweep().unwrap();

    assert_eq!(
        evicted,
        vec!["https://a.test/old".to_string()],
        "просмотр сдвинул очередь вытеснения"
    );
}

#[test]
fn индекс_прошлой_версии_дочитывается_и_принимает_артефакт() {
    let root = root("migrate");
    std::fs::create_dir_all(root.join("objects")).unwrap();
    let old = Connection::open(root.join("index.sqlite")).unwrap();
    old.execute_batch(OLD_SCHEMA).unwrap();
    old.execute(
        "insert into objects (hash, size, used_at) values ('abc', 5, 10)",
        [],
    )
    .unwrap();
    old.execute(
        "insert into urls (url, hash, kind, fetched_at) values ('https://a.test/x', 'abc', 'article', 10)",
        [],
    )
    .unwrap();
    drop(old);

    let mut store = Store::open(&root, 1024 * 1024).unwrap();
    let held = store.get("https://a.test/x", 20).unwrap().unwrap();
    assert_eq!(held.kind, "article");
    assert!(held.path.starts_with(root.join("objects")));

    let clone = cloned(&store, "https://git.test/llama", b"fn main() {}");
    store
        .keep("https://git.test/llama", "p", "repo", 30)
        .unwrap();
    assert_eq!(
        store
            .get("https://git.test/llama", 40)
            .unwrap()
            .unwrap()
            .path,
        clone
    );
}
