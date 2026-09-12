#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "offline gate: a panic here is the report"
)]

use std::cell::RefCell;

use tolearn_offline::book::{Book, BookError, Wanted, find};
use tolearn_offline::page::{PageError, Source};
use url::Url;

const BY_ISBN: &[u8] = include_bytes!("../../fixtures/openlibrary/isbn.json");
const BY_TITLE: &[u8] = include_bytes!("../../fixtures/openlibrary/title.json");
const NONE: &[u8] = include_bytes!("../../fixtures/openlibrary/none.json");
const BROKEN: &[u8] = include_bytes!("../../fixtures/openlibrary/broken.json");

const SICP: &str = "Structure and Interpretation of Computer Programs (SICP)";

#[derive(Debug)]
struct Recorded {
    answer: &'static [u8],
    asked: RefCell<Vec<String>>,
}

impl Recorded {
    fn new(answer: &'static [u8]) -> Self {
        Self {
            answer,
            asked: RefCell::default(),
        }
    }

    fn params(&self) -> Vec<(String, String)> {
        let asked = self.asked.borrow();
        assert_eq!(asked.len(), 1, "{asked:?}");
        let url = Url::parse(&asked[0]).unwrap();
        assert_eq!(url.scheme(), "https");
        assert_eq!(url.host_str(), Some("openlibrary.org"));
        assert_eq!(url.path(), "/search.json");
        url.query_pairs()
            .map(|(key, value)| (key.into_owned(), value.into_owned()))
            .collect()
    }
}

impl Source for Recorded {
    fn fetch(&self, url: &str) -> Result<Vec<u8>, PageError> {
        self.asked.borrow_mut().push(url.to_owned());
        Ok(self.answer.to_vec())
    }
}

struct Offline;

impl Source for Offline {
    fn fetch(&self, url: &str) -> Result<Vec<u8>, PageError> {
        Err(PageError::Unreachable(
            url.to_owned(),
            "нет сети".to_owned(),
        ))
    }
}

fn wanted(isbn: Option<&str>) -> Wanted {
    Wanted {
        isbn: isbn.map(str::to_owned),
        title: "Structure and Interpretation of Computer Programs".to_owned(),
        author: "Abelson".to_owned(),
    }
}

fn has(params: &[(String, String)], key: &str) -> bool {
    params.iter().any(|(name, _)| name == key)
}

#[test]
fn an_isbn_is_looked_up_by_itself() {
    let source = Recorded::new(BY_ISBN);

    let book = find(&source, &wanted(Some("978-0-262-51087-5"))).unwrap();

    assert_eq!(
        book,
        Some(Book {
            title: SICP.to_owned(),
            authors: vec![
                "Harold Abelson".to_owned(),
                "Gerald Jay Sussman".to_owned(),
                "Julie Sussman".to_owned(),
            ],
            isbn: Some("9780262510875".to_owned()),
        })
    );
    let params = source.params();
    assert!(
        params.contains(&("isbn".to_owned(), "9780262510875".to_owned())),
        "{params:?}"
    );
    assert!(
        !has(&params, "title") && !has(&params, "author"),
        "{params:?}"
    );
}

#[test]
fn without_an_isbn_the_title_and_author_are_asked() {
    let source = Recorded::new(BY_TITLE);

    let book = find(&source, &wanted(None)).unwrap().unwrap();

    assert_eq!(book.title, SICP);
    assert_eq!(
        book.authors.first().map(String::as_str),
        Some("Harold Abelson")
    );
    assert_eq!(book.isbn.as_deref(), Some("9787111135104"));
    let params = source.params();
    assert!(
        params.contains(&(
            "title".to_owned(),
            "Structure and Interpretation of Computer Programs".to_owned()
        )),
        "{params:?}"
    );
    assert!(
        params.contains(&("author".to_owned(), "Abelson".to_owned())),
        "{params:?}"
    );
    assert!(!has(&params, "isbn"), "{params:?}");
}

#[test]
fn a_book_open_library_does_not_know_is_not_found() {
    assert_eq!(find(&Recorded::new(NONE), &wanted(None)).unwrap(), None);
}

#[test]
fn a_book_without_authors_or_isbn_keeps_what_it_has() {
    let bare = r#"{"numFound":1,"docs":[{"title":"Без автора"}]}"#;

    let book = find(&Recorded::new(bare.as_bytes()), &wanted(None)).unwrap();

    assert_eq!(
        book,
        Some(Book {
            title: "Без автора".to_owned(),
            authors: Vec::new(),
            isbn: None,
        })
    );
}

#[test]
fn a_broken_answer_is_an_error_not_a_miss() {
    let error = find(&Recorded::new(BROKEN), &wanted(None)).unwrap_err();

    assert!(matches!(error, BookError::Malformed(_)), "{error:?}");
    assert!(error.to_string().contains("Open Library"), "{error}");
}

#[test]
fn a_network_failure_carries_its_reason() {
    let error = find(&Offline, &wanted(Some("9780262510875"))).unwrap_err();

    assert!(
        matches!(&error, BookError::Unreachable(PageError::Unreachable(..))),
        "{error:?}"
    );
    assert!(error.to_string().contains("нет сети"), "{error}");
}
