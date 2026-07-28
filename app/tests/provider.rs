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
        json!({ "save": Value::Null, "key": Value::Null, "forget": false, "check": false }),
    )
    .unwrap()
}

fn save(case: &Case, provider: Value, key: Value, check: bool) -> Result<Value, IpcError> {
    ask(
        case,
        json!({ "save": provider, "key": key, "forget": false, "check": check }),
    )
}

fn settings(endpoint: &str, flavor: &str) -> Value {
    json!({ "enabled": true, "flavor": flavor, "endpoint": endpoint, "model": "llama3:8b" })
}

#[test]
fn провайдер_выключен_пока_его_не_включили() {
    let case = case("default");

    let answer = read(&case);

    assert_eq!(answer["provider"]["enabled"], json!(false));
    assert_eq!(answer["provider"]["flavor"], json!("ollama"));
    assert_eq!(answer["has_key"], json!(false));
    assert_eq!(answer["checked"], Value::Null);
}

#[test]
fn настройки_переживают_перезапуск() {
    let case = case("stored");

    save(
        &case,
        settings("http://127.0.0.1:11434", "ollama"),
        Value::Null,
        false,
    )
    .unwrap();
    let answer = read(&case);

    assert_eq!(answer["provider"]["enabled"], json!(true));
    assert_eq!(answer["provider"]["model"], json!("llama3:8b"));
}

#[test]
fn ключ_ложится_в_хранилище_а_не_в_конфиг() {
    let case = case("key");

    let answer = save(
        &case,
        settings("https://api.example.test/v1", "openai"),
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
        settings("https://api.example.test/v1", "openai"),
        json!("sk-секрет"),
        false,
    )
    .unwrap();

    let answer = ask(
        &case,
        json!({ "save": Value::Null, "key": Value::Null, "forget": true, "check": false }),
    )
    .unwrap();

    assert_eq!(answer["has_key"], json!(false));
    assert_eq!(case.vault.key().unwrap(), None);
}

#[test]
fn проверка_соединения_возвращает_модели() {
    let case = case("check");
    let heard = stub("200 OK", OLLAMA);

    let answer = save(
        &case,
        settings(&heard.endpoint, "ollama"),
        Value::Null,
        true,
    )
    .unwrap();

    assert_eq!(answer["checked"]["models"], json!(["llama3:8b"]));
}

#[test]
fn отказ_провайдера_приходит_кодом_ошибки() {
    let case = case("rejected");
    let heard = stub("401 Unauthorized", "{}");

    let failed = save(
        &case,
        settings(&heard.endpoint, "openai"),
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
    let mut asked = settings(&heard.endpoint, "ollama");
    asked["enabled"] = json!(false);

    let failed = save(&case, asked, Value::Null, true).unwrap_err();

    assert_eq!(failed.code, "provider.disabled");
}

#[test]
fn неизвестный_вид_провайдера_отвергается() {
    let case = case("flavor");

    let failed = save(
        &case,
        settings("http://127.0.0.1:11434", "anthropic"),
        Value::Null,
        false,
    )
    .unwrap_err();

    assert_eq!(failed.code, "provider.unknown-value");
}
