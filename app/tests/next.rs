#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "app gate: a panic here is the report"
)]

mod support;

use std::sync::Arc;

use serde_json::{Value, json};
use support::planner::Net;
use support::starter::{Case, LEVEL, flat, told};
use tolearn_app::ipc::{Context, IpcError, call};
use tolearn_core::library::Library;

const FORK: &str = r#"{"next": {"why": "Голоса идут сразу за первым звуком", "recommended": false}, "alternatives": [{"id": "noise", "title": "Шумовой канал", "hours": [2, 3], "why": "Ударные без шума не собрать", "recommended": true}]}"#;

fn answers() -> Vec<String> {
    let mut answers = flat();
    answers.push(FORK.to_owned());
    answers.extend(flat().into_iter().skip(1));
    answers
}

fn begun(case: &Case) -> String {
    let started = case.start(&case.plan(), LEVEL).unwrap();
    started["program"].as_str().unwrap().to_owned()
}

fn forked(context: &Context, program: &str) -> Result<Value, IpcError> {
    call(
        context,
        "fork",
        &json!({ "program": program, "node": "", "stage": "tracker" }),
    )
}

fn taken(context: &Context, program: &str, choice: u32) -> Result<Value, IpcError> {
    call(
        context,
        "take_next",
        &json!({ "program": program, "node": "", "stage": "tracker", "choice": choice }),
    )
}

fn offline(case: &Case) -> Context {
    case.context.clone().with_reach(Arc::new(Net(false)))
}

#[test]
fn развилка_предлагает_следующий_этап_и_выбранный_ложится_в_программу() {
    let case = Case::new(true, told(answers()));
    let program = begun(&case);
    let before = case.steps().len();

    let fork = forked(&case.context, &program).unwrap();
    let taken = taken(&case.context, &program, 1).unwrap();

    let variants = fork["variants"].as_array().unwrap();
    assert_eq!(variants.len(), 2);
    assert_eq!(variants[0]["id"], json!("voices"));
    assert_eq!(variants[0]["recommended"], json!(false));
    assert_eq!(
        variants[1],
        json!({
            "id": "noise",
            "title": "Шумовой канал",
            "hours": { "min": 2, "max": 3 },
            "why": "Ударные без шума не собрать",
            "recommended": true,
        })
    );
    assert_eq!(taken, json!({ "node": program, "stage": "noise" }));
    let tree = Library::at(&case.data).open(&program).unwrap();
    assert_eq!(tree.program.map.stages[1].id, "noise");
    assert_eq!(tree.stages.keys().collect::<Vec<_>>(), ["noise", "tracker"]);
    let read = call(
        &case.context,
        "stage",
        &json!({ "program": program, "node": "", "stage": "noise" }),
    );
    assert_eq!(read.unwrap()["program"], json!(program));
    let steps: Vec<(String, String)> = case.steps().split_off(before);
    let expected: Vec<(String, String)> = ["fork", "sources", "text", "diagrams", "write"]
        .iter()
        .flat_map(|step| ["began", "ended"].map(|state| (state.to_owned(), (*step).to_owned())))
        .collect();
    assert_eq!(steps, expected);
    assert_eq!(case.model.heard().len(), 6);
}

#[test]
fn без_сети_развилка_берётся_из_кэша_а_новую_и_следующий_этап_не_зовут() {
    let case = Case::new(true, told(answers()));
    let program = begun(&case);
    let away = offline(&case);

    let missed = forked(&away, &program).unwrap_err();
    let fork = forked(&case.context, &program).unwrap();
    let heard = case.model.heard().len();
    let kept = forked(&away, &program).unwrap();
    let refused = taken(&away, &program, 1).unwrap_err();

    assert_eq!(missed.code, "generate.offline");
    assert_eq!(kept, fork);
    assert_eq!(refused.code, "generate.offline");
    assert_eq!(case.model.heard().len(), heard);
    let tree = Library::at(&case.data).open(&program).unwrap();
    assert_eq!(tree.stages.keys().collect::<Vec<_>>(), ["tracker"]);
}

#[test]
fn после_последнего_этапа_карты_развилки_нет_и_модель_не_зовут() {
    let case = Case::new(true, told(answers()));
    let mut plan = case.plan();
    plan["stages"].as_array_mut().unwrap().truncate(1);
    let started = case.start(&plan, LEVEL).unwrap();
    let program = started["program"].as_str().unwrap();
    let heard = case.model.heard().len();

    let refused = forked(&case.context, program).unwrap_err();

    assert_eq!(refused.code, "next.end");
    assert_eq!(case.model.heard().len(), heard);
}
