#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "generate gate: a panic here is the report"
)]

mod support;

use tolearn_generate::online;
use tolearn_generate::plan::{self, Plan};

use support::{Up, model, request};

const WISH: &str = "Только чиптюн в Famitracker, без теории музыки";

fn previous() -> Plan {
    let first = model(&["split.txt"]);
    plan::plan(&online(&Up, &first).unwrap(), &request()).unwrap()
}

#[test]
fn revision_carries_the_previous_map_and_the_wish() {
    let previous = previous();
    let second = model(&["flat.txt"]);

    let revised =
        plan::revise(&online(&Up, &second).unwrap(), &request(), &previous, WISH).unwrap();

    assert_eq!(revised.title, "Чиптюн с нуля");
    let prompts = second.prompts();
    assert_eq!(prompts.len(), 1);
    for asked in [
        "Хочу писать чиптюн",
        "Прежняя карта",
        "Локальные LLM",
        "\"local-llm\"",
        "\"volatile\"",
        "Квантование и форматы",
        "Выбирать формат и квантование под своё железо.",
        "[30,50]",
        WISH,
    ] {
        assert!(prompts[0].contains(asked), "{asked}: {}", prompts[0]);
    }
}

#[test]
fn broken_revision_is_repaired_with_the_wish_kept() {
    let previous = previous();
    let second = model(&["broken.txt", "flat.txt"]);

    let revised =
        plan::revise(&online(&Up, &second).unwrap(), &request(), &previous, WISH).unwrap();

    assert_eq!(revised.stages.len(), 8);
    let prompts = second.prompts();
    assert_eq!(prompts.len(), 2);
    assert!(prompts[1].starts_with(&prompts[0]), "{}", prompts[1]);
    assert!(prompts[1].contains(WISH), "{}", prompts[1]);
    assert!(
        prompts[1].contains("«tracker» повторяется"),
        "{}",
        prompts[1]
    );
}
