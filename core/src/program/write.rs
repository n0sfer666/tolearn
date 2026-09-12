use std::fmt;

use saphyr::Yaml;

use super::parse::SCHEMA;
use super::types::{Book, ChildRow, Page, Program, StageRow};
use crate::Hours;
use crate::yaml::{dump, list, map, number, text};

pub fn write(program: &Program) -> Result<String, fmt::Error> {
    let generation = &program.generation;
    dump(&map([
        ("schema", text(SCHEMA)),
        ("uuid", text(&program.uuid)),
        ("slug", text(&program.slug)),
        ("title", text(&program.title)),
        ("goal", text(&program.goal)),
        ("level", text(&program.level)),
        (
            "generation",
            map([
                ("locale", text(&generation.locale)),
                ("volatility", text(generation.volatility.label())),
                ("request", text(&generation.request)),
            ]),
        ),
        (
            "map",
            map([
                ("stages", list(program.map.stages.iter().map(stage_row))),
                ("children", list(program.map.children.iter().map(child_row))),
            ]),
        ),
        (
            "sources",
            map([
                ("books", list(program.sources.books.iter().map(book))),
                ("pages", list(program.sources.pages.iter().map(page))),
            ]),
        ),
    ]))
}

fn stage_row(row: &StageRow) -> Yaml<'_> {
    map([
        ("id", text(&row.id)),
        ("title", text(&row.title)),
        ("hours", hours(row.hours)),
    ])
}

fn child_row(row: &ChildRow) -> Yaml<'_> {
    map([
        ("uuid", text(&row.uuid)),
        ("title", text(&row.title)),
        ("hours", hours(row.hours)),
    ])
}

fn hours(hours: Hours) -> Yaml<'static> {
    list([number(hours.min), number(hours.max)])
}

fn book(book: &Book) -> Yaml<'_> {
    map([
        ("title", text(&book.title)),
        (
            "authors",
            list(book.authors.iter().map(|author| text(author))),
        ),
        ("isbn", text(&book.isbn)),
        ("chapter", text(&book.chapter)),
        ("checked_at", text(&book.checked_at)),
    ])
}

fn page(page: &Page) -> Yaml<'_> {
    map([
        ("title", text(&page.title)),
        ("url", text(&page.url)),
        ("checked_at", text(&page.checked_at)),
    ])
}
