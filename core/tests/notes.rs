#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "core gate: a panic here is the report"
)]

use std::path::{Path, PathBuf};

use tolearn_core::notes::{NoteError, index, read, save};

fn directory(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("tolearn-notes-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    root
}

fn put(root: &Path, name: &str, text: &str) -> PathBuf {
    let file = root.join(name);
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(&file, text).unwrap();
    file
}

const NOTE: &str =
    "---\ntolearn:\n  roadmap: llm-agents-base\n  topic: local-runtime\n---\n\nЗаметки\n";

#[test]
fn привязку_держит_фронтматтер_а_не_путь() {
    let root = directory("bound");
    put(&root, "своя-папка/как-угодно-названный.md", NOTE);

    let note = read(&root, "llm-agents-base", "local-runtime")
        .unwrap()
        .unwrap();

    assert_eq!(note.body.trim(), "Заметки");
    assert!(note.path.ends_with("как-угодно-названный.md"));
}

#[test]
fn чужая_программа_не_подхватывается() {
    let root = directory("foreign");
    put(
        &root,
        "чужой.md",
        &NOTE.replace("llm-agents-base", "другая-программа"),
    );

    assert!(
        read(&root, "llm-agents-base", "local-runtime")
            .unwrap()
            .is_none()
    );
}

#[test]
fn файл_без_фронтматтера_не_считается_конспектом() {
    let root = directory("plain");
    put(&root, "просто.md", "# Просто заметка\n");

    assert!(index(&root).unwrap().is_empty());
}

#[test]
fn новый_конспект_пишется_с_фронтматтером_и_читается_обратно() {
    let root = directory("create");

    let stamp = save(
        &root,
        "llm-agents-base",
        "local-runtime",
        "Первый текст",
        None,
    )
    .unwrap();
    let note = read(&root, "llm-agents-base", "local-runtime")
        .unwrap()
        .unwrap();

    assert_eq!(note.body.trim(), "Первый текст");
    assert_eq!(note.stamp, stamp);
    let raw = std::fs::read_to_string(&note.path).unwrap();
    assert!(raw.starts_with("---\n"), "{raw}");
    assert!(raw.contains("roadmap: llm-agents-base"), "{raw}");
    assert!(raw.contains("topic: local-runtime"), "{raw}");
}

#[test]
fn правка_снаружи_подхватывается_при_следующем_чтении() {
    let root = directory("outside");
    save(
        &root,
        "llm-agents-base",
        "local-runtime",
        "Первый текст",
        None,
    )
    .unwrap();
    let note = read(&root, "llm-agents-base", "local-runtime")
        .unwrap()
        .unwrap();

    std::fs::write(&note.path, NOTE.replace("Заметки", "Правка снаружи")).unwrap();

    let again = read(&root, "llm-agents-base", "local-runtime")
        .unwrap()
        .unwrap();
    assert_eq!(again.body.trim(), "Правка снаружи");
}

#[test]
fn одновременная_правка_не_сливается_молча() {
    let root = directory("conflict");
    let stamp = save(
        &root,
        "llm-agents-base",
        "local-runtime",
        "Первый текст",
        None,
    )
    .unwrap();
    let note = read(&root, "llm-agents-base", "local-runtime")
        .unwrap()
        .unwrap();
    std::fs::write(&note.path, NOTE.replace("Заметки", "Версия снаружи")).unwrap();

    let refused = save(
        &root,
        "llm-agents-base",
        "local-runtime",
        "Версия из приложения",
        Some(stamp),
    );

    match refused {
        Err(NoteError::Conflict { theirs, ours }) => {
            assert_eq!(theirs.trim(), "Версия снаружи");
            assert_eq!(ours, "Версия из приложения");
        }
        other => panic!("конфликт не пойман: {other:?}"),
    }
    let kept = std::fs::read_to_string(&note.path).unwrap();
    assert!(
        kept.contains("Версия снаружи"),
        "чужой текст затёрт: {kept}"
    );
}

#[test]
fn запись_поверх_своего_же_слепка_проходит() {
    let root = directory("same");
    let stamp = save(
        &root,
        "llm-agents-base",
        "local-runtime",
        "Первый текст",
        None,
    )
    .unwrap();

    let next = save(
        &root,
        "llm-agents-base",
        "local-runtime",
        "Второй текст",
        Some(stamp),
    )
    .unwrap();

    assert_ne!(next, stamp);
    let note = read(&root, "llm-agents-base", "local-runtime")
        .unwrap()
        .unwrap();
    assert_eq!(note.body.trim(), "Второй текст");
}

#[test]
fn удаление_снаружи_не_ошибка_а_отсутствие_конспекта() {
    let root = directory("deleted");
    save(
        &root,
        "llm-agents-base",
        "local-runtime",
        "Первый текст",
        None,
    )
    .unwrap();
    let note = read(&root, "llm-agents-base", "local-runtime")
        .unwrap()
        .unwrap();
    std::fs::remove_file(&note.path).unwrap();

    assert!(
        read(&root, "llm-agents-base", "local-runtime")
            .unwrap()
            .is_none()
    );
}

#[test]
fn осиротевший_конспект_остаётся_в_каталоге() {
    let root = directory("orphan");
    put(
        &root,
        "сирота.md",
        &NOTE.replace("local-runtime", "темы-больше-нет"),
    );

    let found = index(&root).unwrap();

    assert!(
        found
            .iter()
            .any(|note| note.topic == "темы-больше-нет" && note.roadmap == "llm-agents-base"),
        "{found:#?}"
    );
}

#[test]
fn отсутствующий_каталог_читается_как_пустой() {
    let root = std::env::temp_dir().join("tolearn-notes-нет-такого-каталога");
    let _ = std::fs::remove_dir_all(&root);

    assert!(index(&root).unwrap().is_empty());
    assert!(
        read(&root, "llm-agents-base", "local-runtime")
            .unwrap()
            .is_none()
    );
}
