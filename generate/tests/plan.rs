#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "generate gate: a panic here is the report"
)]

mod support;

use tolearn_core::Hours;
use tolearn_core::program::{StageRow, Volatility};
use tolearn_generate::plan::{self, Part, Plan};
use tolearn_generate::{GenerateError, REPAIRS, online};

use support::{Up, answer, model, request};

fn drawn(answers: &[&str]) -> (Result<Plan, GenerateError>, Vec<String>) {
    let model = model(answers);
    let result = plan::plan(&online(&Up, &model).unwrap(), &request());
    (result, model.prompts())
}

fn part(title: &str, min: u32, max: u32) -> Part {
    Part {
        title: title.to_owned(),
        goal: format!("Цель {title}"),
        hours: Hours { min, max },
    }
}

#[test]
fn flat_map_is_taken_from_the_first_answer() {
    let (result, prompts) = drawn(&["flat.txt"]);
    let plan = result.unwrap();

    assert_eq!(prompts.len(), 1);
    assert_eq!(plan.title, "Чиптюн с нуля");
    assert_eq!(plan.slug, "chiptune");
    assert_eq!(plan.volatility, Volatility::Stable);
    assert_eq!(plan.stages.len(), 8);
    assert_eq!(
        plan.stages[7],
        StageRow {
            id: "first-track".to_owned(),
            title: "Первый трек целиком".to_owned(),
            hours: Hours { min: 3, max: 4 },
        }
    );
    assert!(plan.children.is_empty());
    assert_eq!(plan.hours(), Hours { min: 19, max: 30 });
    for asked in [
        "Хочу писать чиптюн",
        "Нот не знаю",
        "ru",
        "2–4 ч",
        "70 ч",
        "25 этапов",
    ] {
        assert!(prompts[0].contains(asked), "{asked}: {}", prompts[0]);
    }
}

#[test]
fn map_over_seventy_hours_comes_split_into_subprograms() {
    let (result, prompts) = drawn(&["split.txt"]);
    let plan = result.unwrap();

    assert_eq!(prompts.len(), 1);
    assert_eq!(plan.volatility, Volatility::Volatile);
    assert!(plan.stages.is_empty());
    assert_eq!(plan.children.len(), 4);
    assert_eq!(plan.hours(), Hours { min: 170, max: 250 });
    assert_eq!(
        plan.children[1],
        Part {
            title: "Квантование и форматы".to_owned(),
            goal: "Выбирать формат и квантование под своё железо.".to_owned(),
            hours: Hours { min: 30, max: 50 },
        }
    );
}

#[test]
fn broken_map_goes_back_with_its_flaws_and_heals() {
    let (result, prompts) = drawn(&["broken.txt", "flat.txt"]);

    assert_eq!(result.unwrap().stages.len(), 8);
    assert_eq!(prompts.len(), 2);
    let repair = &prompts[1];
    assert!(repair.starts_with(&prompts[0]), "{repair}");
    assert!(repair.contains(&answer("broken.txt")), "{repair}");
    for flaw in [
        "«Голоса»",
        "«everything»",
        "8–12 ч",
        "«tracker» повторяется",
    ] {
        assert!(repair.contains(flaw), "{flaw}: {repair}");
    }
}

#[test]
fn map_that_never_heals_is_refused_with_the_list_of_flaws() {
    let (result, prompts) = drawn(&["broken.txt"]);
    let error = result.unwrap_err();

    assert_eq!(prompts.len(), REPAIRS + 1);
    assert_eq!(error.code(), "generate.unrepaired");
    let GenerateError::Unrepaired { flaws, .. } = &error else {
        panic!("{error:?}");
    };
    assert_eq!(flaws.len(), 3, "{flaws:?}");
    for flaw in flaws {
        assert!(error.to_string().contains(flaw.as_str()), "{error}");
    }
    assert!(error.to_string().contains("карту"), "{error}");
}

#[test]
fn unreadable_answer_is_a_flaw_like_any_other() {
    let (result, prompts) = drawn(&["Извините, не могу составить карту.", "flat.txt"]);

    assert!(result.is_ok());
    assert!(prompts[1].contains("нет объекта JSON"), "{}", prompts[1]);
}

#[test]
fn subprogram_expands_by_the_same_step_with_its_goal_and_hours() {
    let model = model(&["part.txt"]);
    let gate = online(&Up, &model).unwrap();
    let row = part("Квантование и форматы", 30, 50);

    let plan = plan::expand(&gate, &request(), &row, 2).unwrap();

    assert_eq!(plan.stages.len(), 3);
    let prompt = &model.prompts()[0];
    for asked in [
        "Квантование и форматы",
        "Цель Квантование и форматы",
        "30–50 ч",
        "Хочу писать чиптюн",
    ] {
        assert!(prompt.contains(asked), "{asked}: {prompt}");
    }
}

#[test]
fn third_level_is_not_split_again() {
    let model = model(&["split.txt", "part.txt"]);
    let gate = online(&Up, &model).unwrap();

    let plan = plan::expand(&gate, &request(), &part("Форматы", 30, 50), 3).unwrap();

    assert!(plan.children.is_empty());
    let prompts = model.prompts();
    assert_eq!(prompts.len(), 2);
    assert!(prompts[0].contains("не дробится"), "{}", prompts[0]);
    assert!(prompts[1].contains("на уровне 3 из 3"), "{}", prompts[1]);
}
