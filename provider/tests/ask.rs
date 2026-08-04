#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "provider gate: a panic here is the report"
)]

mod support;

use support::{closed, harness, stub};
use tolearn_provider::{Api, CheckError, Http, Kind, Provider, ask, probe};

fn http(endpoint: &str, api: Api, model: &str) -> Http {
    Http {
        endpoint: endpoint.to_owned(),
        api,
        model: model.to_owned(),
    }
}

fn provider(endpoint: &str, kind: Kind, model: &str) -> Provider {
    Provider {
        enabled: true,
        active: kind,
        local: http(endpoint, Api::Ollama, model),
        remote: http(endpoint, Api::OpenAi, model),
        ..Provider::default()
    }
}

#[test]
fn локальная_модель_возвращает_текст_ответа() {
    let heard = stub(
        "200 OK",
        r#"{"message":{"content":"{\"result\":\"pass\"}"}}"#,
    );
    let asked = provider(&heard.endpoint, Kind::Local, "llama3:8b");

    let answer = ask(&asked, None, "спроси меня").expect("локальная модель отвечает");

    assert_eq!(answer, "{\"result\":\"pass\"}");
    let request = heard.heard().join("\n");
    assert!(request.contains("POST /api/chat"), "{request}");
    assert!(request.contains("llama3:8b"), "{request}");
    assert!(request.contains("спроси меня"), "{request}");
}

#[test]
fn локальный_openai_совместимый_сервер_отвечает_без_ключа() {
    let heard = stub(
        "200 OK",
        r#"{"choices":[{"message":{"content":"вердикт"}}]}"#,
    );
    let mut asked = provider(&heard.endpoint, Kind::Local, "qwen3");
    asked.local.api = Api::OpenAi;

    let answer = ask(&asked, None, "спроси меня").expect("llama.cpp отвечает");

    assert_eq!(answer, "вердикт");
    let request = heard.heard().join("\n");
    assert!(request.contains("POST /chat/completions"), "{request}");
    assert!(
        !request.to_lowercase().contains("authorization"),
        "{request}"
    );
}

#[test]
fn внешний_путь_получает_ключ() {
    let heard = stub(
        "200 OK",
        r#"{"choices":[{"message":{"content":"вердикт"}}]}"#,
    );
    let asked = provider(&heard.endpoint, Kind::Remote, "gpt-4o-mini");

    let answer = ask(&asked, Some("sk-ключ"), "спроси меня").expect("внешний отвечает");

    assert_eq!(answer, "вердикт");
    let request = heard.heard().join("\n");
    assert!(request.contains("POST /chat/completions"), "{request}");
    assert!(request.contains("Bearer sk-ключ"), "{request}");
}

#[test]
fn харнесс_получает_промпт_через_stdin() {
    let asked = harness(&["say".to_owned()], 20);

    let answer = ask(&asked, None, "спроси меня").expect("харнесс отвечает");

    assert_eq!(answer, "услышал: спроси меня");
}

#[test]
fn пробный_запрос_меряет_время_и_показывает_ответ() {
    let asked = harness(&["say".to_owned()], 20);

    let probed = probe(&asked, None).expect("харнесс отвечает");

    assert!(probed.said.starts_with("услышал: "), "{}", probed.said);
    assert!(probed.said.contains("одним словом"), "{}", probed.said);
    assert!(probed.took_ms < 20_000, "{}", probed.took_ms);
}

#[test]
fn пробный_запрос_просит_короткий_ответ() {
    let heard = stub("200 OK", r#"{"choices":[{"message":{"content":"готов"}}]}"#);
    let mut asked = provider(&heard.endpoint, Kind::Local, "qwen3");
    asked.local.api = Api::OpenAi;

    probe(&asked, None).expect("сервер отвечает");

    let request = heard.heard().join("\n");
    assert!(request.contains("\"max_tokens\""), "{request}");
}

#[test]
fn пробный_запрос_к_ollama_ограничен_её_полем() {
    let heard = stub("200 OK", r#"{"message":{"content":"готов"}}"#);
    let asked = provider(&heard.endpoint, Kind::Local, "llama3:8b");

    probe(&asked, None).expect("сервер отвечает");

    let request = heard.heard().join("\n");
    assert!(request.contains("\"num_predict\""), "{request}");
}

#[test]
fn пробный_запрос_принимает_рассуждение_вместо_пустого_ответа() {
    let heard = stub(
        "200 OK",
        r#"{"choices":[{"message":{"content":"","reasoning_content":"думаю, что готов"}}]}"#,
    );
    let mut asked = provider(&heard.endpoint, Kind::Local, "qwen3");
    asked.local.api = Api::OpenAi;

    let probed = probe(&asked, None).expect("рассуждение — тоже ответ");

    assert_eq!(probed.said, "думаю, что готов");
    assert!(probed.thinking);
}

#[test]
fn обычный_ответ_рассуждением_не_зовётся() {
    let heard = stub("200 OK", r#"{"choices":[{"message":{"content":"готов"}}]}"#);
    let mut asked = provider(&heard.endpoint, Kind::Local, "qwen3");
    asked.local.api = Api::OpenAi;

    let probed = probe(&asked, None).expect("сервер отвечает");

    assert_eq!(probed.said, "готов");
    assert!(!probed.thinking);
}

#[test]
fn экзамен_не_подменяет_ответ_рассуждением_и_длину_не_режет() {
    let heard = stub(
        "200 OK",
        r#"{"choices":[{"message":{"content":"","reasoning_content":"думаю"}}]}"#,
    );
    let mut asked = provider(&heard.endpoint, Kind::Local, "qwen3");
    asked.local.api = Api::OpenAi;

    assert_eq!(ask(&asked, None, "спроси"), Err(CheckError::BadAnswer));
    let request = heard.heard().join("\n");
    assert!(!request.contains("max_tokens"), "{request}");
}

#[test]
fn отказ_по_ключу_виден_как_отказ() {
    let heard = stub("401 Unauthorized", "{}");
    let asked = provider(&heard.endpoint, Kind::Remote, "gpt-4o-mini");

    assert_eq!(
        ask(&asked, Some("sk-чужой"), "спроси"),
        Err(CheckError::Rejected)
    );
}

#[test]
fn неожиданный_код_называется_кодом() {
    let heard = stub("503 Service Unavailable", "{}");
    let asked = provider(&heard.endpoint, Kind::Local, "llama3:8b");

    assert_eq!(ask(&asked, None, "спроси"), Err(CheckError::Answered(503)));
}

#[test]
fn чужой_формат_ответа_отвергается() {
    let heard = stub("200 OK", r#"{"answer":"вердикт"}"#);
    let asked = provider(&heard.endpoint, Kind::Local, "llama3:8b");

    assert_eq!(ask(&asked, None, "спроси"), Err(CheckError::BadAnswer));
}

#[test]
fn недоступный_endpoint_не_валит_запрос() {
    let asked = provider(&closed(), Kind::Local, "llama3:8b");

    let failed = ask(&asked, None, "спроси").expect_err("порт закрыт");

    assert_eq!(failed.code(), "provider.unreachable");
}

#[test]
fn выключенный_провайдер_в_сеть_не_ходит() {
    let heard = stub("200 OK", r#"{"message":{"content":"вердикт"}}"#);
    let mut asked = provider(&heard.endpoint, Kind::Local, "llama3:8b");
    asked.enabled = false;

    assert_eq!(ask(&asked, None, "спроси"), Err(CheckError::Disabled));
    assert!(heard.heard().is_empty());
}

#[test]
fn без_модели_в_сеть_не_ходит() {
    let heard = stub("200 OK", r#"{"message":{"content":"вердикт"}}"#);
    let asked = provider(&heard.endpoint, Kind::Local, "  ");

    assert_eq!(ask(&asked, None, "спроси"), Err(CheckError::NoModel));
    assert!(heard.heard().is_empty());
}

#[test]
fn внешний_без_ключа_в_сеть_не_ходит() {
    let heard = stub("200 OK", r#"{"choices":[{"message":{"content":"в"}}]}"#);
    let asked = provider(&heard.endpoint, Kind::Remote, "gpt-4o-mini");

    assert_eq!(ask(&asked, None, "спроси"), Err(CheckError::NoKey));
    assert!(heard.heard().is_empty());
}
