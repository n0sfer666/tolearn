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
use tolearn_core::state::{Grade, Sitting, State};
use tolearn_generate::ledger;

const PASSED: &str = r#"{"stage": "tracker", "per_question": [{"id": "q1", "result": "ok"}, {"id": "q2", "result": "ok"}, {"id": "q3", "result": "ok"}]}"#;

const GRADED: &str = r#"Вот разбор ответов.

```json
{"stage": "tracker", "per_question": [
  {"id": "q1", "result": "ok"},
  {"id": "q2", "result": "partial", "missed": ["нет регулировки громкости"]},
  {"id": "q3", "result": "miss", "missed": ["весь ответ"]}
]}
```"#;

const FOREIGN: &str = r#"{"stage": "voices", "per_question": [{"id": "q1", "result": "ok"}]}"#;

const ANSWERS: [(&str, &str); 3] = [("q1", "Пять каналов"), ("q2", "Громкостью"), ("q3", "")];

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

fn answered(program: &str, answers: &[(&str, &str)]) -> Value {
    let mut input = at(program);
    input["answers"] = answers
        .iter()
        .map(|&(id, text)| json!({ "id": id, "text": text }))
        .collect();
    input
}

fn prompted(context: &Context, program: &str, answers: &[(&str, &str)]) -> Result<Value, IpcError> {
    call(context, "exam_prompt", &answered(program, answers))
}

fn pasted(context: &Context, program: &str, text: &str) -> Result<Value, IpcError> {
    let mut input = at(program);
    input["text"] = json!(text);
    call(context, "exam_paste", &input)
}

fn unplugged(case: &Case) -> Context {
    let mut saved = provider(&case.model.endpoint);
    saved["enabled"] = json!(false);
    let input = json!({ "save": saved, "key": Value::Null, "forget": false, "check": false, "probe": false });
    call(&case.context, "provider", &input).unwrap();
    case.context.clone().with_reach(Arc::new(Net(false)))
}

fn logged(case: &Case, program: &str) -> usize {
    ledger::read(&ledger::path(&case.data, program))
        .unwrap()
        .len()
}

fn state(case: &Case, program: &str) -> PathBuf {
    case.data.join("state").join(program).join("state.yaml")
}

#[test]
fn промпт_копипаста_тот_же_что_уходит_модели_на_зачёте() {
    let (case, program) = begun(&[PASSED]);

    let copied = prompted(&case.context, &program, &ANSWERS).unwrap();
    call(&case.context, "exam", &answered(&program, &ANSWERS)).unwrap();

    let prompt = copied["prompt"].as_str().unwrap();
    assert_eq!(case.model.heard().last().map(String::as_str), Some(prompt));
    assert!(prompt.contains("Громкостью"));
    assert!(prompt.contains("нет ответа"));
    assert!(prompt.contains(LEVEL));
    assert!(prompt.contains("закончи ответ блоком JSON"));
}

#[test]
fn копипаст_работает_без_провайдера_и_сети() {
    let (case, program) = begun(&[]);
    let offline = unplugged(&case);
    assert!(call(&offline, "exam", &answered(&program, &ANSWERS)).is_err());
    let heard = case.model.heard().len();
    let records = logged(&case, &program);

    let copied = prompted(&offline, &program, &ANSWERS).unwrap();
    assert!(copied["prompt"].as_str().unwrap().contains("Пять каналов"));
    let result = pasted(&offline, &program, GRADED).unwrap();

    assert_eq!(result, json!({ "passed": false }));
    assert_eq!(case.model.heard().len(), heard);
    assert_eq!(logged(&case, &program), records);
    let read = State::read(&case.data, &program).unwrap();
    let attempt = read.last_attempt(&program, "tracker").unwrap();
    assert_eq!(attempt.by, Sitting::Copypaste);
    assert_eq!(attempt.model, None);
    let grades: Vec<Grade> = attempt.per_question.iter().map(|row| row.result).collect();
    assert_eq!(grades, [Grade::Ok, Grade::Partial, Grade::Miss]);
    let shown = call(&offline, "stage", &at(&program)).unwrap()["questions"].clone();
    assert_eq!(shown[2]["result"], json!("miss"));
    assert_eq!(shown[2]["missed"], json!(["весь ответ"]));
}

#[test]
fn неразобранная_вставка_называет_причину_и_не_трогает_состояние() {
    let (case, program) = begun(&[]);
    let mut draft = at(&program);
    draft["question"] = json!("q1");
    draft["text"] = json!("Пять");
    call(&case.context, "answer", &draft).unwrap();
    let before = std::fs::read(state(&case, &program)).unwrap();

    let absent = pasted(&case.context, &program, "Всё отлично, так держать!").unwrap_err();
    assert_eq!(absent.code, "exam.verdict");
    assert!(
        absent.message.contains("нет JSON-блока"),
        "{}",
        absent.message
    );
    let foreign = pasted(&case.context, &program, FOREIGN).unwrap_err();
    assert_eq!(foreign.code, "exam.verdict");
    assert!(
        foreign.message.contains("не в тот этап"),
        "{}",
        foreign.message
    );
    let empty = prompted(&case.context, &program, &[("q1", "  \n"), ("q2", "")]).unwrap_err();
    assert_eq!(empty.code, "exam.empty");
    let stray = prompted(&case.context, &program, &[("q9", "Пять")]).unwrap_err();
    assert_eq!(stray.code, "question.absent");

    assert_eq!(std::fs::read(state(&case, &program)).unwrap(), before);
}
