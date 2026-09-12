mod cache;
mod excerpt;
mod image;
mod verdict;

pub use excerpt::{PAGE_CHARS, excerpt};
pub use image::Illustration;
pub use verdict::{Outcome, Verdict, Verified};

use std::fmt;
use std::path::Path;

use serde::Serialize;
use tolearn_offline::book::{self, Book, Wanted};
use tolearn_offline::commons::{self, Found};
use tolearn_offline::page::{self, Fetching, Prerenderer, Source};
use tolearn_offline::reader;
use tolearn_offline::store::{Fetched, Store, StoreError};

use crate::error::GenerateError;

use cache::Cache;

const PAGES: &str = "pages";
const BOOKS: &str = "books";
const FETCH_TIMEOUT_SECS: u64 = 30;
const NO_TEXT: &str = "на странице нет связного текста: читалка ничего не извлекла";
const NOT_FOUND: &str = "Open Library такой книги не знает";

pub struct Sources<'a> {
    source: &'a dyn Source,
    renderer: &'a dyn Prerenderer,
    store: &'a mut Store,
    cache: Cache,
    program: String,
    at: i64,
}

impl<'a> Sources<'a> {
    pub fn new(
        source: &'a dyn Source,
        renderer: &'a dyn Prerenderer,
        store: &'a mut Store,
        data: &Path,
        program: &str,
        at: i64,
    ) -> Self {
        Self {
            source,
            renderer,
            store,
            cache: Cache::at(data, program),
            program: program.to_owned(),
            at,
        }
    }

    pub fn page(&mut self, url: &str) -> Result<Outcome<Verified>, GenerateError> {
        if let Some(outcome) = self.cache.load(PAGES, url)? {
            return Ok(outcome);
        }
        let fetching = Fetching {
            timeout: FETCH_TIMEOUT_SECS,
            domains: Some(Vec::new()),
        };
        let saved = match page::save(url, self.source, self.renderer, &fetching) {
            Ok(saved) => saved,
            Err(error) => return Ok(self.refused(error.to_string())),
        };
        self.keep(url, &saved.html)?;
        let reading = reader::read(&String::from_utf8_lossy(&saved.html), url);
        let verdict = if reading.extracted {
            Verdict::Passed(Verified {
                url: url.to_owned(),
                title: reading.title.or(saved.title),
                text: reading.text,
            })
        } else {
            Verdict::Refused(NO_TEXT.to_owned())
        };
        self.remember(PAGES, url, verdict)
    }

    pub fn book(&self, wanted: &Wanted) -> Result<Outcome<Book>, GenerateError> {
        let key = asked(wanted);
        if let Some(outcome) = self.cache.load(BOOKS, &key)? {
            return Ok(outcome);
        }
        match book::find(self.source, wanted) {
            Ok(Some(found)) => self.remember(BOOKS, &key, Verdict::Passed(found)),
            Ok(None) => self.remember(BOOKS, &key, Verdict::Refused(NOT_FOUND.to_owned())),
            Err(error) => Ok(self.refused(error.to_string())),
        }
    }

    pub fn image(&self, query: &str, caption: &str) -> Outcome<Illustration> {
        let verdict = match commons::find(self.source, query) {
            Ok(Found::Picture(picture)) => Verdict::Passed(Illustration::new(picture, caption)),
            Ok(Found::Refused(reason)) => Verdict::Refused(reason),
            Ok(Found::Missing) => Verdict::Refused(format!(
                "на Commons не нашлось картинки по запросу «{query}»"
            )),
            Err(error) => Verdict::Refused(error.to_string()),
        };
        Outcome {
            checked_at: self.at,
            verdict,
        }
    }

    fn keep(&mut self, url: &str, html: &[u8]) -> Result<(), GenerateError> {
        let fetched = Fetched {
            kind: "page",
            bytes: html,
            etag: None,
            last_modified: None,
        };
        self.store
            .put(url, &self.program, &fetched, self.at)
            .map_err(unkept)?;
        self.store.sweep().map_err(unkept)?;
        Ok(())
    }

    fn remember<T: Serialize>(
        &self,
        kind: &str,
        key: &str,
        verdict: Verdict<T>,
    ) -> Result<Outcome<T>, GenerateError> {
        let outcome = Outcome {
            checked_at: self.at,
            verdict,
        };
        self.cache.save(kind, key, &outcome)?;
        Ok(outcome)
    }

    fn refused<T>(&self, reason: String) -> Outcome<T> {
        Outcome {
            checked_at: self.at,
            verdict: Verdict::Refused(reason),
        }
    }
}

impl fmt::Debug for Sources<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Sources")
            .field("program", &self.program)
            .field("at", &self.at)
            .finish_non_exhaustive()
    }
}

fn asked(wanted: &Wanted) -> String {
    match &wanted.isbn {
        Some(isbn) => format!("isbn:{isbn}"),
        None => format!("title:{}\nauthor:{}", wanted.title, wanted.author),
    }
}

fn unkept(error: StoreError) -> GenerateError {
    GenerateError::Cache(error.to_string())
}
