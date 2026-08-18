#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "sealed gate: a panic here is the report"
)]

use std::path::{Path, PathBuf};

use tolearn_core::notes::{index, save};
use tolearn_core::sealed::{Keys, SealError, Sealed, keys, lock, sealed, settle, unlock};

struct Yard {
    root: PathBuf,
}

impl Drop for Yard {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn yard(name: &str) -> Yard {
    let root = std::env::temp_dir().join(format!("tolearn-sealed-{name}"));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("notes")).unwrap();
    Yard { root }
}

impl Yard {
    fn notes(&self) -> PathBuf {
        self.root.join("notes")
    }
}

fn plain(root: &Path) {
    save(root, "llm", "агенты", "первый абзац", None).unwrap();
    save(root, "llm", "память", "второй абзац", None).unwrap();
}

fn contents(root: &Path) -> Vec<String> {
    let mut found = Vec::new();
    for entry in std::fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        if path.is_file() {
            found.push(String::from_utf8_lossy(&std::fs::read(&path).unwrap()).into_owned());
        }
    }
    found
}

#[test]
fn запертый_каталог_открывается_ключом_устройства() {
    let yard = yard("round-trip");
    plain(&yard.notes());
    let keys = lock(&yard.notes(), "длинная парольная фраза").unwrap();

    let store = Sealed::new(&yard.notes(), keys);
    let kept = store.index().unwrap();
    assert_eq!(kept.len(), 2);
    let note = store.read("llm", "агенты").unwrap().unwrap();
    assert_eq!(note.body.trim(), "первый абзац");
}

#[test]
fn на_диске_текста_конспекта_не_остаётся() {
    let yard = yard("opaque");
    plain(&yard.notes());
    lock(&yard.notes(), "длинная парольная фраза").unwrap();

    assert!(index(&yard.notes()).unwrap().is_empty());
    for text in contents(&yard.notes()) {
        assert!(!text.contains("первый абзац"), "текст виден на диске");
    }
}

#[test]
fn имена_файлов_не_выдают_тему() {
    let yard = yard("names");
    plain(&yard.notes());
    lock(&yard.notes(), "длинная парольная фраза").unwrap();

    for entry in std::fs::read_dir(yard.notes()).unwrap() {
        let name = entry.unwrap().file_name().to_string_lossy().into_owned();
        assert!(!name.contains("агенты") && !name.contains("llm"), "{name}");
    }
}

#[test]
fn парольная_фраза_открывает_хранилище_без_ключа_устройства() {
    let yard = yard("phrase");
    plain(&yard.notes());
    lock(&yard.notes(), "длинная парольная фраза").unwrap();

    let recovered = keys(&yard.notes(), "длинная парольная фраза").unwrap();
    let note = Sealed::new(&yard.notes(), recovered)
        .read("llm", "память")
        .unwrap()
        .unwrap();
    assert_eq!(note.body.trim(), "второй абзац");
}

#[test]
fn чужая_парольная_фраза_хранилище_не_открывает() {
    let yard = yard("wrong-phrase");
    plain(&yard.notes());
    lock(&yard.notes(), "длинная парольная фраза").unwrap();

    let failed = keys(&yard.notes(), "другая фраза").unwrap_err();
    assert!(matches!(failed, SealError::WrongKey), "{failed}");
}

#[test]
fn чужой_ключ_устройства_конспекты_не_читает() {
    let yard = yard("wrong-key");
    plain(&yard.notes());
    lock(&yard.notes(), "длинная парольная фраза").unwrap();

    let failed = Sealed::new(&yard.notes(), Keys::generate())
        .index()
        .unwrap_err();
    assert!(matches!(failed, SealError::WrongKey), "{failed}");
}

#[test]
fn выключение_возвращает_открытые_конспекты() {
    let yard = yard("unlock");
    plain(&yard.notes());
    let keys = lock(&yard.notes(), "длинная парольная фраза").unwrap();
    unlock(&yard.notes(), &keys).unwrap();

    assert!(!sealed(&yard.notes()));
    let kept = index(&yard.notes()).unwrap();
    assert_eq!(kept.len(), 2);
    assert!(kept.iter().any(|note| note.body.trim() == "первый абзац"));
}

#[test]
fn после_переключения_открытая_копия_рядом_не_остаётся() {
    let yard = yard("leftover");
    plain(&yard.notes());
    let retired = yard.root.join("notes.retired");

    let keys = lock(&yard.notes(), "длинная парольная фраза").unwrap();
    assert!(!retired.exists(), "открытая копия осталась рядом");

    unlock(&yard.notes(), &keys).unwrap();
    assert!(!retired.exists(), "запертая копия осталась рядом");
}

#[test]
fn прерванное_включение_открытый_каталог_не_портит() {
    let yard = yard("interrupted");
    plain(&yard.notes());
    let half = yard.root.join("notes.switching");
    std::fs::create_dir_all(&half).unwrap();
    std::fs::write(half.join("мусор.age"), "половина").unwrap();

    settle(&yard.notes()).unwrap();
    assert!(!half.exists());
    assert_eq!(index(&yard.notes()).unwrap().len(), 2);
    assert!(!sealed(&yard.notes()));
}

#[test]
fn прерванная_подмена_каталога_откатывается() {
    let yard = yard("retired");
    plain(&yard.notes());
    let retired = yard.root.join("notes.retired");
    std::fs::rename(yard.notes(), &retired).unwrap();

    settle(&yard.notes()).unwrap();
    assert!(!retired.exists());
    assert_eq!(index(&yard.notes()).unwrap().len(), 2);
}

#[test]
fn запертое_хранилище_принимает_правку_и_отдаёт_её_обратно() {
    let yard = yard("save");
    plain(&yard.notes());
    let keys = lock(&yard.notes(), "длинная парольная фраза").unwrap();

    let store = Sealed::new(&yard.notes(), keys);
    store
        .save("llm", "агенты", "переписанный абзац", None)
        .unwrap();
    assert_eq!(store.index().unwrap().len(), 2);
    let note = store.read("llm", "агенты").unwrap().unwrap();
    assert_eq!(note.body.trim(), "переписанный абзац");
}

#[test]
fn чужая_правка_поверх_ожидаемой_метки_отклоняется() {
    let yard = yard("conflict");
    plain(&yard.notes());
    let keys = lock(&yard.notes(), "длинная парольная фраза").unwrap();

    let store = Sealed::new(&yard.notes(), keys);
    let stale = store.read("llm", "агенты").unwrap().unwrap().stamp;
    store.save("llm", "агенты", "чужая правка", None).unwrap();

    let failed = store
        .save("llm", "агенты", "наша правка", Some(stale))
        .unwrap_err();
    assert!(matches!(failed, SealError::Busy { .. }), "{failed}");
}

#[test]
fn индекс_поиска_ложится_рядом_зашифрованным() {
    let yard = yard("blob");
    let keys = lock(&yard.notes(), "длинная парольная фраза").unwrap();

    let store = Sealed::new(&yard.notes(), keys);
    assert!(store.blob("search").unwrap().is_none());
    store.keep("search", "первый абзац").unwrap();
    assert_eq!(
        store.blob("search").unwrap().as_deref(),
        Some("первый абзац")
    );

    for text in contents(&yard.notes().join("blobs")) {
        assert!(!text.contains("первый абзац"), "индекс виден на диске");
    }
}
