#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "branch gate: a panic here is the report"
)]

mod support;

use tolearn_core::Hours;
use tolearn_core::program::{self, Tree};

const CHIPTUNE: &str = "examples/chiptune";
const NES_DEV: &str = "fixtures/v2/valid/nes-dev";
const TOOLS: &str = "b25e8f13-6d47-4a09-a3c1-9e7f2d4b8c56";
const ROM: &str = "e4c90b7a-15f2-4d6e-8b38-0a2c6f9d1e47";
const SOUND: &str = "5c8e1f2a-7b3d-4e69-9a04-2d6f8b1c3e75";

fn load(relative: &str) -> Tree {
    program::load(&support::root().join(relative)).unwrap()
}

#[test]
fn the_root_is_its_own_branch_with_no_trail() {
    let tree = load(NES_DEV);

    let branch = tree.branch(&tree.program.uuid).unwrap();

    assert!(std::ptr::eq(branch.tree, &tree));
    assert!(branch.trail.is_empty());
    assert_eq!(branch.prefix, "");
}

#[test]
fn a_grandchild_is_found_with_its_trail_and_prefix() {
    let tree = load(NES_DEV);

    let branch = tree.branch(ROM).unwrap();

    assert_eq!(branch.tree.program.uuid, ROM);
    let trail: Vec<&str> = branch
        .trail
        .iter()
        .map(|node| node.program.uuid.as_str())
        .collect();
    assert_eq!(trail, [tree.program.uuid.as_str(), TOOLS]);
    assert_eq!(branch.prefix, format!("children/{TOOLS}/children/{ROM}/"));
    let pixel = format!("{}assets/pixel.png", branch.prefix);
    assert!(tree.files().contains(&pixel), "{pixel} is not a tree file");
}

#[test]
fn an_ungenerated_or_unknown_uuid_has_no_branch() {
    let tree = load(NES_DEV);

    assert!(tree.branch(SOUND).is_none());
    assert!(
        tree.branch("00000000-0000-4000-8000-000000000000")
            .is_none()
    );
    assert!(tree.branch("").is_none());
}

#[test]
fn map_hours_add_up_stage_and_child_rows() {
    assert_eq!(
        load(CHIPTUNE).program.map.hours(),
        Hours { min: 7, max: 11 }
    );
    assert_eq!(
        load(NES_DEV).program.map.hours(),
        Hours { min: 14, max: 22 }
    );
}
