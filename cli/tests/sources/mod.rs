#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "architecture gate: a panic here is the report"
)]

pub mod document;
pub mod lockfile;
pub mod manifest;

use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

pub fn read(relative: &str) -> String {
    let path = workspace_root().join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

pub fn parse(relative: &str) -> toml::Table {
    read(relative)
        .parse()
        .unwrap_or_else(|e| panic!("{relative}: {e}"))
}
