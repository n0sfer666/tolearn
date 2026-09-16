#![allow(
    dead_code,
    reason = "every test binary takes its own part of the shared support module"
)]

#[path = "../../../tests-support/scratch.rs"]
pub mod scratch;

pub mod archives;
pub mod packages;
pub mod pictures;
pub mod programs;
pub mod sealing;
pub mod search;
pub mod states;

use std::path::{Path, PathBuf};

pub fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

pub fn read(relative: &str) -> String {
    let path = root().join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

pub fn line_of(source: &str, needle: &str) -> usize {
    source
        .lines()
        .position(|line| line.contains(needle))
        .unwrap_or_else(|| panic!("`{needle}` is not in the source"))
        + 1
}
