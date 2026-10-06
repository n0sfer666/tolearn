use tolearn_core::Hours;
use tolearn_core::program::StageRow;
use tolearn_generate::online;
use tolearn_generate::plan::MAX_STAGES;
use tolearn_generate::sources::PAGE_CHARS;
use tolearn_generate::stage::{
    self, MAX_PAGES, PREVIOUS_CHARS, Place, SOURCES_PROMPT_CHARS, TEXT_PROMPT_CHARS,
};

use crate::answers::{self, DELIVERABLE, LONG, QUESTION, fine};
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

#[test]
fn the_regeneration_prompt_fits_its_ceiling_with_the_longest_map_pages_and_stage() {
    let mut program = chiptune();
    let title = "Этап с длинным названием про звук старых игровых приставок ".repeat(2);
    while program.map.stages.len() < MAX_STAGES {
        program.map.stages.push(StageRow {
            id: format!("extra-{}", program.map.stages.len()),
            title: title.clone(),
            hours: Hours { min: 1, max: 2 },
        });
    }
    let (gathered, _) = gathered(&mut Bench::new("ceiling-again"), &program, &[&many()]);
    let mut previous = answers::compose("ceiling-previous", &program, "voices", &[fine()])
        .result
        .unwrap()
        .stage;
    previous.blocks[0].text = LONG.repeat(2_000);

    let model = Scripted::new(vec!["{}".to_owned()]);
    let place = Place::find(&program, "voices").unwrap();
    let _ = stage::recompose(
        &online(&Up, &model).unwrap(),
        &place,
        &gathered,
        &previous,
        &(),
    );
    let prompt = &model.prompts()[0];
    let size = prompt.chars().count();
    let before = &prompt[prompt.find("Прежний текст этапа:").unwrap()..];

    assert!(size <= TEXT_PROMPT_CHARS, "{size} > {TEXT_PROMPT_CHARS}");
    assert!(before.chars().count() > PREVIOUS_CHARS);
    for part in [
        "Практика:",
        DELIVERABLE,
        "Вопросы:",
        QUESTION,
        "Теория:",
        LONG,
    ] {
        assert!(before.contains(part), "{part}");
    }
}
