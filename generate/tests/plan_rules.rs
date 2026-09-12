#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "generate gate: a panic here is the report"
)]

use tolearn_core::Hours;
use tolearn_core::program::{StageRow, Volatility};
use tolearn_generate::plan::{self, Flaw, Part, Plan};

fn stage(id: &str, min: u32, max: u32) -> StageRow {
    StageRow {
        id: id.to_owned(),
        title: format!("Этап {id}"),
        hours: Hours { min, max },
    }
}

fn part(title: &str, min: u32, max: u32) -> Part {
    Part {
        title: title.to_owned(),
        goal: format!("Цель {title}"),
        hours: Hours { min, max },
    }
}

fn leaf(stages: Vec<StageRow>) -> Plan {
    Plan {
        title: "Чиптюн".to_owned(),
        slug: "chiptune".to_owned(),
        goal: "Свести трек".to_owned(),
        volatility: Volatility::Stable,
        stages,
        children: Vec::new(),
    }
}

fn node(children: Vec<Part>) -> Plan {
    Plan {
        children,
        ..leaf(Vec::new())
    }
}

fn said(flaws: &[Flaw]) -> Vec<String> {
    flaws.iter().map(ToString::to_string).collect()
}

#[test]
fn stage_takes_two_to_four_hours_under_a_slug_id() {
    let fine = leaf(vec![stage("a", 2, 4), stage("b", 2, 2), stage("c", 4, 4)]);
    assert_eq!(plan::check(&fine, 1), Vec::<Flaw>::new());

    let cases = [
        (stage("short", 1, 3), "short", "1–3 ч"),
        (stage("long", 3, 5), "long", "3–5 ч"),
        (stage("upside", 4, 2), "upside", "4–2 ч"),
        (stage("Тема-1", 2, 3), "Тема-1", "латиница"),
    ];
    for (row, id, expected) in cases {
        let flaws = said(&plan::check(&leaf(vec![row]), 1));
        assert_eq!(flaws.len(), 1, "{id}: {flaws:?}");
        assert!(
            flaws[0].contains(id) && flaws[0].contains(expected),
            "{flaws:?}"
        );
    }
}

#[test]
fn leaf_holds_at_most_seventy_hours_and_twenty_five_stages() {
    let heavy = leaf((0..18).map(|n| stage(&format!("s{n}"), 3, 4)).collect());
    let flaws = said(&plan::check(&heavy, 1));
    assert_eq!(flaws.len(), 1, "{flaws:?}");
    assert!(flaws[0].contains("54–72 ч"), "{flaws:?}");

    let long = leaf((0..26).map(|n| stage(&format!("s{n}"), 2, 2)).collect());
    let flaws = said(&plan::check(&long, 1));
    assert_eq!(flaws.len(), 1, "{flaws:?}");
    assert!(flaws[0].contains("26 этапов"), "{flaws:?}");

    let edge = leaf((0..25).map(|n| stage(&format!("s{n}"), 2, 2)).collect());
    assert!(plan::check(&edge, 1).is_empty());
}

#[test]
fn node_rows_hold_at_most_seventy_hours_and_depth_stops_at_three() {
    let fine = node(vec![part("A", 50, 70), part("B", 60, 70)]);
    assert!(plan::check(&fine, 2).is_empty());

    let flaws = said(&plan::check(
        &node(vec![part("A", 50, 80), part("B", 0, 10)]),
        1,
    ));
    assert_eq!(flaws.len(), 2, "{flaws:?}");
    assert!(flaws[0].contains("50–80 ч"), "{flaws:?}");
    assert!(flaws[1].contains("0–10 ч"), "{flaws:?}");

    let deep = node(vec![part("A", 50, 70)]);
    assert_eq!(plan::check(&deep, 3), vec![Flaw::TooDeep { depth: 3 }]);
}

#[test]
fn map_is_either_a_node_or_a_leaf_and_names_itself() {
    assert_eq!(plan::check(&leaf(Vec::new()), 1), vec![Flaw::Empty]);

    let mixed = Plan {
        children: vec![part("A", 10, 20)],
        ..leaf(vec![stage("a", 2, 3)])
    };
    assert_eq!(plan::check(&mixed, 1), vec![Flaw::Mixed]);

    let nameless = Plan {
        title: " ".to_owned(),
        slug: "Chip tune".to_owned(),
        goal: String::new(),
        ..leaf(vec![stage("a", 2, 3), stage("a", 2, 3)])
    };
    let flaws = said(&plan::check(&nameless, 1));
    assert_eq!(flaws.len(), 4, "{flaws:?}");
    for expected in [
        "название программы",
        "цель программы",
        "«Chip tune»",
        "«a» повторяется",
    ] {
        assert!(
            flaws.iter().any(|flaw| flaw.contains(expected)),
            "{expected}: {flaws:?}"
        );
    }
}
