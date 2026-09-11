use super::types::{Block, Kind};
use crate::yaml::{ParseError, Reader};

pub(crate) fn block(node: &Reader<'_>) -> Result<Block, ParseError> {
    let kinds = Kind::ALL.map(|kind| (kind.label(), kind));
    Ok(Block {
        id: node.field("id")?.text()?,
        kind: node.field("kind")?.choice("block kind", &kinds)?,
        text: node.field("text")?.text()?,
        lang: optional(node, "lang")?,
        asset: optional(node, "asset")?,
        license: optional(node, "license")?,
        attribution: optional(node, "attribution")?,
        source: optional(node, "source")?,
    })
}

fn optional(node: &Reader<'_>, name: &str) -> Result<Option<String>, ParseError> {
    node.optional_field(name)?
        .as_ref()
        .map(Reader::text)
        .transpose()
}
