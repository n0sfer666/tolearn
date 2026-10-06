use serde_json::json;
use tolearn_generate::online;
use tolearn_generate::stage::{self, Place};

use crate::answers::{compose, fine, ids};
use crate::support::{Scripted, Up};
use crate::web::{Bench, chiptune, gathered};

const RULE: [&str; 6] = [
    "Вопросы — по теории этапа и проверяют её понимание, а не пересказ того, что пишет источник",
    "почему так устроено",
    "что будет, если это изменить",
    "как применить в практике этапа",
    "Вопрос, ответ на который — дословная фраза из источника, не годится",
    "У каждого вопроса эталонный ответ, и он передаёт суть, которую ученик должен назвать своими словами",
];

fn pinned(prompt: &str) {
    for part in RULE {
        assert!(prompt.contains(part), "{part}");
    }
    assert!(!prompt.contains("Вопросы проверяют понимание теории"));
}

#[test]
fn the_stage_prompt_asks_questions_on_understanding() {
    let run = compose("understanding-text", &chiptune(), "voices", &[fine()]);

    pinned(&run.prompts[0]);
}

#[test]
fn the_repair_prompt_asks_questions_on_understanding() {
    let mut broken = fine();
    broken["blocks"][1]["sources"] = json!(["p9"]);
    let id = ids(&broken)[1].clone();
    let text = "У чипа NES пять каналов.";
    let mended = json!({"blocks": {id: {"kind": "paragraph", "text": text, "sources": ["p1"]}}});
    let run = compose(
        "understanding-mend",
        &chiptune(),
        "voices",
        &[broken, mended],
    );

    assert_eq!(run.prompts.len(), 2);
    assert!(run.prompts[1].starts_with("Ты чинишь один этап"));
    pinned(&run.prompts[1]);
}

#[test]
fn the_regeneration_prompt_asks_questions_on_understanding() {
    let program = chiptune();
    let (gathered, _) = gathered(
        &mut Bench::new("understanding-again"),
        &program,
        &["sources.txt"],
    );
    let previous = compose("understanding-previous", &program, "voices", &[fine()])
        .result
        .unwrap()
        .stage;
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
    assert!(prompt.contains("Прежний текст этапа:"));
    pinned(prompt);
}
