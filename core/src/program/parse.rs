use super::types::{Book, ChildRow, Generation, Map, Page, Program, Sources, StageRow, Volatility};
use crate::hours::hours;
use crate::yaml::{ParseError, Reader, read};

pub(super) const SCHEMA: &str = "tolearn/program/1";

pub fn parse(source: &str) -> Result<Program, ParseError> {
    read(source, program)
}

fn program(node: &Reader<'_>) -> Result<Program, ParseError> {
    node.field("schema")?
        .choice("program schema", &[(SCHEMA, ())])?;
    Ok(Program {
        uuid: node.field("uuid")?.uuid()?,
        slug: node.field("slug")?.slug()?,
        title: node.field("title")?.text()?,
        goal: node.field("goal")?.text()?,
        level: node.field("level")?.text()?,
        generation: generation(&node.field("generation")?)?,
        map: map(&node.field("map")?)?,
        sources: sources(&node.field("sources")?)?,
    })
}

fn generation(node: &Reader<'_>) -> Result<Generation, ParseError> {
    Ok(Generation {
        locale: node.field("locale")?.text()?,
        volatility: node.field("volatility")?.choice(
            "volatility",
            &[
                ("stable", Volatility::Stable),
                ("evolving", Volatility::Evolving),
                ("volatile", Volatility::Volatile),
            ],
        )?,
        request: node.field("request")?.text()?,
    })
}

fn map(node: &Reader<'_>) -> Result<Map, ParseError> {
    Ok(Map {
        stages: node.field("stages")?.list(stage_row)?,
        children: node.field("children")?.list(child_row)?,
    })
}

fn stage_row(node: &Reader<'_>) -> Result<StageRow, ParseError> {
    Ok(StageRow {
        id: node.field("id")?.slug()?,
        title: node.field("title")?.text()?,
        hours: hours(&node.field("hours")?)?,
    })
}

fn child_row(node: &Reader<'_>) -> Result<ChildRow, ParseError> {
    Ok(ChildRow {
        uuid: node.field("uuid")?.uuid()?,
        title: node.field("title")?.text()?,
        goal: node
            .optional_field("goal")?
            .map(|goal| goal.text())
            .transpose()?,
        hours: hours(&node.field("hours")?)?,
    })
}

fn sources(node: &Reader<'_>) -> Result<Sources, ParseError> {
    Ok(Sources {
        books: node.field("books")?.list(book)?,
        pages: node.field("pages")?.list(page)?,
    })
}

fn book(node: &Reader<'_>) -> Result<Book, ParseError> {
    Ok(Book {
        title: node.field("title")?.text()?,
        authors: node.field("authors")?.texts()?,
        isbn: node.field("isbn")?.text()?,
        chapter: node.field("chapter")?.text()?,
        checked_at: node.field("checked_at")?.date()?,
    })
}

fn page(node: &Reader<'_>) -> Result<Page, ParseError> {
    Ok(Page {
        title: node.field("title")?.text()?,
        url: node.field("url")?.text()?,
        checked_at: node.field("checked_at")?.date()?,
    })
}
