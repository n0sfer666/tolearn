#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "history gate: a panic here is the report"
)]

mod support;

use tolearn_core::history::{Kept, Snapshot, diff, prune};
use tolearn_core::merge::Part;
use tolearn_core::progress::Progress;
use tolearn_core::roadmap::Roadmap;
use tolearn_core::topic::Topic;

use support::bundles;

fn taken(
    before: (&Roadmap, &[Topic]),
    after: (&Roadmap, &[Topic]),
    state: &Progress,
) -> Vec<String> {
    let drawn = diff(
        Snapshot {
            roadmap: before.0,
            topics: before.1,
        },
        Snapshot {
            roadmap: after.0,
            topics: after.1,
        },
        state,
    );
    drawn.rewritten.iter().map(|it| it.id.clone()).collect()
}

fn without(map: &Roadmap, id: &str) -> Roadmap {
    let mut cut = map.clone();
    cut.topics.retain(|entry| entry.id != id);
    cut
}

fn reworded(topics: &[Topic], id: &str) -> Vec<Topic> {
    let mut changed = topics.to_vec();
    let place = bundles::document(&changed, id);
    changed[place].questions.push(bundles::a_question());
    changed
}

fn retitled(topics: &[Topic], id: &str) -> Vec<Topic> {
    let mut changed = topics.to_vec();
    let place = bundles::document(&changed, id);
    changed[place].title = "Другое название".to_owned();
    changed
}

#[test]
fn тема_которой_не_было_числится_добавленной() {
    let (map, topics) = bundles::reference();
    let before = without(&map, "cp-gateway");

    let drawn = diff(
        Snapshot {
            roadmap: &before,
            topics: &topics,
        },
        Snapshot {
            roadmap: &map,
            topics: &topics,
        },
        &bundles::recorded(&[]),
    );

    assert_eq!(drawn.added, ["cp-gateway"]);
    assert!(drawn.removed.is_empty());
}

#[test]
fn исчезнувшая_тема_числится_удалённой() {
    let (map, topics) = bundles::reference();
    let after = without(&map, "provider-routing");

    let drawn = diff(
        Snapshot {
            roadmap: &map,
            topics: &topics,
        },
        Snapshot {
            roadmap: &after,
            topics: &topics,
        },
        &bundles::recorded(&[]),
    );

    assert_eq!(drawn.removed, ["provider-routing"]);
    assert!(drawn.added.is_empty());
}

#[test]
fn переписанная_тема_названа_изменившимися_частями() {
    let (map, topics) = bundles::reference();
    let after = reworded(&topics, "structured-output");

    let drawn = diff(
        Snapshot {
            roadmap: &map,
            topics: &topics,
        },
        Snapshot {
            roadmap: &map,
            topics: &after,
        },
        &bundles::recorded(&[]),
    );

    let rewritten = drawn.rewritten.first().unwrap();
    assert_eq!(rewritten.id, "structured-output");
    assert_eq!(rewritten.changed, [Part::Questions]);
    assert!(
        !rewritten.title.is_empty(),
        "переписанная тема без названия"
    );
}

#[test]
fn правка_мимо_сути_переписыванием_не_считается() {
    let (map, topics) = bundles::reference();
    let after = retitled(&topics, "structured-output");

    let rewritten = taken((&map, &topics), (&map, &after), &bundles::recorded(&[]));

    assert!(
        rewritten.is_empty(),
        "смена названия сошла за переписывание"
    );
}

#[test]
fn понижение_зачёта_названо_причиной() {
    let (map, topics) = bundles::reference();
    let after = reworded(&topics, "structured-output");
    let state = bundles::recorded(&[("structured-output", "stale_passed")]);

    let drawn = diff(
        Snapshot {
            roadmap: &map,
            topics: &topics,
        },
        Snapshot {
            roadmap: &map,
            topics: &after,
        },
        &state,
    );

    let rewritten = drawn.rewritten.first().unwrap();
    assert!(rewritten.demoted, "понижение зачёта потеряло причину");
    assert_eq!(rewritten.changed, [Part::Questions]);
}

#[test]
fn переписанная_но_незачтённая_тема_понижением_не_числится() {
    let (map, topics) = bundles::reference();
    let after = reworded(&topics, "structured-output");
    let state = bundles::recorded(&[("structured-output", "in_progress")]);

    let drawn = diff(
        Snapshot {
            roadmap: &map,
            topics: &topics,
        },
        Snapshot {
            roadmap: &map,
            topics: &after,
        },
        &state,
    );

    assert!(!drawn.rewritten.first().unwrap().demoted);
}

#[test]
fn несгенерированная_тема_переписанной_не_числится() {
    let (map, whole) = bundles::reference();
    let topics: Vec<Topic> = whole
        .iter()
        .filter(|topic| topic.id != "structured-output")
        .cloned()
        .collect();

    let rewritten = taken((&map, &topics), (&map, &whole), &bundles::recorded(&[]));

    assert!(
        rewritten.is_empty(),
        "тема без прошлой версии сошла за переписанную"
    );
}

fn versions(count: u32, bytes: u64) -> Vec<Kept> {
    (1..=count)
        .map(|n| Kept {
            n,
            saved_at: format!("2026-07-{n:02}"),
            bytes,
        })
        .collect()
}

#[test]
fn глубина_истории_режет_старые_версии() {
    let kept = versions(5, 10);

    let dropped = prune(&kept, 2, 1_000);

    assert_eq!(dropped, [1, 2, 3]);
}

#[test]
fn доля_бюджета_режет_старые_версии_сверх_глубины() {
    let kept = versions(4, 100);

    let dropped = prune(&kept, 10, 250);

    assert_eq!(dropped, [1, 2]);
}

#[test]
fn самая_свежая_версия_остаётся_даже_вне_бюджета() {
    let kept = versions(3, 100);

    let dropped = prune(&kept, 10, 1);

    assert_eq!(dropped, [1, 2], "история осталась без единой версии");
}

#[test]
fn нулевая_глубина_выключает_историю() {
    let kept = versions(3, 10);

    let dropped = prune(&kept, 0, 1_000);

    assert_eq!(dropped, [1, 2, 3]);
}

#[test]
fn версии_вразнобой_режутся_по_возрасту_а_не_по_месту() {
    let mut kept = versions(3, 10);
    kept.reverse();

    let dropped = prune(&kept, 1, 1_000);

    assert_eq!(dropped, [1, 2], "порядок в списке принят за возраст");
}

#[test]
fn история_ровно_в_бюджет_не_режется() {
    let kept = versions(3, 100);

    let dropped = prune(&kept, 10, 300);

    assert!(dropped.is_empty(), "укладываясь в бюджет, история урезана");
}

#[test]
fn ушедшая_из_роадмапа_тема_числится_удалённой_а_не_переписанной() {
    let (map, topics) = bundles::reference();
    let after = without(&map, "structured-output");
    let rewritten = reworded(&topics, "structured-output");

    let drawn = diff(
        Snapshot {
            roadmap: &map,
            topics: &topics,
        },
        Snapshot {
            roadmap: &after,
            topics: &rewritten,
        },
        &bundles::recorded(&[]),
    );

    assert_eq!(drawn.removed, ["structured-output"]);
    assert!(
        drawn.rewritten.is_empty(),
        "тема вне роадмапа названа переписанной: {:?}",
        drawn.rewritten
    );
}
