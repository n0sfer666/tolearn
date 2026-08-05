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
use support::stub;
use tolearn_app::ipc::{Context, IpcError, call};
use tolearn_provider::{Remembered, Vault};

static CASES: AtomicUsize = AtomicUsize::new(0);

const OLLAMA: &str = r#"{"models":[{"name":"llama3:8b"}]}"#;

struct Case {
    context: Context,
    data: PathBuf,
    vault: Arc<Remembered>,
}

fn case(name: &str) -> Case {
    let data = std::env::temp_dir().join(format!(
        "tolearn-provider-ipc-{name}-{}-{}",
        std::process::id(),
        CASES.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&data);
    std::fs::create_dir_all(&data).unwrap();
    let vault = Arc::new(Remembered::default());
    Case {
        context: Context::with_vault(&data, Arc::clone(&vault) as Arc<dyn Vault>),
        data,
        vault,
    }
}

fn ask(case: &Case, payload: Value) -> Result<Value, IpcError> {
    call(&case.context, "provider", &payload)
}

fn read(case: &Case) -> Value {
    ask(
        case,
        json!({
            "save": Value::Null,
            "key": Value::Null,
            "forget": false,
            "check": false,
            "probe": false,
        }),
    )
    .unwrap()
}

fn save(case: &Case, provider: Value, key: Value, check: bool) -> Result<Value, IpcError> {
    ask(
        case,
        json!({
            "save": provider,
            "key": key,
            "forget": false,
            "check": check,
            "probe": false,
        }),
    )
}

fn settings(endpoint: &str, active: &str) -> Value {
    let http = |api| {
        json!({
            "endpoint": endpoint,
            "api": api,
            "model": "llama3:8b",
            "num_ctx": 0,
            "temperature_tenths": 7,
        })
    };
    json!({
        "enabled": true,
        "active": active,
        "local": http("ollama"),
        "remote": http("openai"),
        "harness": harness("claude"),
    })
}

fn harness(command: &str) -> Value {
    json!({
        "id": "custom",
        "command": command,
        "args": ["-p"],
        "timeout_secs": 180,
    })
}

#[test]
fn провайдер_выключен_пока_его_не_включили() {
    let case = case("default");

    let answer = read(&case);

    assert_eq!(answer["provider"]["enabled"], json!(false));
    assert_eq!(answer["provider"]["active"], json!("local"));
    assert_eq!(answer["has_key"], json!(false));
    assert_eq!(answer["checked"], Value::Null);
    assert_eq!(answer["probed"], Value::Null);
}

#[test]
fn реестр_харнессов_приходит_вместе_с_настройками() {
    let case = case("presets");

    let answer = read(&case);

    let presets = answer["presets"].as_array().unwrap().clone();
    let ids: Vec<&str> = presets
        .iter()
        .filter_map(|preset| preset["id"].as_str())
        .collect();
    assert_eq!(ids, ["claude", "opencode", "pi", "custom"]);
}

#[test]
fn совет_по_моделям_приходит_обоими_именами() {
    let case = case("advised");

    let answer = read(&case);

    let advised = answer["advised"].as_array().unwrap().clone();
    assert!(!advised.is_empty());
    let first = advised.first().unwrap();
    assert!(first["gigabytes"].as_u64().unwrap() > 0);
    assert!(!first["id"].as_str().unwrap().contains('/'));
    assert!(first["repo"].as_str().unwrap().contains('/'));
    assert_eq!(first["installed"], json!(false));
}

#[test]
fn установленная_модель_помечена_в_совете() {
    let case = case("advised-installed");
    let heard = stub("200 OK", r#"{"models":[{"name":"qwen3:8b"}]}"#);

    let mut asked = settings(&heard.endpoint, "local");
    asked["local"]["model"] = json!("qwen3:8b");
    let answer = save(&case, asked, Value::Null, true).unwrap();

    let advised = answer["advised"].as_array().unwrap().clone();
    let marked: Vec<&str> = advised
        .iter()
        .filter(|advice| advice["installed"] == json!(true))
        .filter_map(|advice| advice["id"].as_str())
        .collect();
    assert_eq!(marked, ["qwen3:8b"]);
}

#[test]
fn настройки_переживают_перезапуск() {
    let case = case("stored");

    save(
        &case,
        settings("http://127.0.0.1:11434", "local"),
        Value::Null,
        false,
    )
    .unwrap();
    let answer = read(&case);

    assert_eq!(answer["provider"]["enabled"], json!(true));
    assert_eq!(answer["provider"]["local"]["model"], json!("llama3:8b"));
}

#[test]
fn смена_вида_не_теряет_настройки_остальных() {
    let case = case("switch");
    let mut asked = settings("http://127.0.0.1:11434", "local");
    save(&case, asked.clone(), Value::Null, false).unwrap();

    asked["active"] = json!("harness");
    save(&case, asked, Value::Null, false).unwrap();
    let answer = read(&case);

    assert_eq!(answer["provider"]["active"], json!("harness"));
    assert_eq!(answer["provider"]["local"]["model"], json!("llama3:8b"));
    assert_eq!(answer["provider"]["harness"]["command"], json!("claude"));
}

#[test]
fn ключ_ложится_в_хранилище_а_не_в_конфиг() {
    let case = case("key");

    let answer = save(
        &case,
        settings("https://api.example.test/v1", "remote"),
        json!("sk-секрет"),
        false,
    )
    .unwrap();

    assert_eq!(answer["has_key"], json!(true));
    assert_eq!(case.vault.key().unwrap(), Some("sk-секрет".to_owned()));
    let written = std::fs::read_to_string(case.data.join("provider.yaml")).unwrap();
    assert!(!written.contains("sk-секрет"), "{written}");
}

#[test]
fn забытый_ключ_уходит_из_хранилища() {
    let case = case("forget");
    save(
        &case,
        settings("https://api.example.test/v1", "remote"),
        json!("sk-секрет"),
        false,
    )
    .unwrap();

    let answer = ask(
        &case,
        json!({
            "save": Value::Null,
            "key": Value::Null,
            "forget": true,
            "check": false,
            "probe": false,
        }),
    )
    .unwrap();

    assert_eq!(answer["has_key"], json!(false));
    assert_eq!(case.vault.key().unwrap(), None);
}

#[test]
fn проверка_соединения_возвращает_модели() {
    let case = case("check");
    let heard = stub("200 OK", OLLAMA);

    let answer = save(&case, settings(&heard.endpoint, "local"), Value::Null, true).unwrap();

    assert_eq!(answer["checked"]["models"], json!(["llama3:8b"]));
    assert_eq!(answer["checked"]["version"], Value::Null);
}

#[test]
fn отказ_провайдера_приходит_кодом_ошибки() {
    let case = case("rejected");
    let heard = stub("401 Unauthorized", "{}");

    let failed = save(
        &case,
        settings(&heard.endpoint, "remote"),
        json!("sk-плохой"),
        true,
    )
    .unwrap_err();

    assert_eq!(failed.code, "provider.rejected");
}

#[test]
fn выключенный_провайдер_не_проверяется() {
    let case = case("disabled");
    let heard = stub("200 OK", OLLAMA);
    let mut asked = settings(&heard.endpoint, "local");
    asked["enabled"] = json!(false);

    let failed = save(&case, asked, Value::Null, true).unwrap_err();

    assert_eq!(failed.code, "provider.disabled");
}

#[test]
fn ненайденный_харнесс_приходит_своим_кодом() {
    let case = case("harness");
    let mut asked = settings("http://127.0.0.1:11434", "harness");
    asked["harness"] = harness("tolearn-нет-такой-команды");

    let failed = save(&case, asked, Value::Null, true).unwrap_err();

    assert_eq!(failed.code, "harness.not-found");
}

#[test]
fn неизвестный_вид_провайдера_отвергается() {
    let case = case("kind");

    let failed = save(
        &case,
        settings("http://127.0.0.1:11434", "anthropic"),
        Value::Null,
        false,
    )
    .unwrap_err();

    assert_eq!(failed.code, "provider.unknown-value");
}

#[test]
fn неизвестный_api_локальной_модели_отвергается() {
    let case = case("api");
    let mut asked = settings("http://127.0.0.1:8080/v1", "local");
    asked["local"]["api"] = json!("llama.cpp");

    let failed = save(&case, asked, Value::Null, false).unwrap_err();

    assert_eq!(failed.code, "provider.unknown-value");
}

#[test]
fn выбранный_api_переживает_перезапуск() {
    let case = case("api-stored");
    let mut asked = settings("http://127.0.0.1:8080/v1", "local");
    asked["local"]["api"] = json!("openai");

    save(&case, asked, Value::Null, false).unwrap();
    let answer = read(&case);

    assert_eq!(answer["provider"]["local"]["api"], json!("openai"));
}

#[test]
fn нелепая_температура_отвергается() {
    let case = case("temperature");
    let mut asked = settings("http://127.0.0.1:11434", "local");
    asked["local"]["temperature_tenths"] = json!(99);

    let failed = save(&case, asked, Value::Null, false).unwrap_err();

    assert_eq!(failed.code, "provider.unknown-value");
}

#[test]
fn контекст_и_температура_переживают_перезапуск() {
    let case = case("tuning-stored");
    let mut asked = settings("http://127.0.0.1:11434", "local");
    asked["local"]["num_ctx"] = json!(16_384);
    asked["local"]["temperature_tenths"] = json!(3);

    save(&case, asked, Value::Null, false).unwrap();
    let answer = read(&case);

    assert_eq!(answer["provider"]["local"]["num_ctx"], json!(16_384));
    assert_eq!(answer["provider"]["local"]["temperature_tenths"], json!(3));
}

#[test]
fn нелепый_таймаут_харнесса_отвергается() {
    let case = case("timeout");
    let mut asked = settings("http://127.0.0.1:11434", "harness");
    asked["harness"]["timeout_secs"] = json!(0);

    let failed = save(&case, asked, Value::Null, false).unwrap_err();

    assert_eq!(failed.code, "provider.unknown-value");
}
