use tolearn_core::program::Sources;

pub(super) fn merged(mut known: Sources, cited: Sources) -> Sources {
    for page in cited.pages {
        if !known.pages.iter().any(|seen| seen.url == page.url) {
            known.pages.push(page);
        }
    }
    for book in cited.books {
        if !known
            .books
            .iter()
            .any(|seen| seen.isbn == book.isbn && seen.chapter == book.chapter)
        {
            known.books.push(book);
        }
    }
    known
}
