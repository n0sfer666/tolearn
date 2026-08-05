#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "provider gate: a panic here is the report"
)]

use tolearn_provider::{advised, memory};

#[test]
fn у_модели_есть_оба_имени_короткое_и_репозиторий() {
    let advised = advised(64);

    let first = advised.first().expect("таблица не пуста");
    assert!(!first.id.contains('/'), "{first:?}");
    assert!(first.repo.contains('/'), "{first:?}");
    assert!(first.gigabytes > 0, "{first:?}");
}

#[test]
fn на_маленькой_машине_тяжёлые_модели_помечены() {
    let tight = advised(8);
    let roomy = advised(128);

    assert!(tight.iter().any(|model| model.heavy), "{tight:?}");
    assert!(tight.iter().any(|model| !model.heavy), "{tight:?}");
    assert!(roomy.iter().all(|model| !model.heavy), "{roomy:?}");
}

#[test]
fn неизвестный_объём_памяти_никого_не_чернит() {
    let advised = advised(0);

    assert!(advised.iter().all(|model| !model.heavy), "{advised:?}");
}

#[test]
fn таблица_идёт_от_меньшей_модели_к_большей() {
    let advised = advised(64);

    let sizes: Vec<u32> = advised.iter().map(|model| model.gigabytes).collect();
    let mut sorted = sizes.clone();
    sorted.sort_unstable();
    assert_eq!(sizes, sorted);
}

#[test]
fn память_машины_измеряется_без_сети_и_без_зависимостей() {
    let gigabytes = memory();

    assert!(gigabytes > 0, "{gigabytes}");
    assert!(gigabytes < 4_096, "{gigabytes}");
}
