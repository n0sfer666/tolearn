use tolearn_offline::book::Book;

use crate::sources::{Illustration, Verified};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Gathered {
    pub books: Vec<Chaptered>,
    pub pages: Vec<Visited>,
    pub images: Vec<Illustration>,
    pub dropped: Vec<Dropped>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chaptered {
    pub book: Book,
    pub chapter: String,
    pub checked_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Visited {
    pub page: Verified,
    pub checked_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dropped {
    pub what: String,
    pub reason: String,
}

impl Gathered {
    pub fn image(&self, id: &str) -> Option<&Illustration> {
        self.images.get(numbered(id, 'i')?)
    }

    pub fn knows(&self, id: &str) -> bool {
        numbered(id, 'b').is_some_and(|index| index < self.books.len())
            || numbered(id, 'p').is_some_and(|index| index < self.pages.len())
            || self.image(id).is_some()
    }

    pub fn visited(&self, url: &str) -> bool {
        self.pages.iter().any(|visited| visited.page.url == url)
    }
}

fn numbered(id: &str, prefix: char) -> Option<usize> {
    let number: usize = id.strip_prefix(prefix)?.parse().ok()?;
    number.checked_sub(1)
}
