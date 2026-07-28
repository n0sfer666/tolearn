#![allow(
    dead_code,
    reason = "every test binary takes its own part of the shared support module"
)]

pub mod archives;
pub mod bundles;

use std::fmt::Display;
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

pub fn every_fixture_parses<T, E: Display>(
    directory: &str,
    parse: impl Fn(&str) -> Result<T, E>,
    minimum: usize,
) {
    let mut seen = 0;
    for entry in std::fs::read_dir(root().join(directory)).unwrap() {
        let name = entry.unwrap().file_name().to_string_lossy().into_owned();
        if !name.ends_with(".yaml") {
            continue;
        }
        let relative = format!("{directory}/{name}");
        parse(&read(&relative)).unwrap_or_else(|e| panic!("{relative}: {e}"));
        seen += 1;
    }
    assert!(
        seen >= minimum,
        "{directory} lost its fixtures: {seen} of {minimum}"
    );
}
