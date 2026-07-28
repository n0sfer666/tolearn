#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "practice gate: a panic here is the report"
)]

use tolearn_core::practice::{Session, Step, advance, timer};

const BOX: u32 = 45;

fn started(at: &str) -> Session {
    advance(&Session::default(), Step::Start, BOX, at)
}

#[test]
fn отсчёт_идёт_от_таймбокса_темы() {
    let idle = Session::default();

    let left = timer(&idle, BOX, "2026-07-28T10:00:00+03:00");

    assert_eq!(left.left_sec, 45 * 60);
    assert_eq!(left.spent_sec, 0);
    assert!(!left.running);
    assert!(!left.expired);
}

#[test]
fn запущенный_таймер_считает_от_момента_старта() {
    let session = started("2026-07-28T10:00:00+03:00");

    let left = timer(&session, BOX, "2026-07-28T10:10:00+03:00");

    assert!(left.running);
    assert_eq!(left.spent_sec, 600);
    assert_eq!(left.left_sec, 45 * 60 - 600);
}

#[test]
fn пауза_замораживает_потраченное() {
    let session = started("2026-07-28T10:00:00+03:00");
    let paused = advance(&session, Step::Pause, BOX, "2026-07-28T10:10:00+03:00");

    let left = timer(&paused, BOX, "2026-07-28T11:00:00+03:00");

    assert!(!left.running);
    assert_eq!(left.spent_sec, 600);
    assert_eq!(left.left_sec, 45 * 60 - 600);
}

#[test]
fn продолжение_после_паузы_прибавляет_ко_вчерашнему() {
    let session = started("2026-07-28T10:00:00+03:00");
    let paused = advance(&session, Step::Pause, BOX, "2026-07-28T10:10:00+03:00");
    let again = advance(&paused, Step::Start, BOX, "2026-07-28T11:00:00+03:00");

    let left = timer(&again, BOX, "2026-07-28T11:05:00+03:00");

    assert_eq!(left.spent_sec, 900);
}

#[test]
fn сброс_обнуляет_и_потраченное_и_истечение() {
    let session = started("2026-07-28T10:00:00+03:00");
    let expired = advance(&session, Step::Pause, BOX, "2026-07-28T11:00:00+03:00");
    assert!(expired.expired);

    let fresh = advance(&expired, Step::Reset, BOX, "2026-07-28T11:01:00+03:00");

    assert_eq!(fresh, Session::default());
    assert!(!timer(&fresh, BOX, "2026-07-28T11:01:00+03:00").expired);
}

#[test]
fn истечение_не_останавливает_работу() {
    let session = started("2026-07-28T10:00:00+03:00");

    let left = timer(&session, BOX, "2026-07-28T11:00:00+03:00");

    assert!(left.running, "таймер встал сам собой: {left:?}");
    assert!(left.expired);
    assert_eq!(left.left_sec, 45 * 60 - 3600);
    assert_eq!(left.spent_sec, 3600);
}

#[test]
fn истечение_остаётся_записанным_после_паузы() {
    let session = started("2026-07-28T10:00:00+03:00");
    let paused = advance(&session, Step::Pause, BOX, "2026-07-28T11:00:00+03:00");

    let back = advance(&paused, Step::Start, BOX, "2026-07-28T11:10:00+03:00");

    assert!(paused.expired);
    assert!(back.expired, "истечение забыто при продолжении: {back:?}");
}

#[test]
fn ровно_таймбокс_уже_истёк() {
    let session = started("2026-07-28T10:00:00+03:00");

    let left = timer(&session, BOX, "2026-07-28T10:45:00+03:00");

    assert_eq!(left.spent_sec, 45 * 60);
    assert_eq!(left.left_sec, 0);
    assert!(
        left.expired,
        "ровный таймбокс не засчитан истёкшим: {left:?}"
    );
}

#[test]
fn истечение_переживает_рост_таймбокса() {
    let session = started("2026-07-28T10:00:00+03:00");
    let paused = advance(&session, Step::Pause, BOX, "2026-07-28T11:00:00+03:00");
    assert!(paused.expired);

    let left = timer(&paused, BOX * 2, "2026-07-28T11:10:00+03:00");
    let back = advance(&paused, Step::Start, BOX * 2, "2026-07-28T11:10:00+03:00");

    assert!(left.expired, "рост таймбокса стёр запись об истечении");
    assert!(back.expired, "рост таймбокса стёр запись об истечении");
}

#[test]
fn таймер_переживает_переход_между_экранами() {
    let session = started("2026-07-28T10:00:00+03:00");

    let stored = Session {
        started_at: session.started_at.clone(),
        spent_sec: session.spent_sec,
        expired: session.expired,
    };
    let left = timer(&stored, BOX, "2026-07-28T10:30:00+03:00");

    assert_eq!(left.spent_sec, 1800);
    assert!(left.running);
}

#[test]
fn часовые_пояса_не_сдвигают_отсчёт() {
    let session = started("2026-07-28T10:00:00+03:00");

    let left = timer(&session, BOX, "2026-07-28T08:10:00+01:00");

    assert_eq!(left.spent_sec, 600);
}

#[test]
fn кривой_момент_не_двигает_счёт() {
    let session = started("2026-07-28T10:00:00+03:00");

    let left = timer(&session, BOX, "вчера");
    let stuck = advance(&session, Step::Pause, BOX, "вчера");

    assert_eq!(left.spent_sec, 0);
    assert_eq!(stuck.spent_sec, 0);
    assert_eq!(stuck.started_at, None);
}

#[test]
fn кривой_момент_не_запускает_таймер() {
    let broken = started("вчера");
    let dateless = started("2026-07-28+03:00");

    assert_eq!(
        broken.started_at, None,
        "таймер запущен от нечитаемого момента"
    );
    assert_eq!(
        dateless.started_at, None,
        "дата без времени принята за момент"
    );
    assert!(!timer(&broken, BOX, "2026-07-28T10:00:00+03:00").running);
    assert!(!timer(&dateless, BOX, "2026-07-28T10:00:00+03:00").running);
}

#[test]
fn часы_назад_не_отматывают_потраченное() {
    let session = started("2026-07-28T10:00:00+03:00");

    let left = timer(&session, BOX, "2026-07-28T09:00:00+03:00");

    assert_eq!(left.spent_sec, 0);
}

#[test]
fn повторный_старт_не_перезапускает_отсчёт() {
    let session = started("2026-07-28T10:00:00+03:00");

    let again = advance(&session, Step::Start, BOX, "2026-07-28T10:10:00+03:00");

    assert_eq!(again.started_at, session.started_at);
    assert_eq!(again.spent_sec, 0);
}
