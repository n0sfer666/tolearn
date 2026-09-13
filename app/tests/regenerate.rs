#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "app gate: a panic here is the report"
)]

mod support;

use serde_json::{Value, json};
use support::starter::{Case, LEVEL, fine, flat, told};
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

fn at(program: &str) -> Value {
    json!({ "program": program, "node": "", "stage": "tracker" })
}

fn shown(context: &Context, program: &str) -> Value {
    call(context, "stage", &at(program)).unwrap()
}

fn id_of(stage: &Value, kind: &str) -> String {
    stage["blocks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|block| block["kind"] == json!(kind))
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned()
}

fn clarified(context: &Context, program: &str, block: &str) -> Value {
    let mut input = at(program);
    input["block"] = json!(block);
    input["question"] = json!("");
    input["chain"] = Value::Null;
    call(context, "clarify", &input).unwrap()["clarifications"].clone()
}

fn reworded() -> String {
    let mut stage: Value = serde_json::from_str(&fine()).unwrap();
    stage["blocks"][5]["text"] = json!("Треугольный канал звучит на одной громкости.");
    stage.to_string()
}

#[test]
fn уточнения_переживают_перегенерацию_а_осиротевшее_убирает_только_человек() {
    let mut answers = flat();
    answers.extend([
        "Про абзац.".to_owned(),
        "Про врезку.".to_owned(),
        reworded(),
    ]);
    let case = Case::new(true, told(answers));
    let program = begun(&case);
    let before = shown(&case.context, &program);
    let paragraph = id_of(&before, "paragraph");
    let callout = id_of(&before, "callout");
    clarified(&case.context, &program, &paragraph);
    let chains = clarified(&case.context, &program, &callout);

    regenerated(&case.context, &program, "tracker").unwrap();

    let after = shown(&case.context, &program);
    assert_eq!(id_of(&after, "paragraph"), paragraph);
    assert_ne!(id_of(&after, "callout"), callout);
    assert_eq!(after["clarifications"], chains);
    let mut input = at(&program);
    input["chain"] = json!(1);
    let removed = call(&case.context, "unclarify", &input).unwrap();
    assert_eq!(removed["clarifications"], json!([chains[0]]));
    assert_eq!(
        shown(&case.context, &program)["clarifications"],
        json!([chains[0]])
    );
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
