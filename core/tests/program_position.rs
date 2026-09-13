#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "position gate: a panic here is the report"
)]

mod support;

use tolearn_core::program::{self, Tree, position};

fn load(relative: &str) -> Tree {
    program::load(&support::root().join(relative)).unwrap()
}

#[test]
fn the_last_generated_stage_of_a_flat_program_is_the_position() {
    let tree = load("examples/chiptune");

    let at = position(&tree).unwrap();

    assert!(std::ptr::eq(at.node, &tree));
    assert_eq!(at.row.id, "voices");
}

#[test]
fn a_subprogram_goes_before_the_stages_of_its_parent() {
    let tree = load("fixtures/v2/valid/nes-dev");

    let at = position(&tree).unwrap();

    assert_eq!(at.node.program.uuid, "e4c90b7a-15f2-4d6e-8b38-0a2c6f9d1e47");
    assert_eq!(at.row.id, "linker");
}

#[test]
fn a_program_without_generated_stages_has_no_position() {
    let mut tree = load("examples/chiptune");
    tree.stages.clear();

    assert!(position(&tree).is_none());
}
