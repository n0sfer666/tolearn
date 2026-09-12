use std::str::FromStr;

use super::render::{BUILD, SCHEMA};
use super::types::{Document, KINDS, Seen, Source, Stamp};
use crate::yaml::{ParseError, Reader, read};

pub fn sources(source: &str) -> Result<Vec<Source>, ParseError> {
    read(source, |node| {
        node.field("schema")?
            .choice("search schema", &[(SCHEMA, ())])?;
        node.field("build")?
            .choice("search build", &[(BUILD, ())])?;
        match node.optional_field("sources")? {
            Some(field) => field.list(one),
            None => Ok(Vec::new()),
        }
    })
}

fn one(node: &Reader<'_>) -> Result<Source, ParseError> {
    Ok(Source {
        program: node.field("program")?.text()?,
        files: node.field("files")?.list(seen)?,
        documents: node.field("documents")?.list(document)?,
    })
}

fn seen(node: &Reader<'_>) -> Result<Seen, ParseError> {
    Ok(Seen {
        name: node.field("name")?.text()?,
        stamp: Stamp {
            modified_nanos: number(&node.field("modified")?)?,
            size: number(&node.field("size")?)?,
        },
    })
}

fn document(node: &Reader<'_>) -> Result<Document, ParseError> {
    Ok(Document {
        kind: node.field("kind")?.choice("kind", &KINDS)?,
        node: node.field("node")?.any_text()?,
        node_title: node.field("node_title")?.any_text()?,
        stage: node.field("stage")?.any_text()?,
        title: node.field("title")?.any_text()?,
        block: node.field("block")?.any_text()?,
        text: node.field("text")?.any_text()?,
    })
}

fn number<T: FromStr>(node: &Reader<'_>) -> Result<T, ParseError> {
    let text = node.text()?;
    text.parse()
        .map_err(|_| node.malformed(format!("`{text}` is no whole number")))
}
