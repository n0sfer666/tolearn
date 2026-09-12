use serde::{Deserialize, Serialize};
use tolearn_offline::book::Book;

use crate::sources::{Illustration, Verified};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Gathered {
    pub books: Vec<Chaptered>,
    pub pages: Vec<Visited>,
    pub images: Vec<Illustration>,
    pub dropped: Vec<Dropped>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Chaptered {
    pub book: Book,
    pub chapter: String,
    pub checked_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Visited {
    pub page: Verified,
    pub checked_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dropped {
    pub what: String,
    pub reason: String,
}

impl Gathered {
    pub fn image(&self, id: &str) -> Option<&Illustration> {
        self.images.get(numbered(id, 'i')?)
    }

    pub fn book(&self, id: &str) -> Option<&Chaptered> {
        self.books.get(numbered(id, 'b')?)
    }

    pub fn page(&self, id: &str) -> Option<&Visited> {
        self.pages.get(numbered(id, 'p')?)
    }

    pub fn knows(&self, id: &str) -> bool {
        self.book(id).is_some() || self.page(id).is_some() || self.image(id).is_some()
    }

    pub fn visited(&self, url: &str) -> bool {
        self.pages.iter().any(|visited| visited.page.url == url)
    }
}

fn numbered(id: &str, prefix: char) -> Option<usize> {
    let number: usize = id.strip_prefix(prefix)?.parse().ok()?;
    number.checked_sub(1)
}
