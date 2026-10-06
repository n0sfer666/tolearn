#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "provider gate: a panic here is the report"
)]

mod support;

use support::stub;
use tolearn_provider::{Api, Http, Kind, Provider, Said, Tokens, ask};

fn asked(body: &'static str, api: Api) -> Said {
    let heard = stub("200 OK", body);
    let provider = Provider {
        enabled: true,
        active: Kind::Local,
        local: Http {
            endpoint: heard.endpoint.clone(),
            api,
            model: "llama3:8b".to_owned(),
            ..Http::local()
        },
        ..Provider::default()
    };
    ask(&provider, None, "спроси").unwrap()
}

#[test]
fn ollama_делит_токены_на_вход_и_выход_и_называет_модель_ответа() {
    let answer = asked(
        r#"{"model":"llama3:8b-q4","message":{"content":"вердикт"},"prompt_eval_count":12,"eval_count":5}"#,
        Api::Ollama,
    );

    assert_eq!(
        answer.tokens,
        Tokens {
            input: Some(12),
            output: Some(5)
        }
    );
    assert_eq!(answer.model.as_deref(), Some("llama3:8b-q4"));
}

#[test]
fn ollama_без_счёта_промпта_не_выдумывает_ноль_а_модель_берёт_из_запроса() {
    let answer = asked(
        r#"{"message":{"content":"вердикт"},"eval_count":5}"#,
        Api::Ollama,
    );

    assert_eq!(
        answer.tokens,
        Tokens {
            input: None,
            output: Some(5)
        }
    );
    assert_eq!(answer.model.as_deref(), Some("llama3:8b"));
}

#[test]
fn openai_совместимый_делит_usage_на_вход_и_выход() {
    let answer = asked(
        r#"{"model":"qwen3-32b","choices":[{"message":{"content":"вердикт"}}],"usage":{"prompt_tokens":20,"completion_tokens":4,"total_tokens":24}}"#,
        Api::OpenAi,
    );

    assert_eq!(
        answer.tokens,
        Tokens {
            input: Some(20),
            output: Some(4)
        }
    );
    assert_eq!(answer.model.as_deref(), Some("qwen3-32b"));
}

#[test]
fn ответ_без_usage_оставляет_токены_без_данных() {
    let answer = asked(
        r#"{"choices":[{"message":{"content":"вердикт"}}]}"#,
        Api::OpenAi,
    );

    assert_eq!(answer.tokens, Tokens::default());
}
