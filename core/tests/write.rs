#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "writer gate: a panic here is the report"
)]

mod support;

use tolearn_core::program::{self, Tree};
use tolearn_core::stage::{self, Check};

const FIXTURES: [&str; 2] = ["fixtures/v2/valid/nes-dev", "examples/chiptune"];
const LEAF: &str = "fixtures/v2/valid/nes-dev/children/b25e8f13-6d47-4a09-a3c1-9e7f2d4b8c56/children/e4c90b7a-15f2-4d6e-8b38-0a2c6f9d1e47";

const AWKWARD: [&str; 30] = [
    "12345678",
    "1e5",
    "0x1F",
    ".5",
    "true",
    "null",
    "~",
    "yes",
    "2026-09-11",
    "- пункт",
    "ключ: значение",
    "# не комментарий",
    "  отступ",
    "хвост  ",
    "строка\nвторая",
    "  первая с отступом\nвторая",
    "\nначинается с пустой",
    "кончается переводом\n",
    "два перевода\n\n",
    "crlf\r\nтекст",
    "кавычки \" и '",
    "таб\tвнутри",
    "эмодзи 🎮",
    "{скобки}",
    "[a, b]",
    "@at",
    "`code`",
    "%процент",
    "обратный \\ слэш",
    "код:\n  отступ\n\n    ещё\nконец",
];

fn every(tree: &Tree) -> Vec<&Tree> {
    let mut found = vec![tree];
    found.extend(tree.children.values().flat_map(every));
    found
}

#[test]
fn the_fixtures_read_back_as_they_were_written() {
    for relative in FIXTURES {
        let tree = program::load(&support::root().join(relative)).unwrap();
        for node in every(&tree) {
            let written = program::write(&node.program).unwrap();
            assert_eq!(program::parse(&written).unwrap(), node.program, "{written}");
            for stage in node.stages.values() {
                let written = stage::write(stage).unwrap();
                assert_eq!(stage::parse(&written).unwrap(), *stage, "{written}");
            }
        }
    }
}

#[test]
fn awkward_texts_survive_the_round_trip() {
    let tree = program::load(&support::root().join(LEAF)).unwrap();
    let original = tree.stages["first-rom"].clone();
    for text in AWKWARD {
        let mut stage = original.clone();
        stage.title = text.to_owned();
        stage.blocks[0].text = text.to_owned();
        stage.blocks[0].id = text.to_owned();
        stage.practice.task[3].text = format!("{text}\n{text}");
        stage.practice.acceptance.push(Check {
            id: text.to_owned(),
            claim: text.to_owned(),
            check: None,
            expect: text.to_owned(),
        });
        let written = stage::write(&stage).unwrap();
        assert_eq!(
            stage::parse(&written).unwrap(),
            stage,
            "{text:?}\n{written}"
        );

        let mut program = tree.program.clone();
        program.goal = text.to_owned();
        program.map.stages[0].title = text.to_owned();
        let written = program::write(&program).unwrap();
        assert_eq!(
            program::parse(&written).unwrap(),
            program,
            "{text:?}\n{written}"
        );
    }
}

#[test]
fn a_child_row_goal_is_written_only_when_it_is_known() {
    let tree = program::load(&support::root().join(FIXTURES[0])).unwrap();
    let mut program = tree.program.clone();
    assert!(program.map.children.iter().all(|row| row.goal.is_none()));

    let bare = program::write(&program).unwrap();
    program.map.children[0].goal = Some("Собрать первый ROM: goal".to_owned());
    let written = program::write(&program).unwrap();

    assert_eq!(bare.matches("goal:").count(), 1, "{bare}");
    assert_eq!(written.matches("  goal:").count(), 1, "{written}");
    assert_eq!(program::parse(&written).unwrap(), program, "{written}");
}

#[test]
fn code_is_written_as_a_literal_block() {
    let tree = program::load(&support::root().join(LEAF)).unwrap();

    let written = stage::write(&tree.stages["first-rom"]).unwrap();

    assert!(
        written.starts_with("schema: tolearn/stage/1\n"),
        "{written}"
    );
    assert!(
        written.contains("text: |-\n        ca65 main.s -o main.o\n"),
        "{written}"
    );
}
