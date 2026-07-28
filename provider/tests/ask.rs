mod support;

use tolearn_provider::{CheckError, Flavor, Provider, ask};

use support::{closed, stub};

fn provider(endpoint: &str, flavor: Flavor, model: &str) -> Provider {
    Provider {
        enabled: true,
        flavor,
        endpoint: endpoint.to_owned(),
        model: model.to_owned(),
    }
}

#[test]
fn ollama_возвращает_текст_ответа() {
    let heard = stub(
        "200 OK",
        r#"{"message":{"content":"{\"result\":\"pass\"}"}}"#,
    );
    let asked = provider(&heard.endpoint, Flavor::Ollama, "llama3:8b");

    let answer = ask(&asked, None, "спроси меня").expect("ollama отвечает");

    assert_eq!(answer, "{\"result\":\"pass\"}");
    let request = heard.heard().join("\n");
    assert!(request.contains("POST /api/chat"), "{request}");
    assert!(request.contains("llama3:8b"), "{request}");
    assert!(request.contains("спроси меня"), "{request}");
}

#[test]
fn openai_совместимый_путь_получает_ключ() {
    let heard = stub(
        "200 OK",
        r#"{"choices":[{"message":{"content":"вердикт"}}]}"#,
    );
    let asked = provider(&heard.endpoint, Flavor::OpenAi, "gpt-4o-mini");

    let answer = ask(&asked, Some("sk-ключ"), "спроси меня").expect("openai отвечает");

    assert_eq!(answer, "вердикт");
    let request = heard.heard().join("\n");
    assert!(request.contains("POST /chat/completions"), "{request}");
    assert!(request.contains("Bearer sk-ключ"), "{request}");
}

#[test]
fn отказ_по_ключу_виден_как_отказ() {
    let heard = stub("401 Unauthorized", "{}");
    let asked = provider(&heard.endpoint, Flavor::OpenAi, "gpt-4o-mini");

    assert_eq!(
        ask(&asked, Some("sk-чужой"), "спроси"),
        Err(CheckError::Rejected)
    );
}

#[test]
fn неожиданный_код_называется_кодом() {
    let heard = stub("503 Service Unavailable", "{}");
    let asked = provider(&heard.endpoint, Flavor::Ollama, "llama3:8b");

    assert_eq!(ask(&asked, None, "спроси"), Err(CheckError::Answered(503)));
}

#[test]
fn чужой_формат_ответа_отвергается() {
    let heard = stub("200 OK", r#"{"answer":"вердикт"}"#);
    let asked = provider(&heard.endpoint, Flavor::Ollama, "llama3:8b");

    assert_eq!(ask(&asked, None, "спроси"), Err(CheckError::BadAnswer));
}

#[test]
fn недоступный_endpoint_не_валит_запрос() {
    let asked = provider(&closed(), Flavor::Ollama, "llama3:8b");

    let failed = ask(&asked, None, "спроси").expect_err("порт закрыт");

    assert_eq!(failed.code(), "provider.unreachable");
}

#[test]
fn выключенный_провайдер_в_сеть_не_ходит() {
    let heard = stub("200 OK", r#"{"message":{"content":"вердикт"}}"#);
    let mut asked = provider(&heard.endpoint, Flavor::Ollama, "llama3:8b");
    asked.enabled = false;

    assert_eq!(ask(&asked, None, "спроси"), Err(CheckError::Disabled));
    assert!(heard.heard().is_empty());
}

#[test]
fn без_модели_в_сеть_не_ходит() {
    let heard = stub("200 OK", r#"{"message":{"content":"вердикт"}}"#);
    let asked = provider(&heard.endpoint, Flavor::Ollama, "  ");

    assert_eq!(ask(&asked, None, "спроси"), Err(CheckError::NoModel));
    assert!(heard.heard().is_empty());
}

#[test]
fn openai_без_ключа_в_сеть_не_ходит() {
    let heard = stub("200 OK", r#"{"choices":[{"message":{"content":"в"}}]}"#);
    let asked = provider(&heard.endpoint, Flavor::OpenAi, "gpt-4o-mini");

    assert_eq!(ask(&asked, None, "спроси"), Err(CheckError::NoKey));
    assert!(heard.heard().is_empty());
}
