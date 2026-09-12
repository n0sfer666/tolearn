mod error;

pub use error::BookError;

use serde::{Deserialize, Serialize};
use url::form_urlencoded::Serializer;

use crate::page::Source;

const SEARCH: &str = "https://openlibrary.org/search.json";
const FIELDS: &str = "title,author_name,isbn";
const ISBN_13: usize = 13;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wanted {
    pub isbn: Option<String>,
    pub title: String,
    pub author: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Book {
    pub title: String,
    pub authors: Vec<String>,
    pub isbn: Option<String>,
}

#[derive(Deserialize)]
struct Answer {
    docs: Vec<Doc>,
}

#[derive(Deserialize)]
struct Doc {
    title: String,
    #[serde(default)]
    author_name: Vec<String>,
    #[serde(default)]
    isbn: Vec<String>,
}

pub fn find(source: &dyn Source, wanted: &Wanted) -> Result<Option<Book>, BookError> {
    let isbn = wanted
        .isbn
        .as_deref()
        .map(digits)
        .filter(|number| !number.is_empty());
    let bytes = source
        .fetch(&search(isbn.as_deref(), wanted))
        .map_err(BookError::Unreachable)?;
    let answer: Answer =
        serde_json::from_slice(&bytes).map_err(|error| BookError::Malformed(error.to_string()))?;
    Ok(answer.docs.into_iter().next().map(|doc| Book {
        isbn: isbn.or_else(|| listed(&doc.isbn)),
        title: doc.title,
        authors: doc.author_name,
    }))
}

fn search(isbn: Option<&str>, wanted: &Wanted) -> String {
    let mut query = Serializer::new(String::new());
    match isbn {
        Some(number) => query.append_pair("isbn", number),
        None => query
            .append_pair("title", &wanted.title)
            .append_pair("author", &wanted.author),
    };
    query
        .append_pair("fields", FIELDS)
        .append_pair("limit", "1");
    format!("{SEARCH}?{}", query.finish())
}

fn digits(isbn: &str) -> String {
    isbn.chars()
        .filter(|sign| sign.is_ascii_digit() || sign.eq_ignore_ascii_case(&'x'))
        .map(|sign| sign.to_ascii_uppercase())
        .collect()
}

fn listed(numbers: &[String]) -> Option<String> {
    numbers
        .iter()
        .find(|number| number.len() == ISBN_13)
        .or_else(|| numbers.first())
        .cloned()
}
