#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "provider gate: a panic here is the report"
)]

mod support;

use std::time::Instant;

use support::harness;
use tolearn_provider::{CheckError, ask};

fn asked(mode: &str, timeout_secs: u32) -> Result<String, CheckError> {
    ask(&harness(&[mode.to_owned()], timeout_secs), None, "спроси")
}

#[test]
fn харнесс_запускается_в_пустом_каталоге() {
    let answer = asked("cwd", 20).expect("харнесс отвечает");

    assert!(answer.starts_with("0 файлов "), "{answer}");
    assert!(!answer.contains("Cargo.toml"), "{answer}");
}

#[test]
fn аргументы_доходят_до_команды() {
    let provider = harness(&["args".to_owned(), "--no-tools".to_owned()], 20);

    let answer = ask(&provider, None, "спроси").expect("харнесс отвечает");

    assert_eq!(answer, "--no-tools");
}

#[test]
fn цветной_вывод_очищается_от_управляющих_последовательностей() {
    let answer = asked("ansi", 20).expect("харнесс отвечает");

    assert_eq!(answer, "услышал: спроси");
}

#[test]
fn баннер_остаётся_в_ответе() {
    let answer = asked("banner", 20).expect("харнесс отвечает");

    assert!(answer.starts_with("добро пожаловать"), "{answer}");
    assert!(answer.ends_with("услышал: спроси"), "{answer}");
}

#[test]
fn пустой_вывод_при_нулевом_коде_отвергается() {
    assert_eq!(asked("silent", 20), Err(CheckError::BadAnswer));
}

#[test]
fn ненулевой_код_приносит_жалобу_из_stderr() {
    let failed = asked("fail", 20).expect_err("харнесс падает");

    assert_eq!(failed.code(), "harness.failed");
    let said = failed.to_string();
    assert!(said.contains("не залогинен"), "{said}");
    assert!(said.contains("кодом 3"), "{said}");
    assert!(!said.contains("строка четыре"), "{said}");
}

#[test]
fn слишком_длинный_вывод_не_идёт_в_разбор() {
    let failed = asked("flood", 60).expect_err("вывод превысил потолок");

    assert_eq!(failed.code(), "harness.truncated");
}

#[test]
fn зависший_харнесс_убивается_по_таймауту() {
    let started = Instant::now();

    let failed = asked("hang", 1).expect_err("харнесс висит");

    assert_eq!(failed.code(), "harness.timeout");
    assert!(started.elapsed().as_secs() < 10, "{:?}", started.elapsed());
    assert!(failed.to_string().contains('1'), "{failed}");
}

#[test]
fn пустая_команда_не_запускает_ничего() {
    let mut provider = harness(&["say".to_owned()], 20);
    provider.harness.command = "   ".to_owned();

    let failed = ask(&provider, None, "спроси").expect_err("команды нет");

    assert_eq!(failed.code(), "harness.not-found");
}
