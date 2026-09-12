use tolearn_generate::online;
use tolearn_generate::sources::PAGE_CHARS;
use tolearn_generate::stage::{self, MAX_PAGES, Place, SOURCES_PROMPT_CHARS, TEXT_PROMPT_CHARS};

use crate::support::{Scripted, Up};
use crate::web::{Bench, TAIL, chiptune, gathered, many};

#[test]
fn the_sources_prompt_fits_its_ceiling() {
    let (_, prompts) = gathered(
        &mut Bench::new("ceiling-sources"),
        &chiptune(),
        &["sources.txt"],
    );
    let size = prompts[0].chars().count();

    assert!(
        size <= SOURCES_PROMPT_CHARS,
        "{size} > {SOURCES_PROMPT_CHARS}"
    );
}

#[test]
fn the_text_prompt_fits_its_ceiling_with_the_longest_pages() {
    let program = chiptune();
    let (gathered, _) = gathered(&mut Bench::new("ceiling-text"), &program, &[&many()]);
    assert_eq!(gathered.pages.len(), MAX_PAGES);
    assert!(
        gathered
            .pages
            .iter()
            .all(|visited| visited.page.text.chars().count() > PAGE_CHARS)
    );

    let model = Scripted::new(vec!["{}".to_owned()]);
    let place = Place::find(&program, "voices").unwrap();
    stage::text(&online(&Up, &model).unwrap(), &place, &gathered).unwrap();
    let prompt = &model.prompts()[0];
    let size = prompt.chars().count();

    assert!(size <= TEXT_PROMPT_CHARS, "{size} > {TEXT_PROMPT_CHARS}");
    assert!(!prompt.contains(TAIL));
}
