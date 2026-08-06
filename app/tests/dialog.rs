#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "app gate: a panic here is the report"
)]

mod support;

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use serde_json::{Value, json};
use support::copied;
use support::speaking::{Speaking, speaking};
use tolearn_app::ipc::{Context, IpcError, call};
use tolearn_provider::{Remembered, Vault};

static CASES: AtomicUsize = AtomicUsize::new(0);

const TOPIC: &str = "local-runtime";
const TODAY: &str = "2026-08-06";

struct Case {
    context: Context,
    data: PathBuf,
    root: PathBuf,
}

fn case(name: &str) -> Case {
    let data = std::env::temp_dir().join(format!(
        "tolearn-dialog-{name}-{}-{}",
        std::process::id(),
        CASES.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&data);
    std::fs::create_dir_all(&data).unwrap();
    let vault: Arc<dyn Vault> = Arc::new(Remembered::default());
    Case {
        context: Context::with_vault(&data, vault),
        data,
        root: copied(&format!("dialog-{name}")),
    }
}

fn enable(case: &Case, endpoint: &str) {
    call(
        &case.context,
        "provider",
        &json!({
            "save": {
                "enabled": true,
                "active": "local",
                "local": {
                    "endpoint": endpoint,
                    "api": "ollama",
                    "model": "llama3:8b",
                    "num_ctx": 0,
                    "temperature_tenths": 7,
                },
                "remote": {
                    "endpoint": endpoint,
                    "api": "openai",
                    "model": "gpt",
                    "num_ctx": 0,
                    "temperature_tenths": 7,
                },
                "harness": {
                    "id": "claude",
                    "command": "claude",
                    "args": ["-p"],
                    "timeout_secs": 180,
                },
            },
            "key": Value::Null,
            "forget": false,
            "check": false,
            "probe": false,
        }),
    )
    .unwrap();
}

fn examiner() -> Speaking {
    speaking(|prompt, _| answer(prompt))
}

fn answer(prompt: &str) -> String {
    if prompt.starts_with("# Диалоговый зачёт: приёмка практики") {
        return "Смотрю логи.\n\n```json\n{\"artifact\": \"passed\", \"failed_checks\": [], \
                \"next\": \"close\", \"say\": \"Практика принята.\"}\n```"
            .to_owned();
    }
    if prompt.starts_with("# Диалоговый зачёт: подсказка") {
        return "Посмотри на колонку PROCESSOR.".to_owned();
    }
    if prompt.starts_with("# Диалоговый зачёт: один вопрос") {
        let id = asked(prompt);
        return format!(
            "```json\n{{\"id\": \"{id}\", \"result\": \"ok\", \"quote\": \"веса плюс KV\", \
             \"missed\": [], \"signal_extension\": false, \"next\": \"close\", \
             \"say\": \"Принято.\"}}\n```"
        );
    }
    format!(
        "Разбор.\n\n```json\n{{\"topic_id\": \"{TOPIC}\", \"verdict\": \"pass\", \
         \"per_question\": [{{\"id\": \"q1\", \"result\": \"ok\"}}], \"gaps\": [], \
         \"date\": \"{TODAY}\"}}\n```"
    )
}

fn asked(prompt: &str) -> String {
    let head = "## Вопрос\n\n`";
    let at = prompt.rfind(head).expect("в промпте есть вопрос");
    prompt[at + head.len()..]
        .split('`')
        .next()
        .expect("вопрос начинается с идентификатора")
        .to_owned()
}

fn state(context: &Context, case: &Case) -> Value {
    call(
        context,
        "exam_state",
        &json!({ "bundle": case.root.display().to_string(), "topic": TOPIC }),
    )
    .unwrap()
}

fn start(case: &Case) -> Value {
    call(
        &case.context,
        "exam_start",
        &json!({ "bundle": case.root.display().to_string(), "topic": TOPIC, "restart": false }),
    )
    .unwrap()
}

fn say(case: &Case, text: &str) -> Result<Value, IpcError> {
    call(
        &case.context,
        "exam_say",
        &json!({ "bundle": case.root.display().to_string(), "topic": TOPIC, "text": text }),
    )
}

fn hint(case: &Case) -> Result<Value, IpcError> {
    call(
        &case.context,
        "exam_hint",
        &json!({ "bundle": case.root.display().to_string(), "topic": TOPIC }),
    )
}

fn finish(case: &Case) -> Result<Value, IpcError> {
    call(
        &case.context,
        "exam_finish",
        &json!({ "bundle": case.root.display().to_string(), "topic": TOPIC, "today": TODAY }),
    )
}

fn last(view: &Value) -> String {
    view["log"]
        .as_array()
        .unwrap()
        .last()
        .unwrap()
        .get("text")
        .unwrap()
        .as_str()
        .unwrap()
        .to_owned()
}

fn passed(case: &Case) {
    say(case, "Вот `predict.md` и четыре лога, числа сошлись.").unwrap();
}

#[test]
fn зачёт_начинается_с_практики_и_переходит_к_вопросам() {
    let case = case("stages");
    let heard = examiner();
    enable(&case, &heard.endpoint);

    let opened = start(&case);
    assert_eq!(opened["stage"], json!("practice"));
    assert_eq!(opened["total"], json!(4));
    assert!(last(&opened).contains("Задание"), "{opened}");

    let after = say(&case, "Вот логи и `predict.md`.").unwrap();
    assert_eq!(after["stage"], json!("question"));
    assert_eq!(last(&after), first(&case).trim());
}

fn first(case: &Case) -> String {
    let topic = call(
        &case.context,
        "topic",
        &json!({ "bundle": case.root.display().to_string(), "topic": TOPIC, "today": TODAY }),
    )
    .unwrap();
    topic["questions"][0]["text"].as_str().unwrap().to_owned()
}

#[test]
fn вопросы_идут_по_одному() {
    let case = case("one");
    let heard = examiner();
    enable(&case, &heard.endpoint);
    start(&case);
    passed(&case);

    say(&case, "SIZE считает веса, KV-кэш и compute buffers.").unwrap();

    let sent = heard.heard();
    let turns: Vec<&String> = sent
        .iter()
        .filter(|prompt| prompt.starts_with("# Диалоговый зачёт: один вопрос"))
        .collect();
    assert_eq!(turns.len(), 1, "{sent:?}");
    assert!(turns[0].contains("`q1`"), "{}", turns[0]);
    assert!(!turns[0].contains("## Вопрос\n\n`q2`"), "{}", turns[0]);
}

#[test]
fn подсказка_помечает_вопрос_hinted() {
    let case = case("hint");
    let heard = examiner();
    enable(&case, &heard.endpoint);
    start(&case);
    passed(&case);

    let helped = hint(&case).unwrap();
    assert_eq!(helped["hinted"], json!(["q1"]));
    assert_eq!(last(&helped), "Посмотри на колонку PROCESSOR.");

    say(&case, "Часть слоёв уехала на CPU.").unwrap();
    let finished = finish(&case).unwrap();
    let sent = heard.heard();
    let verdict = sent
        .iter()
        .rfind(|prompt| prompt.contains("## Диалог уже проведён"))
        .expect("вердикт запрошен");
    assert!(verdict.contains("`q1` — `ok`; подсказка: да"), "{verdict}");
    assert_eq!(finished["stage"], json!("done"));
}

#[test]
fn прерванный_диалог_продолжается_после_перезапуска() {
    let case = case("resume");
    let heard = examiner();
    enable(&case, &heard.endpoint);
    start(&case);
    passed(&case);
    let before = say(&case, "Веса, KV-кэш и compute buffers.").unwrap();

    let vault: Arc<dyn Vault> = Arc::new(Remembered::default());
    let again = Context::with_vault(&case.data, vault);
    let after = state(&again, &case);

    assert_eq!(after["open"], json!(true));
    assert_eq!(after["asked"], before["asked"]);
    assert_eq!(after["log"], before["log"]);
    assert_eq!(after["stage"], json!("question"));
}

#[test]
fn завершение_даёт_вердикт_того_же_формата() {
    let case = case("verdict");
    let heard = examiner();
    enable(&case, &heard.endpoint);
    start(&case);
    passed(&case);
    for _ in 0..4 {
        say(&case, "Веса, KV-кэш и compute buffers.").unwrap();
    }

    let finished = finish(&case).unwrap();
    let text = finished["verdict"].as_str().unwrap();
    let parsed = call(
        &case.context,
        "parse_verdict",
        &json!({
            "bundle": case.root.display().to_string(),
            "topic": TOPIC,
            "text": text,
        }),
    )
    .unwrap();

    assert_eq!(parsed["result"], json!("pass"));
    assert_eq!(finished["stage"], json!("done"));
    assert!(say(&case, "ещё").is_err());
}

#[test]
fn зачёт_нельзя_вести_не_начав() {
    let case = case("cold");
    let heard = examiner();
    enable(&case, &heard.endpoint);

    let cold = state(&case.context, &case);
    assert_eq!(cold["open"], json!(false));
    assert_eq!(say(&case, "привет").unwrap_err().code, "exam.not-started");
}
