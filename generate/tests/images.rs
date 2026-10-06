#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "generate gate: a panic here is the report"
)]

#[path = "../../tests-support/scratch.rs"]
mod scratch;

use std::path::PathBuf;

use tolearn_core::block::{self, Kind};
use tolearn_generate::sources::{Illustration, Outcome, Sources, Verdict};
use tolearn_offline::page::{AsFetched, PageError, Source};
use tolearn_offline::store::Store;

const PROGRAM: &str = "5f0c7a1e-4d0f-4c8e-9a51-6f1d2e3c4b5a";
const API: &str = "https://commons.wikimedia.org/w/api.php?";
const THUMB: &[u8] = b"\x89PNG thumbnail";
const CAPTION: &str = "Машина Тьюринга: лента, головка и таблица переходов";

struct Commons {
    answer: &'static str,
    down: bool,
}

impl Source for Commons {
    fn fetch(&self, url: &str) -> Result<Vec<u8>, PageError> {
        if self.down {
            return Err(PageError::Unreachable(
                url.to_owned(),
                "нет сети".to_owned(),
            ));
        }
        if url.starts_with(API) {
            Ok(std::fs::read(format!("../fixtures/commons/{}", self.answer)).unwrap())
        } else {
            Ok(THUMB.to_vec())
        }
    }
}

struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn image(name: &str, commons: &Commons, query: &str) -> Outcome<Illustration> {
    let dir = Scratch(scratch::named(&format!("images-{name}")));
    let mut store = Store::open(&dir.0.join("cache"), 1024).unwrap();
    Sources::new(commons, &AsFetched, &mut store, &dir.0, PROGRAM, 1_000).image(query, CAPTION)
}

fn refused(outcome: Outcome<Illustration>) -> String {
    match outcome.verdict {
        Verdict::Refused(reason) => reason,
        Verdict::Passed(image) => panic!("прошла: {image:?}"),
    }
}

#[test]
fn a_found_picture_becomes_an_image_block_with_its_asset() {
    let commons = Commons {
        answer: "found.json",
        down: false,
    };

    let outcome = image("found", &commons, "Turing machine");

    assert_eq!(outcome.checked_at, 1_000);
    let Verdict::Passed(image) = outcome.verdict else {
        panic!("картинка не прошла: {outcome:?}");
    };
    assert_eq!(image.bytes, THUMB);
    assert!(image.file.ends_with(".png"), "{}", image.file);
    assert_eq!(image.file.len(), 16 + ".png".len());
    let block = image.block;
    assert_eq!(block.kind, Kind::Image);
    assert_eq!(block.id, block::id(CAPTION));
    assert_eq!(block.text, CAPTION);
    assert_eq!(block.asset, Some(format!("assets/{}", image.file)));
    assert_eq!(block.license.as_deref(), Some("CC BY-SA 4.0"));
    assert_eq!(
        block.attribution.as_deref(),
        Some("Sn KGS, CC BY-SA 4.0, Wikimedia Commons")
    );
    assert_eq!(
        block.source.as_deref(),
        Some("https://commons.wikimedia.org/wiki/File:Example_of_a_Turing_machine.svg")
    );
}

#[test]
fn a_picture_that_is_not_found_is_a_refusal_not_an_error() {
    let commons = Commons {
        answer: "none.json",
        down: false,
    };

    let reason = refused(image("none", &commons, "qzxwvjkplmtr"));

    assert!(reason.contains("qzxwvjkplmtr"), "{reason}");
}

#[test]
fn an_unfree_picture_is_refused_with_its_license() {
    let commons = Commons {
        answer: "unfree.json",
        down: false,
    };

    let reason = refused(image("unfree", &commons, "fjord"));

    assert!(reason.contains("GFDL"), "{reason}");
}

#[test]
fn commons_out_of_reach_is_a_refusal_not_an_error() {
    let commons = Commons {
        answer: "found.json",
        down: true,
    };

    let reason = refused(image("down", &commons, "Turing machine"));

    assert!(reason.contains("нет сети"), "{reason}");
}
