#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "generate gate: a panic here is the report"
)]

#[path = "../../tests-support/scratch.rs"]
mod scratch;

use std::cell::Cell;
use std::path::{Path, PathBuf};

use tolearn_generate::sources::{Outcome, Sources, Verdict, Verified};
use tolearn_offline::page::{AsFetched, PageError, Source};
use tolearn_offline::store::Store;

const PROGRAM: &str = "5f0c7a1e-4d0f-4c8e-9a51-6f1d2e3c4b5a";
const OTHER: &str = "7d2e9b3a-1c4f-4e8d-8b62-0a9f3e5d6c7b";
const ARTICLE: &str = "https://a.test/article";
const MENU: &str = "https://a.test/menu";
const MISSING: &str = "https://a.test/missing";
const BUDGET: u64 = 16 * 1024 * 1024;

#[derive(Default)]
struct Shelf {
    calls: Cell<usize>,
}

impl Source for Shelf {
    fn fetch(&self, url: &str) -> Result<Vec<u8>, PageError> {
        self.calls.set(self.calls.get() + 1);
        let fixture = match url {
            ARTICLE => "semantic.html",
            MENU => "menu-only.html",
            _ => {
                return Err(PageError::Unreachable(
                    url.to_owned(),
                    "нет сети".to_owned(),
                ));
            }
        };
        Ok(std::fs::read(format!("../fixtures/valid/reader/{fixture}")).unwrap())
    }
}

struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn scratch(name: &str) -> Scratch {
    Scratch(scratch::named(&format!("pages-{name}")))
}

fn store(data: &Path, budget: u64) -> Store {
    Store::open(&data.join("cache"), budget).unwrap()
}

fn page(shelf: &Shelf, data: &Path, program: &str, at: i64, url: &str) -> Outcome<Verified> {
    let mut store = store(data, BUDGET);
    Sources::new(shelf, &AsFetched, &mut store, data, program, at)
        .page(url)
        .unwrap()
}

fn passed(outcome: Outcome<Verified>) -> Verified {
    match outcome.verdict {
        Verdict::Passed(page) => page,
        Verdict::Refused(reason) => panic!("отказ: {reason}"),
    }
}

fn refused(outcome: Outcome<Verified>) -> String {
    match outcome.verdict {
        Verdict::Refused(reason) => reason,
        Verdict::Passed(page) => panic!("прошло: {page:?}"),
    }
}

#[test]
fn a_readable_page_passes_and_is_kept_outside_the_program() {
    let dir = scratch("pass");
    let mut store = store(&dir.0, BUDGET);
    let shelf = Shelf::default();

    let outcome = Sources::new(&shelf, &AsFetched, &mut store, &dir.0, PROGRAM, 1_000)
        .page(ARTICLE)
        .unwrap();

    assert_eq!(outcome.checked_at, 1_000);
    let page = passed(outcome);
    assert_eq!(page.url, ARTICLE);
    assert!(page.title.is_some(), "{page:?}");
    assert!(page.text.chars().count() >= 500, "{}", page.text);
    assert!(store.held(ARTICLE).unwrap().is_some());
    let pages = dir.0.join("cache").join(PROGRAM).join("pages");
    assert_eq!(std::fs::read_dir(pages).unwrap().count(), 1);
    assert!(!dir.0.join("programs").exists());
}

#[test]
fn a_page_without_readable_text_is_refused_and_remembered() {
    let dir = scratch("menu");
    let reason = refused(page(&Shelf::default(), &dir.0, PROGRAM, 1_000, MENU));
    let again = Shelf::default();

    let cached = refused(page(&again, &dir.0, PROGRAM, 2_000, MENU));

    assert!(!reason.is_empty());
    assert_eq!(cached, reason);
    assert_eq!(again.calls.get(), 0);
}

#[test]
fn a_page_that_does_not_load_is_refused_and_checked_again_next_time() {
    let dir = scratch("missing");
    let reason = refused(page(&Shelf::default(), &dir.0, PROGRAM, 1_000, MISSING));
    let again = Shelf::default();

    let _ = page(&again, &dir.0, PROGRAM, 2_000, MISSING);

    assert!(reason.contains("нет сети"), "{reason}");
    assert_eq!(again.calls.get(), 1);
}

#[test]
fn a_second_check_in_the_same_program_makes_no_requests() {
    let dir = scratch("again");
    let first = page(&Shelf::default(), &dir.0, PROGRAM, 1_000, ARTICLE);
    let again = Shelf::default();
    let other = Shelf::default();

    let cached = page(&again, &dir.0, PROGRAM, 2_000, ARTICLE);
    let _ = page(&other, &dir.0, OTHER, 3_000, ARTICLE);

    assert_eq!(again.calls.get(), 0);
    assert_eq!(cached, first);
    assert_eq!(cached.checked_at, 1_000);
    assert_eq!(other.calls.get(), 1);
}

#[test]
fn raw_html_gives_way_to_the_disk_budget_but_the_check_stays() {
    let dir = scratch("budget");
    let mut store = store(&dir.0, 0);
    let shelf = Shelf::default();

    let outcome = Sources::new(&shelf, &AsFetched, &mut store, &dir.0, PROGRAM, 1_000)
        .page(ARTICLE)
        .unwrap();
    let again = Shelf::default();

    let _ = passed(outcome);
    assert!(store.held(ARTICLE).unwrap().is_none());
    assert_eq!(store.size().unwrap(), 0);
    let _ = passed(page(&again, &dir.0, PROGRAM, 2_000, ARTICLE));
    assert_eq!(again.calls.get(), 0);
}

#[test]
fn an_unusable_cache_is_a_generation_error() {
    let dir = scratch("blocked");
    std::fs::create_dir_all(&dir.0).unwrap();
    let file = dir.0.join("file");
    std::fs::write(&file, b"").unwrap();
    let mut store = store(&dir.0, BUDGET);
    let shelf = Shelf::default();

    let error = Sources::new(&shelf, &AsFetched, &mut store, &file, PROGRAM, 1_000)
        .page(ARTICLE)
        .unwrap_err();

    assert_eq!(error.code(), "generate.cache");
}
