use tolearn_core::block::{self, Kind};
use tolearn_core::program::Violation;
use tolearn_core::stage;
use tolearn_generate::online;
use tolearn_generate::stage::{self as staged, Drafted, Flaw, Gathered, Place};

use crate::support::Up;
use crate::web::{ARTICLE, Bench, chiptune, gathered, scripted};

fn drafted(label: &str, answer: &str) -> (Drafted, Gathered, String) {
    let program = chiptune();
    let (gathered, _) = gathered(&mut Bench::new(label), &program, &["sources.txt"]);
    let model = scripted(&[answer]);
    let place = Place::find(&program, "voices").unwrap();
    let drafted = staged::text(&online(&Up, &model).unwrap(), &place, &gathered).unwrap();
    (drafted, gathered, model.prompts().remove(0))
}

#[test]
fn the_answer_becomes_a_stage_whose_ids_are_counted_by_core() {
    let (drafted, gathered, _) = drafted("text-stage", "stage.txt");
    assert!(drafted.said.starts_with("Этап готов."));
    let draft = drafted.draft.unwrap();
    let built = &draft.stage;

    assert_eq!(built.id, "voices");
    assert_eq!(built.title, "Голоса чипа");
    assert_eq!(built.blocks.len(), 6);
    assert_eq!(built.practice.task.len(), 1);
    let every: Vec<_> = built.blocks.iter().chain(&built.practice.task).collect();
    let ids: Vec<&str> = every.iter().map(|block| block.id.as_str()).collect();
    assert_eq!(
        ids,
        block::ids(every.iter().map(|block| block.text.as_str()))
    );

    let violations = stage::check(built);
    assert!(
        matches!(
            violations.as_slice(),
            [Violation::BlockWithoutAsset {
                kind: Kind::Diagram,
                ..
            }]
        ),
        "{violations:?}"
    );

    let checks: Vec<&str> = built
        .practice
        .constraints
        .iter()
        .chain(&built.practice.acceptance)
        .map(|check| check.id.as_str())
        .collect();
    assert_eq!(checks, ["c1", "c2", "a1", "a2"]);
    assert_eq!(built.practice.constraints[1].check, None);
    assert_eq!(
        built.practice.acceptance[0].check.as_deref(),
        Some("file voices.ftm")
    );
    let questions: Vec<&str> = built
        .questions
        .iter()
        .map(|question| question.id.as_str())
        .collect();
    assert_eq!(questions, ["q1", "q2", "q3"]);

    let image = &built.blocks[3];
    assert_eq!(image.kind, Kind::Image);
    assert_eq!(image.text, gathered.images[0].block.text);
    assert_eq!(image.asset, gathered.images[0].block.asset);
    assert!(image.license.is_some() && image.attribution.is_some() && image.source.is_some());
    assert_eq!(built.blocks[4].lang.as_deref(), Some("text"));
    assert_eq!(draft.terms, ["импульсная волна", "скважность"]);
    assert_eq!(draft.tools, ["FamiTracker"]);
    assert!(draft.flaws.is_empty(), "{:?}", draft.flaws);
}

#[test]
fn the_prompt_lists_the_checked_sources_under_their_ids() {
    let (_, _, prompt) = drafted("text-prompt", "stage.txt");

    assert!(
        prompt.contains("b1: книга «Structure and Interpretation of Computer Programs (SICP)»")
    );
    assert!(prompt.contains("глава: 2. Push Start Button"));
    assert!(prompt.contains("p1: страница «"));
    assert!(prompt.contains(&format!("», {ARTICLE}\n")));
    assert!(prompt.contains("i1: картинка «Импульсная волна со скважностью 25%»"));
    assert!(prompt.contains("→ 1. Голоса чипа (voices)"));
    assert!(prompt.contains("Язык программы: ru"));
}

#[test]
fn references_beyond_the_checked_sources_are_flaws() {
    let (drafted, _, _) = drafted("text-foreign", "stage-foreign.txt");
    let draft = drafted.draft.unwrap();
    let blocks = &draft.stage.blocks;

    assert_eq!(
        draft.flaws,
        [
            Flaw::UnknownSource {
                block: blocks[0].id.clone(),
                source: "p9".to_owned(),
            },
            Flaw::ForeignLink {
                block: blocks[1].id.clone(),
                url: "https://evil.test/free-samples".to_owned(),
            },
            Flaw::UnknownImage {
                block: blocks[2].id.clone(),
                image: "i7".to_owned(),
            },
        ]
    );
}

#[test]
fn an_unreadable_answer_is_a_flaw_not_an_error() {
    let (prose, _, _) = drafted("text-prose", "этап напишу позже");
    let (table, _, _) = drafted(
        "text-table",
        r#"{"blocks": [{"kind": "table", "text": "Каналы"}], "practice": {}}"#,
    );

    assert!(matches!(prose.draft, Err(Flaw::Unreadable(_))));
    assert!(matches!(table.draft, Err(Flaw::Unreadable(reason)) if reason.contains("table")));
}
