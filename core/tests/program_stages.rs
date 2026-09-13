#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "stages gate: a panic here is the report"
)]

mod support;

use tolearn_core::program::{self, Tree};

const CHIPTUNE: &str = "3f6c2a1e-8b4d-4c7a-9e21-5d0f7b3a6c84";
const TOOLS: &str = "b25e8f13-6d47-4a09-a3c1-9e7f2d4b8c56";
const ROM: &str = "e4c90b7a-15f2-4d6e-8b38-0a2c6f9d1e47";

fn load(relative: &str) -> Tree {
    program::load(&support::root().join(relative)).unwrap()
}

#[test]
fn a_leaf_lists_only_its_generated_stages() {
    let tree = load("examples/chiptune");

    assert_eq!(tree.every_stage(), [(CHIPTUNE, "voices")]);
}

#[test]
fn a_container_lists_the_stages_of_its_generated_subtree_by_node() {
    let tree = load("fixtures/v2/valid/nes-dev");
    let rom = [(ROM, "first-rom"), (ROM, "linker")];

    assert_eq!(tree.every_stage(), rom);
    assert_eq!(tree.children[TOOLS].every_stage(), rom);
}
