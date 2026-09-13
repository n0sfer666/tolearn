#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "app gate: a panic here is the report"
)]

mod support;

use std::path::PathBuf;
use std::sync::Arc;

use serde_json::{Value, json};
use support::planner::{Net, provider};
use support::starter::{Case, LEVEL, flat, told};
use tolearn_app::ipc::{Context, IpcError, call};
use tolearn_core::state::{Attempt, Grade, Sitting, State};
use tolearn_generate::ledger;

const PASSED: &str = r#"{"stage": "tracker", "per_question": [{"id": "q1", "result": "ok"}, {"id": "q2", "result": "ok"}, {"id": "q3", "result": "ok"}]}"#;

const GRADED: &str = r#"Разбор по вопросам.

```json
{"stage": "tracker", "per_question": [
  {"id": "q1", "result": "ok"},
  {"id": "q2", "result": "partial", "missed": ["нет регулировки громкости"]},
  {"id": "q3", "result": "miss", "missed": ["весь ответ"]}
]}
```"#;

const PRAISE: &str = "Всё отлично, так держать!";

fn begun(extra: &[&str]) -> (Case, String) {
    let mut answers = flat();
    answers.extend(extra.iter().map(|&text| text.to_owned()));
    let case = Case::new(true, told(answers));
    let started = case.start(&case.plan(), LEVEL).unwrap();
    let program = started["program"].as_str().unwrap().to_owned();
    (case, program)
}

fn at(program: &str) -> Value {
    json!({ "program": program, "node": "", "stage": "tracker" })
}

fn sat(context: &Context, program: &str, answers: &[(&str, &str)]) -> Result<Value, IpcError> {
    let mut input = at(program);
    input["answers"] = answers
        .iter()
        .map(|&(id, text)| json!({ "id": id, "text": text }))
        .collect();
    call(context, "exam", &input)
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

fn last(case: &Case, program: &str) -> Option<Attempt> {
    State::read(&case.data, program)
        .unwrap()
        .last_attempt(program, "tracker")
        .cloned()
}

fn state(case: &Case, program: &str) -> PathBuf {
    case.data.join("state").join(program).join("state.yaml")
}

fn active(context: &Context, case: &Case, kind: &str) {
    let mut saved = provider(&case.model.endpoint);
    saved["active"] = json!(kind);
    let input = json!({ "save": saved, "key": Value::Null, "forget": false, "check": false, "probe": false });
    call(context, "provider", &input).unwrap();
}

#[test]
fn письменный_зачёт_уходит_одним_запросом_и_становится_попыткой() {
    let (case, program) = begun(&[GRADED]);
    drafted(&case.context, &program, "q1", "Пять каналов").unwrap();
    let heard = case.model.heard().len();

    let result = sat(
        &case.context,
        &program,
        &[("q1", "Пять каналов"), ("q2", "Громкостью"), ("q3", "")],
    )
    .unwrap();

    assert_eq!(result, json!({ "passed": false }));
    let prompts = case.model.heard();
    assert_eq!(prompts.len(), heard + 1);
    let prompt = &prompts[heard];
    assert!(prompt.contains("Громкостью"));
    assert!(prompt.contains("нет ответа"));
    assert!(prompt.contains(LEVEL));
    let steps = case.steps();
    assert!(steps.contains(&("began".to_owned(), "exam".to_owned())));
    assert!(steps.contains(&("ended".to_owned(), "exam".to_owned())));
    let attempt = last(&case, &program).unwrap();
    assert_eq!(attempt.by, Sitting::Written);
    assert_eq!(attempt.model.as_deref(), Some("llama3:8b"));
    let grades: Vec<Grade> = attempt.per_question.iter().map(|row| row.result).collect();
    assert_eq!(grades, [Grade::Ok, Grade::Partial, Grade::Miss]);
    let records = ledger::read(&ledger::path(&case.data, &program)).unwrap();
    let exam: Vec<_> = records
        .iter()
        .filter(|record| record.step == "exam")
        .collect();
    assert_eq!(exam.len(), 1);
    assert_eq!(exam[0].stage.as_deref(), Some("tracker"));
    assert_eq!(exam[0].program.as_deref(), Some(program.as_str()));

    let shown = questions(&case.context, &program);
    assert_eq!(shown[0]["draft"], json!("Пять каналов"));
    assert_eq!(shown[1]["result"], json!("partial"));
    assert_eq!(shown[1]["missed"], json!(["нет регулировки громкости"]));
}

#[test]
fn ответ_без_вердикта_чинится_раз_и_отказ_не_трогает_состояние() {
    let (case, program) = begun(&[PRAISE]);
    drafted(&case.context, &program, "q1", "Пять каналов").unwrap();
    let before = std::fs::read(state(&case, &program)).unwrap();
    let heard = case.model.heard().len();

    let refused = sat(&case.context, &program, &[("q1", "Пять каналов")]).unwrap_err();

    assert_eq!(refused.code, "generate.verdict");
    assert!(
        refused.message.contains("нет JSON-блока"),
        "{}",
        refused.message
    );
    assert_eq!(case.model.heard().len(), heard + 2);
    assert_eq!(std::fs::read(state(&case, &program)).unwrap(), before);
    let records = ledger::read(&ledger::path(&case.data, &program)).unwrap();
    let rounds: Vec<Option<usize>> = records
        .iter()
        .filter(|record| record.step == "exam")
        .map(|record| record.round)
        .collect();
    assert_eq!(rounds, [None, Some(1)]);
}

#[test]
fn сеть_проверяется_только_у_удалённого_провайдера() {
    let (case, program) = begun(&[PASSED]);
    let offline = case.context.clone().with_reach(Arc::new(Net(false)));
    let heard = case.model.heard().len();

    active(&offline, &case, "remote");
    let refused = sat(&offline, &program, &[("q1", "Пять")]).unwrap_err();
    assert_eq!(refused.code, "generate.offline");
    assert_eq!(case.model.heard().len(), heard);
    assert!(last(&case, &program).is_none());

    active(&offline, &case, "local");
    let result = sat(&offline, &program, &[("q1", "Пять")]).unwrap();
    assert_eq!(result, json!({ "passed": true }));
    assert!(last(&case, &program).unwrap().passes());
}

#[test]
fn пустой_зачёт_и_чужой_вопрос_не_зовут_модель() {
    let (case, program) = begun(&[PASSED]);
    let heard = case.model.heard().len();

    let empty = sat(&case.context, &program, &[("q1", "  \n"), ("q2", "")]).unwrap_err();
    assert_eq!(empty.code, "exam.empty");
    let stray = sat(&case.context, &program, &[("q9", "Пять")]).unwrap_err();
    assert_eq!(stray.code, "question.absent");
    assert_eq!(case.model.heard().len(), heard);
    assert!(last(&case, &program).is_none());
}
