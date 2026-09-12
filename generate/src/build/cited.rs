use tolearn_core::program::{Book, Page, Sources};

use crate::stage::{Chaptered, Gathered, Visited};

use super::day::day;

pub(crate) fn cited(ids: &[String], gathered: &Gathered) -> Sources {
    Sources {
        books: ids
            .iter()
            .filter_map(|id| gathered.book(id))
            .filter_map(book)
            .collect(),
        pages: ids
            .iter()
            .filter_map(|id| gathered.page(id))
            .map(page)
            .collect(),
    }
}

fn book(found: &Chaptered) -> Option<Book> {
    let isbn = found
        .book
        .isbn
        .clone()
        .filter(|isbn| !isbn.trim().is_empty())?;
    if found.chapter.trim().is_empty() {
        return None;
    }
    Some(Book {
        title: found.book.title.clone(),
        authors: found.book.authors.clone(),
        isbn,
        chapter: found.chapter.clone(),
        checked_at: day(found.checked_at),
    })
}

fn page(visited: &Visited) -> Page {
    let url = visited.page.url.clone();
    Page {
        title: visited
            .page
            .title
            .clone()
            .filter(|title| !title.trim().is_empty())
            .unwrap_or_else(|| url.clone()),
        url,
        checked_at: day(visited.checked_at),
    }
}
