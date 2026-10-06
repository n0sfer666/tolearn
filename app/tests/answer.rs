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

fn begun() -> (Case, String) {
    let case = Case::new(true, told(flat()));
    let started = case.start(&case.plan(), LEVEL).unwrap();
    let program = started["program"].as_str().unwrap().to_owned();
    (case, program)
}

fn at(program: &str) -> Value {
    json!({ "program": program, "node": "", "stage": "tracker" })
}

fn drafted(
    context: &Context,
    program: &str,
    question: &str,
    text: &str,
) -> Result<Value, IpcError> {
    let mut input = at(program);
    input["question"] = json!(question);
    input["text"] = json!(text);
    call(context, "answer", &input)
}

fn questions(context: &Context, program: &str) -> Value {
    call(context, "stage", &at(program)).unwrap()["questions"].clone()
}

#[test]
fn черновик_ответа_пишется_и_стирается() {
    let (case, program) = begun();

    let saved = drafted(&case.context, &program, "q2", "Нет громкости").unwrap();
    assert_eq!(saved, json!({ "draft": "Нет громкости" }));
    assert_eq!(
        questions(&case.context, &program)[1]["draft"],
        json!("Нет громкости")
    );
    let wiped = drafted(&case.context, &program, "q2", "  \n").unwrap();
    assert_eq!(wiped, json!({ "draft": "" }));
    assert_eq!(questions(&case.context, &program)[1]["draft"], json!(""));

    let stray = drafted(&case.context, &program, "q9", "текст").unwrap_err();
    assert_eq!(stray.code, "question.absent");
    let mut input = at(&program);
    input["stage"] = json!("nope");
    input["question"] = json!("q1");
    input["text"] = json!("текст");
    assert_eq!(
        call(&case.context, "answer", &input).unwrap_err().code,
        "stage.absent"
    );
}
