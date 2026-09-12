#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "app gate: a panic here is the report"
)]

mod support;

use serde_json::{Value, json};
use support::starter::{Case, LEVEL, flat, told};
use tolearn_app::ipc::{Context, IpcError, call};
use tolearn_core::library::Library;

fn begun(case: &Case) -> String {
    let started = case.start(&case.plan(), LEVEL).unwrap();
    started["program"].as_str().unwrap().to_owned()
}

fn regenerated(context: &Context, program: &str, stage: &str) -> Result<Value, IpcError> {
    call(
        context,
        "regenerate_stage",
        &json!({ "program": program, "node": "", "stage": stage }),
    )
}

#[test]
fn перегенерация_переписывает_этап_без_поиска_источников() {
    let case = Case::new(true, told(flat()));
    let program = begun(&case);
    let before = Library::at(&case.data).open(&program).unwrap().stages;
    let steps = case.steps().len();
    let heard = case.model.heard().len();

    let out = regenerated(&case.context, &program, "tracker").unwrap();

    assert_eq!(out, json!({ "stage": "tracker" }));
    let tree = Library::at(&case.data).open(&program).unwrap();
    assert_eq!(tree.stages, before);
    let expected: Vec<(String, String)> = ["text", "diagrams", "write"]
        .iter()
        .flat_map(|step| ["began", "ended"].map(|state| (state.to_owned(), (*step).to_owned())))
        .collect();
    assert_eq!(case.steps().split_off(steps), expected);
    assert_eq!(case.model.heard().len(), heard + 1);
}

#[test]
fn несозданный_этап_не_перегенерируют_и_модель_не_зовут() {
    let case = Case::new(true, told(flat()));
    let program = begun(&case);
    let heard = case.model.heard().len();

    let refused = regenerated(&case.context, &program, "voices").unwrap_err();

    assert_eq!(refused.code, "regenerate.ungenerated");
    assert_eq!(case.model.heard().len(), heard);
}
