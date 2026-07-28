use std::path::PathBuf;
use std::str::FromStr;

use super::types::{Document, KINDS, Source};
use crate::notes::Stamp;
use crate::yaml::{ParseError, Reader, read};

pub fn sources(source: &str) -> Result<Vec<Source>, ParseError> {
    read(source, |node| match node.optional_field("sources")? {
        Some(field) => field.list(one),
        None => Ok(Vec::new()),
    })
}

fn one(node: &Reader<'_>) -> Result<Source, ParseError> {
    Ok(Source {
        path: PathBuf::from(node.field("path")?.text()?),
        roadmap: node.field("roadmap")?.any_text()?,
        stamp: Stamp {
            modified_nanos: number(&node.field("modified")?)?,
            size: number(&node.field("size")?)?,
        },
        documents: node.field("documents")?.list(document)?,
    })
}

fn document(node: &Reader<'_>) -> Result<Document, ParseError> {
    Ok(Document {
        kind: node.field("kind")?.choice("kind", &KINDS)?,
        topic: node.field("topic")?.any_text()?,
        title: node.field("title")?.any_text()?,
        text: node.field("text")?.any_text()?,
    })
}

fn number<T: FromStr>(node: &Reader<'_>) -> Result<T, ParseError> {
    let text = node.text()?;
    text.parse()
        .map_err(|_| node.malformed(format!("`{text}` is no whole number")))
}
