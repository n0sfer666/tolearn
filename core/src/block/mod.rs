mod parse;
mod types;

pub(crate) use parse::block;
pub use types::{Block, Kind};

use std::collections::BTreeMap;

use sha2::{Digest, Sha256};

pub const PREFIX: usize = 8;

pub fn id(text: &str) -> String {
    Sha256::digest(text.as_bytes())
        .iter()
        .take(PREFIX / 2)
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub fn ids<'a>(texts: impl IntoIterator<Item = &'a str>) -> Vec<String> {
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    texts
        .into_iter()
        .map(|text| {
            let base = id(text);
            let count = seen.entry(base.clone()).or_default();
            *count += 1;
            if *count == 1 {
                base
            } else {
                format!("{base}-{count}")
            }
        })
        .collect()
}
