use saphyr::Yaml;

use super::types::Block;
use crate::yaml::{map, text};

pub(crate) fn written(block: &Block) -> Yaml<'_> {
    let mut entries = vec![
        ("id", text(&block.id)),
        ("kind", text(block.kind.label())),
        ("text", text(&block.text)),
    ];
    entries.extend(
        block
            .extras()
            .into_iter()
            .filter_map(|(name, value)| value.map(|value| (name, text(value)))),
    );
    map(entries)
}
