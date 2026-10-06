#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "export gate: a panic here is the report"
)]

mod support;

use std::fs;
use std::path::PathBuf;

use support::programs::scratch;
use tolearn_core::export::{claimed, inside};

fn program(name: &str) -> (PathBuf, PathBuf) {
    let room = scratch(name);
    let program = room.join("program");
    fs::create_dir(&program).unwrap();
    (room, program)
}

#[test]
fn a_target_below_missing_folders_is_still_inside() {
    let (_, program) = program("place-deep");

    assert!(inside(
        &program,
        &program.join("нет").join("и тут").join("out")
    ));
    assert!(inside(
        &program,
        &program.join("нет").join("..").join("out")
    ));
}

#[test]
fn a_target_that_climbs_out_is_outside() {
    let (room, program) = program("place-out");

    assert!(!inside(
        &program,
        &program.join("нет").join("..").join("..").join("out")
    ));
    assert!(!inside(&program, &room.join("programme")));
}

#[cfg(unix)]
#[test]
fn a_link_into_the_program_is_inside() {
    let (room, program) = program("place-link");
    std::os::unix::fs::symlink(&program, room.join("link")).unwrap();

    assert!(inside(&program, &room.join("link").join("нет").join("out")));
}

#[cfg(unix)]
#[test]
fn a_link_behind_a_climb_out_of_a_missing_folder_is_inside() {
    let (room, program) = program("place-climb-link");
    std::os::unix::fs::symlink(&program, room.join("link")).unwrap();

    let target = room.join("нет").join("..").join("link").join("out");

    assert!(inside(&program, &target));
}

#[cfg(unix)]
#[test]
fn an_unresolvable_target_counts_as_inside() {
    let (room, program) = program("place-dangling");
    std::os::unix::fs::symlink(room.join("никуда"), room.join("dangling")).unwrap();

    assert!(inside(&program, &room.join("dangling").join("out")));
}

#[test]
fn a_target_below_any_program_is_claimed() {
    let (room, program) = program("place-claimed");
    fs::write(program.join("program.yaml"), "").unwrap();

    assert!(claimed(&program));
    assert!(claimed(&program.join("нет").join("out")));
    assert!(!claimed(&room.join("out")));
    assert!(!claimed(&program.join("..").join("out")));
}
