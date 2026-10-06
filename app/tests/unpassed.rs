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
use tolearn_core::state::State;

const FORK: &str = r#"{"next": {"why": "Голоса идут сразу за первым звуком", "recommended": false}, "alternatives": [{"id": "noise", "title": "Шумовой канал", "hours": [2, 3], "why": "Ударные без шума не собрать", "recommended": true}]}"#;

const PASSED: &str = r#"{"stage": "tracker", "per_question": [{"id": "q1", "result": "ok"}, {"id": "q2", "result": "ok"}, {"id": "q3", "result": "ok"}]}"#;

const GRADED: &str = r#"{"stage": "tracker", "per_question": [{"id": "q1", "result": "ok"}, {"id": "q2", "result": "partial", "missed": ["громкость"]}, {"id": "q3", "result": "miss", "missed": ["весь ответ"]}]}"#;

const HEAD: &str = "Незачтённые вопросы";

fn begun(answers: Vec<String>) -> (Case, String) {
    let case = Case::new(true, told(answers));
    let started = case.start(&case.plan(), LEVEL).unwrap();
    let program = started["program"].as_str().unwrap().to_owned();
    (case, program)
}

fn forking() -> Vec<String> {
    let mut answers = flat();
    answers.push(FORK.to_owned());
    answers.extend(flat().into_iter().skip(1));
    answers
}

fn at(program: &str) -> Value {
    json!({ "program": program, "node": "", "stage": "tracker" })
}

fn called(context: &Context, command: &str, program: &str) -> Result<Value, IpcError> {
    call(context, command, &at(program))
}

fn pasted(context: &Context, program: &str, text: &str) {
    let mut input = at(program);
    input["text"] = json!(text);
    call(context, "exam_paste", &input).unwrap();
}

fn forked(case: &Case, program: &str) -> String {
    called(&case.context, "fork", program).unwrap();
    case.model.heard().last().unwrap().clone()
}

#[test]
fn после_проваленного_зачёта_развилка_и_следующий_этап_получают_незачтённое() {
    let (case, program) = begun(forking());
    pasted(&case.context, &program, GRADED);
    let heard = case.model.heard().len();

    called(&case.context, "fork", &program).unwrap();
    let mut input = at(&program);
    input["choice"] = json!(1);
    call(&case.context, "take_next", &input).unwrap();

    let prompts = case.model.heard().split_off(heard);
    for part in [HEAD, "Упущено: громкость", "Упущено: весь ответ"]
    {
        assert!(prompts[0].contains(part), "{part}: {}", prompts[0]);
    }
    let text = prompts
        .iter()
        .find(|prompt| prompt.contains("Ты пишешь один этап"))
        .unwrap();
    assert!(text.contains("Упущено: громкость"), "{text}");
}

#[test]
fn без_незачтённого_промпт_развилки_тот_же_что_у_новой_программы() {
    let (fresh, first) = begun(forking());
    let (skipped, second) = begun(forking());
    let (passed, third) = begun(forking());
    called(&skipped.context, "skip", &second).unwrap();
    pasted(&passed.context, &third, PASSED);

    let plain = forked(&fresh, &first);

    assert!(!plain.contains(HEAD), "{plain}");
    assert_eq!(forked(&skipped, &second), plain);
    assert_eq!(forked(&passed, &third), plain);
}

#[test]
fn перегенерация_прячет_прежние_попытки_и_их_незачтённое() {
    let mut answers = flat();
    answers.push(flat().remove(2));
    answers.push(FORK.to_owned());
    let (case, program) = begun(answers);
    pasted(&case.context, &program, GRADED);

    called(&case.context, "regenerate_stage", &program).unwrap();

    let state = State::read(&case.data, &program).unwrap();
    assert!(state.last_attempt(&program, "tracker").is_none());
    let file = case.data.join("state").join(&program).join("state.yaml");
    let text = std::fs::read_to_string(file).unwrap();
    assert!(text.contains("весь ответ"), "{text}");
    assert!(text.contains("since: 1"), "{text}");
    let prompt = forked(&case, &program);
    assert!(!prompt.contains(HEAD), "{prompt}");
}
