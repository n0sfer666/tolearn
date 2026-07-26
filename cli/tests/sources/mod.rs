pub mod document;
pub mod lockfile;
pub mod manifest;

use crate::repo::read;

pub fn parse(relative: &str) -> toml::Table {
    read(relative)
        .parse()
        .unwrap_or_else(|e| panic!("{relative}: {e}"))
}
