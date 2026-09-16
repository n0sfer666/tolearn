#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "generate gate: a panic here is the report"
)]

#[path = "../../tests-support/scratch.rs"]
mod scratch;

use std::cell::Cell;
use std::path::{Path, PathBuf};

use tolearn_generate::sources::{Outcome, Sources, Verdict};
use tolearn_offline::book::{Book, Wanted};
use tolearn_offline::page::{AsFetched, PageError, Source};
use tolearn_offline::store::Store;

const PROGRAM: &str = "5f0c7a1e-4d0f-4c8e-9a51-6f1d2e3c4b5a";
const SICP: &str = "9780262510875";
const UNKNOWN: &str = "qzxwvjkplmtr";

#[derive(Default)]
struct Library {
    down: bool,
    calls: Cell<usize>,
}

impl Source for Library {
    fn fetch(&self, url: &str) -> Result<Vec<u8>, PageError> {
        self.calls.set(self.calls.get() + 1);
        if self.down {
            return Err(PageError::Unreachable(
                url.to_owned(),
                "нет сети".to_owned(),
            ));
        }
        let fixture = if url.contains(UNKNOWN) {
            "none.json"
        } else {
            "isbn.json"
        };
        Ok(std::fs::read(format!("../fixtures/openlibrary/{fixture}")).unwrap())
    }
}

struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn scratch(name: &str) -> Scratch {
    Scratch(scratch::named(&format!("books-{name}")))
}

fn book(library: &Library, data: &Path, at: i64, wanted: &Wanted) -> Outcome<Book> {
    let mut store = Store::open(&data.join("cache"), 1024).unwrap();
    Sources::new(library, &AsFetched, &mut store, data, PROGRAM, at)
        .book(wanted)
        .unwrap()
}

fn wanted(isbn: Option<&str>, title: &str) -> Wanted {
    Wanted {
        isbn: isbn.map(str::to_owned),
        title: title.to_owned(),
        author: "zzqxvw".to_owned(),
    }
}

fn passed(outcome: Outcome<Book>) -> Book {
    match outcome.verdict {
        Verdict::Passed(book) => book,
        Verdict::Refused(reason) => panic!("отказ: {reason}"),
    }
}

fn refused(outcome: Outcome<Book>) -> String {
    match outcome.verdict {
        Verdict::Refused(reason) => reason,
        Verdict::Passed(book) => panic!("нашлась: {book:?}"),
    }
}

#[test]
fn a_found_book_and_a_missing_one_are_both_remembered() {
    let dir = scratch("remembered");
    let sicp = wanted(Some(SICP), "");
    let unknown = wanted(None, UNKNOWN);
    let found = passed(book(&Library::default(), &dir.0, 1_000, &sicp));
    let missing = refused(book(&Library::default(), &dir.0, 1_000, &unknown));
    let again = Library::default();

    assert_eq!(passed(book(&again, &dir.0, 2_000, &sicp)), found);
    assert_eq!(refused(book(&again, &dir.0, 2_000, &unknown)), missing);
    assert_eq!(
        found.title,
        "Structure and Interpretation of Computer Programs (SICP)"
    );
    assert!(!missing.is_empty());
    assert_eq!(again.calls.get(), 0);
}

#[test]
fn a_book_lookup_without_network_is_refused_but_not_remembered() {
    let dir = scratch("dark");
    let sicp = wanted(Some(SICP), "");
    let dark = Library {
        down: true,
        ..Library::default()
    };
    let reason = refused(book(&dark, &dir.0, 1_000, &sicp));
    let again = Library::default();

    let _ = passed(book(&again, &dir.0, 2_000, &sicp));

    assert!(reason.contains("нет сети"), "{reason}");
    assert_eq!(again.calls.get(), 1);
}
