#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "plan gate: a panic here is the report"
)]

mod support;

use tolearn_core::plan::{Plan, plan};
use tolearn_core::roadmap::{Priority, Roadmap};
use tolearn_core::status::effective;
use tolearn_core::topic::Topic;
use tolearn_core::{Date, Hours};

use support::bundles;

const TODAY: &str = "2026-07-28";

fn day(text: &str) -> Date {
    Date::parse(text).unwrap_or_else(|| panic!("`{text}` — не дата"))
}

fn planned(bundle: (Roadmap, Vec<Topic>), passed: &[&str]) -> Plan {
    let (map, topics) = bundle;
    let pairs: Vec<(&str, &str)> = passed.iter().map(|id| (*id, "passed")).collect();
    let state = bundles::recorded(&pairs);
    let statuses = effective(&map, &topics, &state, day(TODAY));
    plan(&map, &topics, &statuses, day(TODAY))
}

fn whole(passed: &[&str]) -> Plan {
    planned(bundles::corpus_whole(), passed)
}

fn left_of(bundle: &(Roadmap, Vec<Topic>)) -> Hours {
    let (map, topics) = bundle;
    map.topics
        .iter()
        .filter(|entry| entry.priority != Priority::Optional)
        .filter(|entry| topics.iter().any(|topic| topic.id == entry.id))
        .fold(Hours::default(), |sum, entry| Hours {
            min: sum.min + entry.est_hours.min,
            max: sum.max + entry.est_hours.max,
        })
}

#[test]
fn дневная_норма_считается_из_недельного_бюджета() {
    let ahead = whole(&[]);

    assert_eq!(ahead.weekly_hours, 4);
    assert!(
        (ahead.daily_hours - 4.0 / 7.0).abs() < 1e-9,
        "{}",
        ahead.daily_hours
    );
}

#[test]
fn прогноз_завершения_это_интервал_из_остатка() {
    let ahead = whole(&[]);
    let left = left_of(&bundles::corpus_whole());

    assert_eq!(ahead.left, left);
    assert!(ahead.soonest.days < ahead.latest.days, "{ahead:?}");
    assert_eq!(
        ahead.soonest.days,
        (f64::from(left.min) / ahead.daily_hours).ceil() as u32
    );
    assert_eq!(
        ahead.latest.days,
        (f64::from(left.max) / ahead.daily_hours).ceil() as u32
    );
    assert_eq!(
        ahead.soonest.date,
        day(TODAY).plus_days(ahead.soonest.days).to_string()
    );
    assert_eq!(
        ahead.latest.date,
        day(TODAY).plus_days(ahead.latest.days).to_string()
    );
}

#[test]
fn пройденное_из_остатка_вычитается() {
    let ahead = whole(&["cycle-a"]);
    let full = whole(&[]);

    assert!(ahead.left.min < full.left.min, "{ahead:?}");
    assert!(ahead.left.max < full.left.max, "{ahead:?}");
    assert!(ahead.latest.days < full.latest.days, "{ahead:?}");
}

#[test]
fn несгенерированные_темы_помечены_неизвестным_остатком() {
    let ahead = planned(bundles::corpus(), &[]);

    assert!(ahead.unknown > 0, "{ahead:?}");
    assert_eq!(ahead.left, left_of(&bundles::corpus()));
}

#[test]
fn несгенерированные_часы_в_интервал_не_входят() {
    let partial = planned(bundles::corpus(), &[]);
    let full = whole(&[]);

    assert!(partial.left.max < full.left.max, "{partial:?}");
    assert_eq!(full.unknown, 0, "{full:?}");
}

#[test]
fn необязательная_тема_не_идёт_ни_в_остаток_ни_в_неизвестное() {
    let (map, topics) = bundles::reference();
    let written = topics.len();
    let optional = map
        .topics
        .iter()
        .filter(|entry| entry.priority == Priority::Optional)
        .count();
    assert!(optional > 0, "в эталоне нет необязательных тем");

    let ahead = planned(bundles::reference(), &[]);

    assert_eq!(
        usize::try_from(ahead.unknown).unwrap(),
        map.topics.len() - optional - written
    );
    assert_eq!(ahead.left, left_of(&bundles::reference()));
}

#[test]
fn пустой_остаток_завершает_программу_сегодня() {
    let ahead = whole(&[
        "cycle-a",
        "cycle-b",
        "stale-knowledge",
        "offline-edge",
        "question-shapes",
    ]);

    assert_eq!(ahead.left, Hours { min: 0, max: 0 });
    assert_eq!(ahead.soonest.days, 0);
    assert_eq!(ahead.latest.days, 0);
    assert_eq!(ahead.soonest.date, TODAY);
    assert_eq!(ahead.latest.date, TODAY);
}
