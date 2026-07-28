#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "provider gate: a panic here is the report"
)]

mod support;

use support::{closed, stub};
use tolearn_provider::{CheckError, Flavor, Provider, check};

const OLLAMA: &str = r#"{"models":[{"name":"llama3:8b"},{"name":"qwen2.5"}]}"#;
const OPENAI: &str = r#"{"data":[{"id":"gpt-4o-mini"}]}"#;

fn provider(flavor: Flavor, endpoint: &str) -> Provider {
    Provider {
        enabled: true,
        flavor,
        endpoint: endpoint.to_owned(),
        model: String::new(),
    }
}

#[test]
fn ollama_отдаёт_список_моделей() {
    let stub = stub("200 OK", OLLAMA);

    let checked = check(&provider(Flavor::Ollama, &stub.endpoint), None).unwrap();

    assert_eq!(checked.models, ["llama3:8b", "qwen2.5"]);
    assert!(
        stub.heard()[0].starts_with("GET /api/tags "),
        "{:?}",
        stub.heard()
    );
}

#[test]
fn openai_совместимый_endpoint_получает_ключ() {
    let stub = stub("200 OK", OPENAI);

    let checked = check(&provider(Flavor::OpenAi, &stub.endpoint), Some("sk-test")).unwrap();

    assert_eq!(checked.models, ["gpt-4o-mini"]);
    let heard = stub.heard()[0].to_lowercase();
    assert!(heard.starts_with("get /models "), "{heard}");
    assert!(heard.contains("authorization: bearer sk-test"), "{heard}");
}

#[test]
fn отказ_по_ключу_виден_как_отказ() {
    let stub = stub("401 Unauthorized", r#"{"error":"no"}"#);

    let failed = check(&provider(Flavor::OpenAi, &stub.endpoint), Some("sk-test")).unwrap_err();

    assert_eq!(failed, CheckError::Rejected);
}

#[test]
fn неожиданный_код_называется_кодом() {
    let stub = stub("503 Service Unavailable", "{}");

    let failed = check(&provider(Flavor::Ollama, &stub.endpoint), None).unwrap_err();

    assert_eq!(failed, CheckError::Answered(503));
}

#[test]
fn чужой_формат_ответа_отвергается() {
    let stub = stub("200 OK", r#"{"greeting":"привет"}"#);

    let failed = check(&provider(Flavor::Ollama, &stub.endpoint), None).unwrap_err();

    assert_eq!(failed, CheckError::BadAnswer);
}

#[test]
fn недоступный_endpoint_не_валит_проверку() {
    let failed = check(&provider(Flavor::Ollama, &closed()), None).unwrap_err();

    assert!(matches!(failed, CheckError::Unreachable(_)), "{failed:?}");
}

#[test]
fn выключенный_провайдер_в_сеть_не_ходит() {
    let stub = stub("200 OK", OLLAMA);
    let mut provider = provider(Flavor::Ollama, &stub.endpoint);
    provider.enabled = false;

    let failed = check(&provider, None).unwrap_err();

    assert_eq!(failed, CheckError::Disabled);
    assert!(stub.heard().is_empty(), "{:?}", stub.heard());
}

#[test]
fn openai_без_ключа_в_сеть_не_ходит() {
    let stub = stub("200 OK", OPENAI);

    let failed = check(&provider(Flavor::OpenAi, &stub.endpoint), Some("  ")).unwrap_err();

    assert_eq!(failed, CheckError::NoKey);
    assert!(stub.heard().is_empty(), "{:?}", stub.heard());
}
