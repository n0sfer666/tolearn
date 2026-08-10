#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "app gate: a panic here is the report"
)]

mod support;

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use support::copied;
use tolearn_app::ipc::{Context, call};
use tolearn_core::scan::scan;
use tolearn_offline::store::{Fetched, Store};

const URL: &str = "https://docs.ollama.com/faq";
const PAGE: &str = "<html><head><title>Ollama FAQ</title></head><body><article>\
<h1>Ollama FAQ</h1>\
<p>Модель держится в памяти пять минут после запроса, потом выгружается сама. \
Держать её дольше стоит через параметр keep_alive: он принимает и длительность, и ноль, \
который выгружает модель сразу, и минус единицу, которая держит её в памяти постоянно.</p>\
<p>Размер окна задаётся параметром num_ctx и упирается в то, сколько памяти готова отдать \
видеокарта: окно на сто двадцать восемь тысяч токенов требует заметно больше, чем окно \
по умолчанию, и на слабой машине запрос просто не поместится.</p>\
<p>Слушать сеть Ollama по умолчанию не станет: адрес задаётся переменной OLLAMA_HOST, \
и без неё сервер отвечает только с самой машины, что для офлайн-работы ровно то, что нужно.</p>\
</article></body></html>";

fn context(name: &str) -> (Context, PathBuf) {
    let data = std::env::temp_dir().join(format!("tolearn-offline-{name}-{}", std::process::id()));
    if data.exists() {
        std::fs::remove_dir_all(&data).unwrap();
    }
    std::fs::create_dir_all(&data).unwrap();
    (Context::new(&data), data.join("offline"))
}

fn saved(root: &Path, program: &str) {
    let mut store = Store::open(root, 64 * 1024 * 1024).unwrap();
    store
        .put(
            URL,
            program,
            &Fetched {
                kind: "archive",
                bytes: PAGE.as_bytes(),
                etag: None,
                last_modified: None,
            },
            1_700_000_000,
        )
        .unwrap();
}

fn material(topic: &Value, url: &str) -> Value {
    topic["materials"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["url"] == url)
        .unwrap_or_else(|| panic!("материала {url} нет в теме: {topic:#}"))
        .clone()
}

fn topic(context: &Context, bundle: &Path) -> Value {
    call(
        context,
        "topic",
        &json!({
            "bundle": bundle.display().to_string(),
            "topic": "local-runtime",
            "today": "2026-07-27",
        }),
    )
    .unwrap()
}

fn awaited(context: &Context, job: &str) -> Value {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let state = call(context, "offline_state", &json!({ "job": job })).unwrap();
        if state["finished"] == json!(true) {
            return state;
        }
        assert!(
            Instant::now() < deadline,
            "выгрузка не закончилась: {state:#}"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}

#[test]
fn сохранённый_материал_виден_в_теме_и_открывается_без_сети() {
    let bundle = copied("offline-seen");
    let (context, root) = context("seen");
    saved(&root, "llm-agents-base");

    let document = topic(&context, &bundle);
    assert_eq!(material(&document, URL)["offline"], "saved");

    let read = call(&context, "read_offline", &json!({ "url": URL })).unwrap();
    assert_eq!(read["kind"], "archive");
    assert_eq!(read["title"], "Ollama FAQ");
    assert!(
        read["text"].as_str().unwrap().contains("пять минут"),
        "текст статьи не дочитался: {read:#}"
    );
}

#[test]
fn несохранённый_материал_помечен_absent_и_не_открывается() {
    let bundle = copied("offline-absent");
    let (context, _) = context("absent");

    let document = topic(&context, &bundle);
    assert_eq!(material(&document, URL)["offline"], "absent");

    let error = call(&context, "read_offline", &json!({ "url": URL })).unwrap_err();
    assert_eq!(error.code, "offline.absent");
}

#[test]
fn выгрузка_заводится_отчитывается_и_останавливается() {
    let bundle = copied("offline-job");
    let (context, _) = context("job");

    let started = call(
        &context,
        "save_offline",
        &json!({
            "bundle": bundle.display().to_string(),
            "topic": "local-runtime",
            "again": "job-которого-не-было",
        }),
    )
    .unwrap();
    let job = started["job"].as_str().unwrap().to_owned();
    assert_eq!(started["total"], 0, "повтор без павших берёт пустой набор");

    let state = awaited(&context, &job);
    assert_eq!(state["done"], 0);
    assert_eq!(state["cancelled"], json!(false));
    assert!(state["failed"].as_array().unwrap().is_empty());

    let stopped = call(&context, "stop_offline", &json!({ "job": job })).unwrap();
    assert_eq!(stopped["stopping"], json!(true));
}

#[test]
fn материал_из_двух_тем_едет_в_очередь_один_раз() {
    let bundle = copied("offline-every");
    let scan = scan(&bundle).unwrap();
    let pieces = tolearn_app::offline::every(&scan);

    let all: usize = scan.topics.iter().map(|topic| topic.materials.len()).sum();
    let mut urls: Vec<&str> = pieces
        .iter()
        .map(|piece| piece.material.url.as_str())
        .collect();
    urls.sort_unstable();
    urls.dedup();

    assert!(all > pieces.len(), "в программе нет общих материалов");
    assert_eq!(urls.len(), pieces.len(), "материал уехал в очередь дважды");
    assert!(
        pieces.iter().all(|piece| !piece.topic.is_empty()),
        "материал поехал без темы-владельца"
    );
}

#[test]
fn цена_всей_программы_называется_до_старта() {
    let bundle = copied("offline-cost");
    let (context, root) = context("cost");
    saved(&root, "llm-agents-base");

    let cost = call(
        &context,
        "offline_cost",
        &json!({ "bundle": bundle.display().to_string() }),
    )
    .unwrap();

    let scan = scan(&bundle).unwrap();
    let materials = tolearn_app::offline::every(&scan).len();
    assert_eq!(cost["materials"], json!(materials));
    assert_eq!(cost["held"], json!(1), "уже сохранённое не сосчиталось");
    assert!(cost["used"].as_u64().unwrap() > 0);
    assert!(cost["need"].as_u64().unwrap() > 0, "цена вышла нулевой");
    assert_eq!(cost["tight"], json!(false), "бюджет по умолчанию не жмёт");
}

#[test]
fn чужая_выгрузка_не_находится() {
    let (context, _) = context("unknown");

    let error = call(&context, "offline_state", &json!({ "job": "job-0" })).unwrap_err();
    assert_eq!(error.code, "offline.unknown-job");

    let stopped = call(&context, "stop_offline", &json!({ "job": "job-0" })).unwrap();
    assert_eq!(stopped["stopping"], json!(false));
}

#[test]
fn неизвестная_тема_не_заводит_выгрузку() {
    let bundle = copied("offline-topic");
    let (context, _) = context("topic");

    let error = call(
        &context,
        "save_offline",
        &json!({ "bundle": bundle.display().to_string(), "topic": "нет-такой" }),
    )
    .unwrap_err();
    assert_eq!(error.code, "topic.unknown");
}
