use std::path::PathBuf;

use super::types::Program;
use crate::yaml::{ParseError, Reader, read};

pub fn programs(source: &str) -> Result<Vec<Program>, ParseError> {
    read(source, |node| match node.optional_field("programs")? {
        Some(field) => field.list(program),
        None => Ok(Vec::new()),
    })
}

fn program(node: &Reader<'_>) -> Result<Program, ParseError> {
    Ok(Program {
        id: node.field("id")?.text()?,
        title: node.field("title")?.text()?,
        path: PathBuf::from(node.field("path")?.text()?),
        opened_at: node
            .optional_field("opened_at")?
            .map(|value| value.optional_text())
            .transpose()?
            .flatten(),
    })
}
