use serde_json::json;
use tolearn_core::Hours;
use tolearn_generate::stage::{MAX_TERMS, MIN_THEORY_CHARS};
use tolearn_generate::{GenerateError, REPAIRS};

use crate::answers::{compose, drafted, fine, ids};
use crate::web::chiptune;

#[test]
fn a_sound_stage_passes_at_the_first_call() {
    let stage = fine();
    let run = compose(
        "compose-fine",
        &chiptune(),
        "voices",
        std::slice::from_ref(&stage),
    );

    let draft = run.result.unwrap();
    assert_eq!(drafted(&draft), ids(&stage));
    assert_eq!(run.prompts.len(), 1);
    let prompt = &run.prompts[0];
    assert!(prompt.contains(&format!("от {MIN_THEORY_CHARS} до")));
    assert!(prompt.contains(&format!("Не больше {MAX_TERMS} новых терминов")));
    assert!(prompt.contains("ровно один инструмент"));
}

#[test]
fn hours_beyond_a_stage_are_refused_without_the_model() {
    let mut program = chiptune();
    program.map.stages[0].hours = Hours { min: 5, max: 6 };
    let run = compose("compose-hours", &program, "voices", &[fine()]);

    let error = run.result.unwrap_err();
    assert_eq!(error.code(), "generate.stage-hours");
    assert!(matches!(&error, GenerateError::StageHours { stage, .. } if stage == "voices"));
    assert!(run.prompts.is_empty());
}

#[test]
fn a_stage_the_model_cannot_mend_is_refused_with_the_flaws() {
    let mut broken = fine();
    broken["blocks"][1]["sources"] = json!(["p9"]);
    let id = ids(&broken)[1].clone();
    let still = json!({"blocks": {id: broken["blocks"][1].clone()}});
    let run = compose("compose-refused", &chiptune(), "voices", &[broken, still]);

    let error = run.result.unwrap_err();
    assert_eq!(error.code(), "generate.unrepaired");
    let GenerateError::Unrepaired { what, flaws } = error else {
        panic!("{error:?}");
    };
    assert_eq!(what, "этап");
    assert!(flaws.iter().any(|flaw| flaw.contains("p9")), "{flaws:?}");
    assert_eq!(run.prompts.len(), 1 + REPAIRS);
}

#[test]
fn an_unreadable_first_answer_asks_for_the_whole_stage_again() {
    let stage = fine();
    let run = compose(
        "compose-prose",
        &chiptune(),
        "voices",
        &[json!("этап напишу позже"), stage.clone()],
    );

    assert_eq!(drafted(&run.result.unwrap()), ids(&stage));
    assert_eq!(run.prompts.len(), 2);
    assert!(run.prompts[1].starts_with(&run.prompts[0]));
    assert!(run.prompts[1].contains("Твой прошлый ответ:\nэтап напишу позже"));
    assert!(run.prompts[1].contains("ответ не прочитан"));
    assert!(run.prompts[1].contains("пришли этап целиком"));
}

#[test]
fn an_unreadable_repair_asks_for_the_same_parts_again() {
    let stage = fine();
    let mut broken = stage.clone();
    broken["blocks"][1]["sources"] = json!(["p9"]);
    let id = ids(&broken)[1].clone();
    let mended = json!({"blocks": {id.clone(): stage["blocks"][1].clone()}});
    let run = compose(
        "compose-garbled",
        &chiptune(),
        "voices",
        &[broken, json!("починю позже"), mended],
    );

    assert_eq!(drafted(&run.result.unwrap()), ids(&stage));
    assert_eq!(run.prompts.len(), 3);
    let again = &run.prompts[2];
    assert!(again.contains("ответ не прочитан"));
    assert!(again.contains("p9"));
    assert!(again.contains(&format!("\"{id}\"")));
}
