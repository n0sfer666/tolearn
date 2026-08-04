#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "provider gate: a panic here is the report"
)]

mod support;

use support::{closed, harness, stub};
use tolearn_provider::{CheckError, Http, Kind, Provider, check};

const OLLAMA: &str = r#"{"models":[{"name":"llama3:8b"},{"name":"qwen2.5"}]}"#;
const OPENAI: &str = r#"{"data":[{"id":"gpt-4o-mini"}]}"#;

fn provider(kind: Kind, endpoint: &str) -> Provider {
    let http = Http {
        endpoint: endpoint.to_owned(),
        model: String::new(),
    };
    Provider {
        enabled: true,
        active: kind,
        local: http.clone(),
        remote: http,
        ..Provider::default()
    }
}

#[test]
fn локальная_модель_отдаёт_список_моделей() {
    let stub = stub("200 OK", OLLAMA);

    let checked = check(&provider(Kind::Local, &stub.endpoint), None).unwrap();

    assert_eq!(checked.models, ["llama3:8b", "qwen2.5"]);
    assert_eq!(checked.version, None);
    assert!(
        stub.heard()[0].starts_with("GET /api/tags "),
        "{:?}",
        stub.heard()
    );
}

#[test]
fn внешний_endpoint_получает_ключ() {
    let stub = stub("200 OK", OPENAI);

    let checked = check(&provider(Kind::Remote, &stub.endpoint), Some("sk-test")).unwrap();

    assert_eq!(checked.models, ["gpt-4o-mini"]);
    let heard = stub.heard()[0].to_lowercase();
    assert!(heard.starts_with("get /models "), "{heard}");
    assert!(heard.contains("authorization: bearer sk-test"), "{heard}");
}

#[test]
fn харнесс_называет_версию() {
    let checked = check(&harness(&["say".to_owned()], 20), None).unwrap();

    assert_eq!(checked.models, Vec::<String>::new());
    assert_eq!(checked.version, Some("fake-harness 1.0".to_owned()));
}

#[test]
fn ненайденный_харнесс_объясняет_как_чинить() {
    let mut provider = harness(&[], 20);
    provider.harness.command = "tolearn-нет-такой-команды".to_owned();

    let failed = check(&provider, None).unwrap_err();

    assert_eq!(failed.code(), "harness.not-found");
    assert!(failed.to_string().contains("which"), "{failed}");
}

#[test]
fn отказ_по_ключу_виден_как_отказ() {
    let stub = stub("401 Unauthorized", r#"{"error":"no"}"#);

    let failed = check(&provider(Kind::Remote, &stub.endpoint), Some("sk-test")).unwrap_err();

    assert_eq!(failed, CheckError::Rejected);
}

#[test]
fn неожиданный_код_называется_кодом() {
    let stub = stub("503 Service Unavailable", "{}");

    let failed = check(&provider(Kind::Local, &stub.endpoint), None).unwrap_err();

    assert_eq!(failed, CheckError::Answered(503));
}

#[test]
fn чужой_формат_ответа_отвергается() {
    let stub = stub("200 OK", r#"{"greeting":"привет"}"#);

    let failed = check(&provider(Kind::Local, &stub.endpoint), None).unwrap_err();

    assert_eq!(failed, CheckError::BadAnswer);
}

#[test]
fn недоступный_endpoint_не_валит_проверку() {
    let failed = check(&provider(Kind::Local, &closed()), None).unwrap_err();

    assert!(matches!(failed, CheckError::Unreachable(_)), "{failed:?}");
}

#[test]
fn выключенный_провайдер_в_сеть_не_ходит() {
    let stub = stub("200 OK", OLLAMA);
    let mut provider = provider(Kind::Local, &stub.endpoint);
    provider.enabled = false;

    let failed = check(&provider, None).unwrap_err();

    assert_eq!(failed, CheckError::Disabled);
    assert!(stub.heard().is_empty(), "{:?}", stub.heard());
}

#[test]
fn выключенный_провайдер_процесс_не_запускает() {
    let mut provider = harness(&["say".to_owned()], 20);
    provider.enabled = false;

    assert_eq!(check(&provider, None).unwrap_err(), CheckError::Disabled);
}

#[test]
fn внешний_без_ключа_в_сеть_не_ходит() {
    let stub = stub("200 OK", OPENAI);

    let failed = check(&provider(Kind::Remote, &stub.endpoint), Some("  ")).unwrap_err();

    assert_eq!(failed, CheckError::NoKey);
    assert!(stub.heard().is_empty(), "{:?}", stub.heard());
}
