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

const TODAY: &str = "2026-07-28";
const FIRST: &str = "tokens-context-cost";
const SECOND: &str = "local-runtime";

struct Case {
    context: Context,
    data: PathBuf,
    root: PathBuf,
}

fn case(name: &str) -> Case {
    let data = std::env::temp_dir().join(format!(
        "tolearn-sweep-{name}-{}-{}",
        std::process::id(),
        CASES.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&data);
    std::fs::create_dir_all(&data).unwrap();
    let root = copied(&format!("sweep-{name}"));
    let queue = support::repository().join("fixtures/valid/progress/review-queue.yaml");
    std::fs::copy(queue, root.join("progress.yaml")).unwrap();
    let vault: Arc<dyn Vault> = Arc::new(Remembered::default());
    Case {
        context: Context::with_vault(&data, vault),
        data,
        root,
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
                "journal": false,
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
    let topic = about(prompt);
    format!(
        "Разбор.\n\n```json\n{{\"topic_id\": \"{topic}\", \"verdict\": \"pass\", \
         \"per_question\": [{{\"id\": \"q1\", \"result\": \"ok\"}}], \"gaps\": [], \
         \"date\": \"{TODAY}\"}}\n```"
    )
}

fn asked(prompt: &str) -> String {
    cut(prompt, "## Вопрос\n\n`", '`')
}

fn about(prompt: &str) -> String {
    cut(prompt, "- `topic_id`: ", '\n')
}

fn cut(prompt: &str, head: &str, till: char) -> String {
    let at = prompt.rfind(head).expect("в промпте есть метка");
    prompt[at + head.len()..]
        .split(till)
        .next()
        .expect("метка не пустая")
        .trim()
        .to_owned()
}

fn state(context: &Context, case: &Case) -> Value {
    call(
        context,
        "sweep_state",
        &json!({ "bundle": case.root.display().to_string(), "today": TODAY }),
    )
    .unwrap()
}

fn start(case: &Case, topics: u32) -> Value {
    call(
        &case.context,
        "sweep_start",
        &json!({
            "bundle": case.root.display().to_string(),
            "today": TODAY,
            "topics": topics,
            "restart": false,
        }),
    )
    .unwrap()
}

fn say(case: &Case, text: &str) -> Result<Value, IpcError> {
    call(
        &case.context,
        "sweep_say",
        &json!({ "bundle": case.root.display().to_string(), "today": TODAY, "text": text }),
    )
}

fn plain(case: &Case, command: &str) -> Result<Value, IpcError> {
    call(
        &case.context,
        command,
        &json!({ "bundle": case.root.display().to_string(), "today": TODAY }),
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

fn progress(case: &Case) -> String {
    std::fs::read_to_string(case.root.join("progress.yaml")).unwrap()
}

#[test]
fn прогон_берёт_подошедшие_темы_и_чередует_их() {
    let case = case("rotate");
    let heard = examiner();
    enable(&case, &heard.endpoint);

    let opened = start(&case, 3);
    assert_eq!(opened["stage"], json!("question"));
    assert_eq!(opened["legs"].as_array().unwrap().len(), 3);
    assert_eq!(opened["legs"][0]["topic"], json!(FIRST));
    assert!(last(&opened).starts_with("Тема «"), "{opened}");

    let after = say(&case, "Контекст тоже оплачивается.").unwrap();
    let title = after["legs"][1]["title"].as_str().unwrap();
    assert!(last(&after).contains(title), "{}", last(&after));
    assert_eq!(after["legs"][0]["asked"], json!(1));
    assert_eq!(after["legs"][1]["asked"], json!(0));
}

#[test]
fn пул_прогона_не_берёт_непройденное() {
    let case = case("pool");
    let heard = examiner();
    enable(&case, &heard.endpoint);

    let cold = state(&case.context, &case);
    let ready: Vec<&str> = cold["ready"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["topic"].as_str().unwrap())
        .collect();

    assert_eq!(cold["open"], json!(false));
    assert_eq!(ready.first(), Some(&FIRST));
    assert!(!ready.contains(&"model-selection"), "{ready:?}");
    assert_eq!(cold["ready"][0]["overdue"], json!(true));
}

#[test]
fn подсказка_живёт_в_теме_текущего_вопроса() {
    let case = case("hint");
    let heard = examiner();
    enable(&case, &heard.endpoint);
    start(&case, 2);

    let helped = plain(&case, "sweep_hint").unwrap();
    assert_eq!(last(&helped), "Посмотри на колонку PROCESSOR.");

    say(&case, "Контекст тоже оплачивается.").unwrap();
    say(&case, "SIZE считает веса и KV-кэш.").unwrap();
    plain(&case, "sweep_finish").unwrap();

    let sent = heard.heard();
    let verdict = sent
        .iter()
        .find(|prompt| {
            prompt.contains("## Диалог уже проведён")
                && prompt.contains(&format!("- `topic_id`: {FIRST}"))
        })
        .expect("вердикт по первой теме запрошен");
    assert!(verdict.contains("подсказка: да"), "{verdict}");
}

#[test]
fn вердикт_идёт_отдельным_запросом_на_каждую_тронутую_тему() {
    let case = case("verdicts");
    let heard = examiner();
    enable(&case, &heard.endpoint);
    start(&case, 3);
    say(&case, "Контекст тоже оплачивается.").unwrap();
    say(&case, "SIZE считает веса и KV-кэш.").unwrap();

    let done = plain(&case, "sweep_finish").unwrap();
    let sent = heard.heard();
    let verdicts: Vec<&String> = sent
        .iter()
        .filter(|prompt| prompt.contains("## Диалог уже проведён"))
        .collect();

    assert_eq!(verdicts.len(), 2, "{verdicts:?}");
    assert!(verdicts.iter().all(|prompt| {
        prompt.contains("практика в этом прогоне не проверялась и на вердикт не влияет")
    }));
    assert_eq!(done["stage"], json!("done"));
    assert!(done["legs"][0]["verdict"].is_string());
    assert!(done["legs"][2]["verdict"].is_null());
}

#[test]
fn прерванный_прогон_продолжается_после_перезапуска() {
    let case = case("resume");
    let heard = examiner();
    enable(&case, &heard.endpoint);
    start(&case, 3);
    let before = say(&case, "Контекст тоже оплачивается.").unwrap();

    let vault: Arc<dyn Vault> = Arc::new(Remembered::default());
    let again = Context::with_vault(&case.data, vault);
    let after = state(&again, &case);

    assert_eq!(after["open"], json!(true));
    assert_eq!(after["asked"], before["asked"]);
    assert_eq!(after["log"], before["log"]);
    assert_eq!(after["stage"], json!("question"));
}

#[test]
fn прогон_пишет_в_бандл_только_после_приёмки() {
    let case = case("accept");
    let heard = examiner();
    enable(&case, &heard.endpoint);
    start(&case, 2);
    say(&case, "Контекст тоже оплачивается.").unwrap();
    say(&case, "SIZE считает веса и KV-кэш.").unwrap();
    let before = progress(&case);

    plain(&case, "sweep_finish").unwrap();
    assert_eq!(progress(&case), before, "завершение тронуло progress.yaml");

    let accepted = plain(&case, "sweep_accept").unwrap();
    let settled = accepted["settled"].as_array().unwrap();
    assert_eq!(settled.len(), 2);
    assert_eq!(settled[0]["topic"], json!(FIRST));
    assert_eq!(settled[0]["result"], json!("pass"));
    assert_eq!(settled[0]["status"], json!("passed"));
    assert_eq!(settled[1]["topic"], json!(SECOND));
    assert_ne!(progress(&case), before);
    assert_eq!(
        state(&case.context, &case)["open"],
        json!(false),
        "прогон не забыт после приёмки"
    );
}

#[test]
fn приёмка_без_вердиктов_ничего_не_пишет() {
    let case = case("early");
    let heard = examiner();
    enable(&case, &heard.endpoint);
    start(&case, 2);
    let before = progress(&case);

    let failed = plain(&case, "sweep_accept").unwrap_err();

    assert_eq!(failed.code, "sweep.unfinished");
    assert_eq!(progress(&case), before);
}

#[test]
fn прогон_нельзя_вести_не_начав() {
    let case = case("cold");
    let heard = examiner();
    enable(&case, &heard.endpoint);

    assert_eq!(say(&case, "привет").unwrap_err().code, "sweep.not-started");
}
