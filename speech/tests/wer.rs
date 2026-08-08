#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "speech gate: a panic here is the report"
)]

use tolearn_speech::wer::{Score, score};
use tolearn_speech::words::words;

#[test]
fn дословное_совпадение_стоит_ноль() {
    let heard = score("Контекст тоже оплачивается", "контекст, тоже оплачивается!");

    assert_eq!(heard.errors, 0);
    assert_eq!(heard.words, 3);
    assert_eq!(heard.rate(), 0.0);
}

#[test]
fn ошибки_считаются_по_словам_а_не_по_буквам() {
    let missed = score("веса плюс KV кэш", "веса плюс кэш");
    let extra = score("веса плюс кэш", "веса плюс KV кэш");
    let wrong = score("веса плюс кэш", "веса минус кэш");

    assert_eq!(missed.errors, 1);
    assert_eq!(extra.errors, 1);
    assert_eq!(wrong.errors, 1);
    assert!((wrong.rate() - 1.0 / 3.0).abs() < 1e-9);
}

#[test]
fn ё_и_регистр_не_считаются_ошибкой() {
    assert_eq!(words("Всё, Ещё"), vec!["все", "еще"]);
    assert_eq!(score("всё ещё", "все еще").errors, 0);
}

#[test]
fn корпус_складывается_в_одно_число() {
    let total: Score = [score("а б в г", "а б в г"), score("д е", "д ж")]
        .into_iter()
        .sum();

    assert_eq!(total.words, 6);
    assert_eq!(total.errors, 1);
    assert!((total.rate() - 1.0 / 6.0).abs() < 1e-9);
}

#[test]
fn пустая_расшифровка_проваливает_всё() {
    assert_eq!(score("веса плюс кэш", "").rate(), 1.0);
    assert_eq!(score("", "").rate(), 0.0);
    assert_eq!(score("", "лишнее").rate(), 1.0);
}
