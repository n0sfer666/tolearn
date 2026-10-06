#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "app gate: a panic here is the report"
)]

use tolearn_app::ipc::Ledger;
use tolearn_generate::ledger::{Kind, Record, Tally};

fn spent(step: &str) -> Vec<Record> {
    vec![Record {
        step: step.to_owned(),
        round: None,
        kind: Kind::Model,
        target: None,
        program: None,
        stage: None,
        at: None,
        ms: 1,
        model: None,
        input: None,
        output: None,
        ok: true,
    }]
}

fn steps(ledger: &Ledger) -> Vec<String> {
    ledger
        .tally()
        .records()
        .into_iter()
        .map(|record| record.step)
        .collect()
}

#[test]
fn последняя_карта_заменяет_накопленное() {
    let ledger = Ledger::default();
    ledger.tally().extend(spent("прежняя"));

    let ticket = ledger.ticket();
    ledger.settle(ticket, spent("новая"), Tally::replace);

    assert_eq!(steps(&ledger), ["новая"]);
}

#[test]
fn опоздавшая_карта_дописывается_и_не_затирает_новую() {
    let ledger = Ledger::default();
    let late = ledger.ticket();
    let fresh = ledger.ticket();

    ledger.settle(fresh, spent("новая"), Tally::replace);
    ledger.settle(late, spent("опоздавшая"), Tally::replace);

    assert_eq!(steps(&ledger), ["новая", "опоздавшая"]);
}

#[test]
fn опоздавшая_карта_во_время_старта_не_вытесняет_возврат_попытки() {
    let ledger = Ledger::default();
    ledger.tally().extend(spent("карта"));
    let late = ledger.ticket();

    let taken = ledger.take();
    ledger.settle(late, spent("опоздавшая"), Tally::replace);
    ledger.tally().restore(taken);

    assert_eq!(steps(&ledger), ["карта", "опоздавшая"]);
}

#[test]
fn старт_делает_прежний_билет_устаревшим() {
    let ledger = Ledger::default();
    ledger.tally().extend(spent("карта"));
    let late = ledger.ticket();

    let taken = ledger.take();
    ledger.tally().restore(taken);
    ledger.settle(late, spent("опоздавшая"), Tally::replace);

    assert_eq!(steps(&ledger), ["карта", "опоздавшая"]);
}
