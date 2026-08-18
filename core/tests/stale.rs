#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "stale gate: a panic here is the report"
)]

mod support;

use tolearn_core::Date;
use tolearn_core::stale::{Aging, Digest, Pin, digest};

use support::bundles;

const TODAY: &str = "2026-07-28";

fn day(text: &str) -> Date {
    Date::parse(text).unwrap_or_else(|| panic!("`{text}` — не дата"))
}

fn taken(today: &str) -> Digest {
    let (map, topics) = bundles::corpus_whole();
    digest(&map, &topics, day(today))
}

fn material<'a>(aging: &'a Digest, title: &str) -> &'a Aging {
    aging
        .materials
        .iter()
        .find(|item| item.title == title)
        .unwrap_or_else(|| panic!("`{title}` нет в дайджесте: {:?}", aging.materials))
}

#[test]
fn тема_стареет_через_свой_срок_после_проверки() {
    let aging = taken(TODAY);

    let expired = aging
        .topics
        .iter()
        .find(|item| item.topic == "stale-knowledge")
        .unwrap_or_else(|| panic!("{:?}", aging.topics));
    assert_eq!(expired.verified_at, "2026-05-01");
    assert_eq!(expired.expired_at, "2026-05-31");
}

#[test]
fn до_срока_тема_в_дайджест_не_попадает() {
    let early = taken("2026-05-30");
    let due = taken("2026-05-31");

    assert!(
        !early
            .topics
            .iter()
            .any(|item| item.topic == "stale-knowledge"),
        "{:?}",
        early.topics
    );
    assert!(
        due.topics
            .iter()
            .any(|item| item.topic == "stale-knowledge"),
        "{:?}",
        due.topics
    );
}

#[test]
fn устаревшие_темы_идут_от_самой_давней() {
    let aging = taken("2030-01-01");

    let dates: Vec<&str> = aging
        .topics
        .iter()
        .map(|item| item.expired_at.as_str())
        .collect();
    let mut sorted = dates.clone();
    sorted.sort_unstable();
    assert_eq!(dates, sorted, "{dates:?}");
    assert!(dates.len() > 1, "нечего сортировать: {dates:?}");
}

#[test]
fn материал_с_флагом_устаревания_назван_вместе_с_темой() {
    let aging = taken(TODAY);

    let flagged = aging
        .materials
        .iter()
        .find(|item| item.stale)
        .unwrap_or_else(|| panic!("{:?}", aging.materials));
    assert_eq!(flagged.topic, "stale-knowledge");
    assert_eq!(flagged.topic_title, "Тема со своим сроком годности");
    assert!(!flagged.url.is_empty());
}

#[test]
fn расхождение_показано_текстом_а_не_флагом() {
    let aging = taken(TODAY);

    let changed = aging
        .materials
        .iter()
        .find(|item| item.delta.is_some())
        .unwrap_or_else(|| panic!("{:?}", aging.materials));
    let delta = changed.delta.as_deref().unwrap();
    assert!(delta.len() > 10, "расхождение пустое: `{delta}`");
}

#[test]
fn спокойный_материал_в_дайджест_не_попадает() {
    let (map, topics) = bundles::corpus_whole();
    let aging = digest(&map, &topics, day(TODAY));

    let all: usize = topics.iter().map(|topic| topic.materials.len()).sum();
    assert!(aging.materials.len() < all, "взяты все материалы подряд");
    assert!(
        aging
            .materials
            .iter()
            .all(|item| item.stale || item.delta.is_some() || item.pin == Pin::Unknown),
        "{:?}",
        aging.materials
    );
}

#[test]
fn версия_из_пинов_программы_вопросов_не_вызывает() {
    let (map, topics) = bundles::reference();
    let aging = digest(&map, &topics, day(TODAY));

    let known = material(&aging, "LiteLLM release notes");
    assert_eq!(known.pin, Pin::Known);
    assert_eq!(known.covers_version.as_deref(), Some("v1.93.0"));
    assert!(map.version_pins.values().any(|pin| pin == "v1.93.0"));
}

#[test]
fn версия_мимо_пинов_помечена_неизвестной() {
    let (map, topics) = bundles::reference();
    let aging = digest(&map, &topics, day(TODAY));

    let unknown = aging
        .materials
        .iter()
        .find(|item| item.pin == Pin::Unknown)
        .unwrap_or_else(|| panic!("{:?}", aging.materials));
    let claimed = unknown.covers_version.as_deref().unwrap();
    assert!(
        !map.version_pins.values().any(|pin| pin == claimed),
        "`{claimed}` всё-таки в пинах"
    );
}

#[test]
fn неизвестный_пин_сам_по_себе_приводит_материал_в_дайджест() {
    let (map, mut topics) = bundles::corpus_whole();
    let quiet = topics
        .iter_mut()
        .flat_map(|topic| topic.materials.iter_mut())
        .find(|material| !material.stale && material.delta.is_none())
        .unwrap_or_else(|| panic!("в корпусе нет спокойного материала"));
    quiet.covers_version = Some("v0.0.0-мимо-пинов".to_owned());
    let title = quiet.title.clone();

    let aging = digest(&map, &topics, day(TODAY));

    let listed = material(&aging, &title);
    assert_eq!(listed.pin, Pin::Unknown);
    assert!(!listed.stale);
    assert_eq!(listed.delta, None);
}

#[test]
fn материал_без_заявленной_версии_неизвестным_пином_не_считается() {
    let (map, topics) = bundles::reference();
    let aging = digest(&map, &topics, day(TODAY));

    assert!(
        aging
            .materials
            .iter()
            .filter(|item| item.covers_version.is_none())
            .all(|item| item.pin == Pin::Absent),
        "{:?}",
        aging.materials
    );
}
