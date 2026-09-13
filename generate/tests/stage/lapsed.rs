use tolearn_core::Hours;
use tolearn_core::program::StageRow;
use tolearn_core::state::Lapse;
use tolearn_generate::fork;
use tolearn_generate::online;
use tolearn_generate::plan::MAX_STAGES;
use tolearn_generate::stage::{self, Place, TEXT_PROMPT_CHARS};
use tolearn_generate::unpassed::{UNPASSED_CHARS, block};

use crate::forking::{ALONE, FORKED_AT, after, begun, said};
use crate::support::{Scripted, Up};
use crate::web::{Bench, TAIL, chiptune, gathered, many};

fn lapses() -> Vec<Lapse> {
    vec![Lapse {
        stage: "Трекер".to_owned(),
        question: "Что такое строка паттерна?".to_owned(),
        missed: vec!["громкость".to_owned()],
    }]
}

fn forked(name: &str, lapses: &[Lapse]) -> String {
    let mut bench = Bench::new(name);
    let uuid = begun(&mut bench);
    let model = said(&[ALONE]);
    fork::propose(
        &online(&Up, &model).unwrap(),
        &bench.dir,
        &after(&uuid, "tracker"),
        FORKED_AT,
        lapses,
    )
    .unwrap();
    model.prompts()[0].clone()
}

#[test]
fn the_fork_prompt_gains_the_unpassed_block_and_is_unchanged_without_lapses() {
    let plain = forked("lapsed-fork-plain", &[]);
    let lapsed = forked("lapsed-fork", &lapses());

    let start = lapsed.find(&block(&lapses()).unwrap()).unwrap();
    let end = lapsed.find("Объясни в next.why").unwrap();

    assert!(!plain.contains("Незачтённые вопросы"));
    assert!(lapsed[start..end].contains("recommended"));
    assert_eq!(format!("{}{}", &lapsed[..start], &lapsed[end..]), plain);
}

#[test]
fn the_text_prompt_gains_the_unpassed_block_and_is_unchanged_without_lapses() {
    let program = chiptune();
    let (gathered, _) = gathered(&mut Bench::new("lapsed-text"), &program, &["sources.txt"]);
    let written = |lapses: &[Lapse]| {
        let model = Scripted::new(vec!["{}".to_owned()]);
        let place = Place {
            lapses,
            ..Place::find(&program, "voices").unwrap()
        };
        let _ = stage::text(&online(&Up, &model).unwrap(), &place, &gathered);
        model.prompts()[0].clone()
    };

    let plain = written(&[]);
    let lapsed = written(&lapses());

    let start = lapsed.find(&block(&lapses()).unwrap()).unwrap() - 1;
    let end = start + lapsed[start..].find("Язык программы:").unwrap();
    assert!(!plain.contains("Незачтённые вопросы"));
    assert!(lapsed[start..end].contains("этап"));
    assert_eq!(format!("{}{}", &lapsed[..start], &lapsed[end..]), plain);
}

#[test]
fn the_text_prompt_fits_its_ceiling_with_the_fullest_block_map_and_pages() {
    let mut program = chiptune();
    let title = "Этап с длинным названием про звук старых игровых приставок ".repeat(2);
    while program.map.stages.len() < MAX_STAGES {
        program.map.stages.push(StageRow {
            id: format!("extra-{}", program.map.stages.len()),
            title: title.clone(),
            hours: Hours { min: 1, max: 2 },
        });
    }
    let (gathered, _) = gathered(&mut Bench::new("lapsed-ceiling"), &program, &[&many()]);
    let lapses = [Lapse {
        stage: "Трекер".to_owned(),
        question: "я".repeat(5_000),
        missed: vec!["всё".to_owned()],
    }];
    let full = block(&lapses).unwrap();
    assert_eq!(full.chars().count(), UNPASSED_CHARS);

    let model = Scripted::new(vec!["{}".to_owned()]);
    let place = Place {
        lapses: &lapses,
        ..Place::find(&program, "voices").unwrap()
    };
    let _ = stage::text(&online(&Up, &model).unwrap(), &place, &gathered);
    let prompt = &model.prompts()[0];
    let size = prompt.chars().count();

    assert!(size <= TEXT_PROMPT_CHARS, "{size} > {TEXT_PROMPT_CHARS}");
    assert!(prompt.contains(&full));
    assert!(!prompt.contains(TAIL));
}
