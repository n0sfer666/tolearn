use tolearn_core::program::Volatility;
use tolearn_generate::stage::{Gathered, MAX_BOOKS, MAX_IMAGES, MAX_PAGES};

use crate::web::{ARTICLE, Bench, DOCS, MENU, chiptune, gathered, many};

fn pages(gathered: &Gathered) -> Vec<&str> {
    gathered
        .pages
        .iter()
        .map(|visited| visited.page.url.as_str())
        .collect()
}

#[test]
fn sources_that_pass_cost_a_single_call() {
    let (gathered, prompts) = gathered(&mut Bench::new("pass"), &chiptune(), &["sources.txt"]);

    assert_eq!(prompts.len(), 1);
    assert_eq!(gathered.books.len(), 1);
    assert_eq!(gathered.books[0].chapter, "2. Push Start Button");
    assert_eq!(gathered.books[0].checked_at, 1_000);
    assert_eq!(pages(&gathered), [ARTICLE, DOCS]);
    assert_eq!(gathered.images.len(), 1);
    assert!(gathered.dropped.is_empty());
}

#[test]
fn refused_sources_are_replaced_once() {
    let (gathered, prompts) = gathered(
        &mut Bench::new("replaced"),
        &chiptune(),
        &["sources-bad.txt", "sources-replaced.txt"],
    );

    assert_eq!(prompts.len(), 2);
    assert!(prompts[1].contains("Open Library такой книги не знает"));
    assert!(prompts[1].contains(MENU));
    assert_eq!(pages(&gathered), [ARTICLE, DOCS]);
    assert_eq!(gathered.books.len(), 1);
    assert_eq!(gathered.images.len(), 1);
    assert_eq!(gathered.dropped.len(), 2);
}

#[test]
fn a_refused_replacement_leaves_the_stage_on_what_passed() {
    let (gathered, prompts) = gathered(
        &mut Bench::new("worse"),
        &chiptune(),
        &["sources-bad.txt", "sources-worse.txt"],
    );

    assert_eq!(prompts.len(), 2);
    assert_eq!(pages(&gathered), [ARTICLE]);
    assert!(gathered.books.is_empty());
    assert_eq!(gathered.images.len(), 1);
    assert_eq!(gathered.dropped.len(), 4);
}

#[test]
fn an_unreadable_proposal_is_asked_for_again() {
    let (gathered, prompts) = gathered(
        &mut Bench::new("unreadable"),
        &chiptune(),
        &["источники подберу позже", "sources.txt"],
    );

    assert_eq!(prompts.len(), 2);
    assert_eq!(gathered.dropped.len(), 1);
    assert_eq!(gathered.dropped[0].what, "ответ модели");
    assert_eq!(pages(&gathered), [ARTICLE, DOCS]);
}

#[test]
fn sources_beyond_the_limits_are_left_unchecked() {
    let (gathered, prompts) = gathered(&mut Bench::new("many"), &chiptune(), &[&many()]);

    assert_eq!(prompts.len(), 1);
    assert_eq!(gathered.books.len(), MAX_BOOKS);
    assert_eq!(gathered.pages.len(), MAX_PAGES);
    assert!(!gathered.images.is_empty() && gathered.images.len() <= MAX_IMAGES);
}

#[test]
fn the_prompt_names_the_place_of_the_stage_and_the_class_of_sources() {
    let (_, prompts) = gathered(&mut Bench::new("place"), &chiptune(), &["sources.txt"]);

    assert!(prompts[0].contains("→ 1. Голоса чипа (voices) — 2–3 ч"));
    assert!(prompts[0].contains("  2. Огибающая и громкость (envelope)"));
    assert!(prompts[0].contains("Это первый этап"));
    assert!(prompts[0].contains("(stable)"));

    let mut program = chiptune();
    program.generation.volatility = Volatility::Volatile;
    let (_, prompts) = gathered(&mut Bench::new("volatile"), &program, &["sources.txt"]);

    assert!(prompts[0].contains("(volatile)"));
}
