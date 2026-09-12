#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "offline gate: a panic here is the report"
)]

use std::cell::RefCell;

use tolearn_offline::commons::{self, CommonsError, Found, MAX_BYTES, Picture, THUMB_WIDTH, free};
use tolearn_offline::page::{PageError, Source};

const API: &str = "https://commons.wikimedia.org/w/api.php?";
const THUMB: &[u8] = b"\x89PNG thumbnail";
const FOUND: &[u8] = include_bytes!("../../fixtures/commons/found.json");

struct Recorded {
    answer: &'static [u8],
    thumb: Vec<u8>,
    asked: RefCell<Vec<String>>,
}

impl Recorded {
    fn new(answer: &'static [u8]) -> Self {
        Self {
            answer,
            thumb: THUMB.to_vec(),
            asked: RefCell::default(),
        }
    }

    fn asked(&self) -> Vec<String> {
        self.asked.borrow().clone()
    }
}

impl Source for Recorded {
    fn fetch(&self, url: &str) -> Result<Vec<u8>, PageError> {
        self.asked.borrow_mut().push(url.to_owned());
        if url.starts_with(API) {
            Ok(self.answer.to_vec())
        } else {
            Ok(self.thumb.clone())
        }
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

fn picture(found: Found) -> Picture {
    match found {
        Found::Picture(picture) => picture,
        other => panic!("не картинка: {other:?}"),
    }
}

fn refused(found: Found) -> String {
    match found {
        Found::Refused(reason) => reason,
        other => panic!("не отказ: {other:?}"),
    }
}

fn params(url: &str) -> Vec<(String, String)> {
    url::Url::parse(url)
        .unwrap()
        .query_pairs()
        .into_owned()
        .collect()
}

fn pair(name: &str, value: &str) -> (String, String) {
    (name.to_owned(), value.to_owned())
}

#[test]
fn the_best_ranked_free_image_comes_as_a_thumbnail_no_wider_than_1200() {
    let commons = Recorded::new(FOUND);

    let found = picture(commons::find(&commons, "Turing machine").unwrap());

    assert_eq!(found.title, "File:Example of a Turing machine.svg");
    assert_eq!(found.license, "CC BY-SA 4.0");
    assert_eq!(found.author, "Sn KGS");
    assert_eq!(
        found.page,
        "https://commons.wikimedia.org/wiki/File:Example_of_a_Turing_machine.svg"
    );
    assert_eq!(found.extension, "png");
    assert_eq!(found.bytes, THUMB);
    let asked = commons.asked();
    assert_eq!(asked.len(), 2);
    let sent = params(&asked[0]);
    assert!(
        sent.contains(&pair("gsrsearch", "Turing machine")),
        "{sent:?}"
    );
    assert!(sent.contains(&pair("gsrnamespace", "6")), "{sent:?}");
    assert!(sent.contains(&pair("iiurlwidth", "960")), "{sent:?}");
    assert!(
        asked[1].contains("/960px-Example_of_a_Turing_machine.svg.png"),
        "{}",
        asked[1]
    );
    assert_eq!(THUMB_WIDTH, 960);
}

#[test]
fn a_license_outside_the_free_list_is_refused_without_downloading() {
    let commons = Recorded::new(include_bytes!("../../fixtures/commons/unfree.json"));

    let reason = refused(commons::find(&commons, "fjord").unwrap());

    assert!(reason.contains("GFDL"), "{reason}");
    assert!(reason.contains("Nusfjord road"), "{reason}");
    assert_eq!(commons.asked().len(), 1);
}

#[test]
fn an_unfree_top_result_gives_way_to_a_free_one() {
    let commons = Recorded::new(include_bytes!("../../fixtures/commons/mixed.json"));

    let found = picture(commons::find(&commons, "fjord").unwrap());

    assert_eq!(
        found.title,
        "File:Emirates Airbus A380-861 A6-EER MUC 2015 01.jpg"
    );
    assert_eq!(found.license, "CC BY 4.0");
    assert!(
        found.author.starts_with("HARRY HUGHES, WIGAN UK"),
        "{}",
        found.author
    );
    assert!(!found.author.contains(['<', '&']), "{}", found.author);
    assert_eq!(found.extension, "jpg");
}

#[test]
fn nothing_found_is_a_miss_not_an_error() {
    let commons = Recorded::new(include_bytes!("../../fixtures/commons/none.json"));

    assert_eq!(
        commons::find(&commons, "qzxwvjkplmtr").unwrap(),
        Found::Missing
    );
    assert_eq!(commons.asked().len(), 1);
}

#[test]
fn a_thumbnail_heavier_than_400_kb_is_refused() {
    let heavy = Recorded {
        thumb: vec![0; MAX_BYTES + 1],
        ..Recorded::new(FOUND)
    };
    let fits = Recorded {
        thumb: vec![0; MAX_BYTES],
        ..Recorded::new(FOUND)
    };

    let reason = refused(commons::find(&heavy, "Turing machine").unwrap());
    let kept = picture(commons::find(&fits, "Turing machine").unwrap());

    assert_eq!(MAX_BYTES, 400 * 1024);
    assert!(reason.contains("400 КБ"), "{reason}");
    assert_eq!(kept.bytes.len(), MAX_BYTES);
}

#[test]
fn only_cc0_public_domain_cc_by_and_cc_by_sa_are_free() {
    for code in [
        "cc0",
        "pd",
        "cc-by-4.0",
        "cc-by-2.0",
        "cc-by-sa-4.0",
        "cc-by-sa-3.0-de",
    ] {
        assert!(free(code), "{code}");
    }
    for code in [
        "",
        "gfdl",
        "cc-by-nd-4.0",
        "cc-by-nc-4.0",
        "cc-by-nc-sa-4.0",
        "fal",
    ] {
        assert!(!free(code), "{code}");
    }
}

#[test]
fn a_broken_answer_is_an_error_not_a_miss() {
    let commons = Recorded::new(include_bytes!("../../fixtures/commons/truncated.json"));

    match commons::find(&commons, "Turing machine") {
        Err(CommonsError::Malformed(_)) => {}
        other => panic!("ожидался Malformed: {other:?}"),
    }
}

#[test]
fn a_network_failure_carries_its_reason() {
    let error = commons::find(&Offline, "Turing machine").unwrap_err();

    assert!(error.to_string().contains("Wikimedia Commons"), "{error}");
    assert!(error.to_string().contains("нет сети"), "{error}");
}
