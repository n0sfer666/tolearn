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
use tolearn_core::state::{Pass, State, Status};

const PASSED: &str = r#"{"stage": "tracker", "per_question": [{"id": "q1", "result": "ok"}, {"id": "q2", "result": "ok"}, {"id": "q3", "result": "ok"}]}"#;

const GRADED: &str = r#"{"stage": "tracker", "per_question": [{"id": "q1", "result": "ok"}, {"id": "q2", "result": "partial", "missed": ["громкость"]}, {"id": "q3", "result": "miss", "missed": ["весь ответ"]}]}"#;

fn begun() -> (Case, String) {
    let case = Case::new(true, told(flat()));
    let started = case.start(&case.plan(), LEVEL).unwrap();
    let program = started["program"].as_str().unwrap().to_owned();
    (case, program)
}

fn at(program: &str, stage: &str) -> Value {
    json!({ "program": program, "node": "", "stage": stage })
}

fn skipped(context: &Context, program: &str, stage: &str) -> Result<Value, IpcError> {
    call(context, "skip", &at(program, stage))
}

fn pasted(context: &Context, program: &str, text: &str) -> Value {
    let mut input = at(program, "tracker");
    input["text"] = json!(text);
    call(context, "exam_paste", &input).unwrap()
}

fn status(case: &Case, program: &str) -> Status {
    State::read(&case.data, program)
        .unwrap()
        .status(program, "tracker")
}

fn state(case: &Case, program: &str) -> Option<Vec<u8>> {
    std::fs::read(case.data.join("state").join(program).join("state.yaml")).ok()
}

#[test]
fn пропуск_отмечает_этап_пройденным_без_попытки_и_без_сети() {
    let (case, program) = begun();
    let away = case.context.clone().with_reach(Arc::new(Net(false)));
    let heard = case.model.heard().len();

    let first = skipped(&away, &program, "tracker").unwrap();
    let again = skipped(&away, &program, "tracker").unwrap();

    assert_eq!(first, json!({ "skipped": true }));
    assert_eq!(again, json!({ "skipped": false }));
    assert_eq!(status(&case, &program), Status::Passed(Pass::Skip));
    let read = State::read(&case.data, &program).unwrap();
    assert!(read.last_attempt(&program, "tracker").is_none());
    assert_eq!(case.model.heard().len(), heard);
    let node = call(
        &case.context,
        "node",
        &json!({ "program": program, "node": "" }),
    )
    .unwrap();
    assert_eq!(node["summary"]["passed"], json!(1));
    assert_eq!(node["summary"]["skipped"], json!(1));
    assert_eq!(node["stages"][0]["status"], json!("passed"));
    assert_eq!(node["stages"][0]["pass"], json!("skip"));
}

#[test]
fn после_пропуска_проваленный_зачёт_не_снимает_пройден_а_сданный_меняет_пометку() {
    let (case, program) = begun();
    skipped(&case.context, &program, "tracker").unwrap();

    assert_eq!(
        pasted(&case.context, &program, GRADED),
        json!({ "passed": false })
    );
    assert_eq!(status(&case, &program), Status::Passed(Pass::Skip));
    let read = State::read(&case.data, &program).unwrap();
    assert!(read.last_attempt(&program, "tracker").is_some());

    assert_eq!(
        pasted(&case.context, &program, PASSED),
        json!({ "passed": true })
    );
    assert_eq!(status(&case, &program), Status::Passed(Pass::Exam));
    let node = call(
        &case.context,
        "node",
        &json!({ "program": program, "node": "" }),
    )
    .unwrap();
    assert_eq!(node["summary"]["skipped"], json!(0));
}

#[test]
fn пропуск_после_сданного_зачёта_ничего_не_меняет() {
    let (case, program) = begun();
    pasted(&case.context, &program, PASSED);
    let before = state(&case, &program);

    assert_eq!(
        skipped(&case.context, &program, "tracker").unwrap(),
        json!({ "skipped": false })
    );

    assert_eq!(status(&case, &program), Status::Passed(Pass::Exam));
    assert_eq!(state(&case, &program), before);
}

#[test]
fn пропуск_несуществующего_этапа_отказывает_и_не_пишет_состояние() {
    let (case, program) = begun();
    let before = state(&case, &program);

    let refused = skipped(&case.context, &program, "nope").unwrap_err();

    assert_eq!(refused.code, "stage.absent");
    assert_eq!(state(&case, &program), before);
}
