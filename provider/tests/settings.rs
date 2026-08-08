#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "provider gate: a panic here is the report"
)]

use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use tolearn_provider::{
    Api, DEFAULT_ENDPOINT, DEFAULT_TEMPERATURE_TENTHS, DEFAULT_TIMEOUT_SECS, Harness, Http,
    Keychain, Kind, OPENAI_ENDPOINT, Provider, Remembered, Vault,
};

static FILES: AtomicUsize = AtomicUsize::new(0);

fn path(name: &str) -> PathBuf {
    let directory = std::env::temp_dir().join(format!(
        "tolearn-provider-{name}-{}-{}",
        std::process::id(),
        FILES.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).unwrap();
    directory.join("provider.yaml")
}

fn filled(active: Kind) -> Provider {
    Provider {
        enabled: true,
        active,
        local: Http {
            endpoint: OPENAI_ENDPOINT.to_owned(),
            api: Api::OpenAi,
            model: "qwen3:8b".to_owned(),
            ..Http::local()
        },
        remote: Http {
            endpoint: "https://api.example.test/v1".to_owned(),
            api: Api::OpenAi,
            model: "gpt-4o-mini".to_owned(),
            ..Http::remote()
        },
        harness: Harness {
            id: "custom".to_owned(),
            command: "/usr/local/bin/claude".to_owned(),
            args: vec!["-p".to_owned(), "--allowedTools".to_owned(), String::new()],
            timeout_secs: 42,
        },
    }
}

#[test]
fn по_умолчанию_провайдер_выключен() {
    let provider = Provider::default();

    assert!(!provider.enabled);
    assert_eq!(provider.active, Kind::Local);
    assert_eq!(provider.local.endpoint, DEFAULT_ENDPOINT);
    assert_eq!(provider.local.api, Api::Ollama);
    assert_eq!(provider.local.model, "");
    assert_eq!(provider.remote.endpoint, "");
    assert_eq!(provider.remote.api, Api::OpenAi);
    assert_eq!(provider.harness.id, "claude");
    assert_eq!(provider.harness.command, "claude");
    assert_eq!(provider.harness.args, Vec::<String>::new());
    assert_eq!(provider.harness.timeout_secs, DEFAULT_TIMEOUT_SECS);
}

#[test]
fn отсутствующий_конфиг_читается_как_умолчание() {
    let stored = Provider::read(&path("missing")).unwrap();

    assert_eq!(stored, Provider::default());
}

#[test]
fn настройки_трёх_видов_переживают_запись_и_чтение() {
    let file = path("roundtrip");
    let provider = filled(Kind::Harness);

    provider.save(&file).unwrap();

    assert_eq!(Provider::read(&file).unwrap(), provider);
}

#[test]
fn смена_вида_не_теряет_настройки_остальных() {
    let file = path("switch");
    let mut provider = filled(Kind::Local);
    provider.save(&file).unwrap();

    provider.active = Kind::Harness;
    provider.save(&file).unwrap();
    let stored = Provider::read(&file).unwrap();

    assert_eq!(stored.active, Kind::Harness);
    assert_eq!(stored.local.model, "qwen3:8b");
    assert_eq!(stored.remote.endpoint, "https://api.example.test/v1");
}

#[test]
fn выбранный_api_переживает_запись_и_чтение() {
    let file = path("api");
    let mut provider = filled(Kind::Local);
    provider.local.api = Api::Ollama;
    provider.save(&file).unwrap();

    let stored = Provider::read(&file).unwrap();

    assert_eq!(stored.local.api, Api::Ollama);
    assert_eq!(stored.remote.api, Api::OpenAi);
}

#[test]
fn контекст_и_температура_переживают_запись_и_чтение() {
    let file = path("tuning");
    let mut provider = filled(Kind::Local);
    provider.local.num_ctx = 16_384;
    provider.local.temperature_tenths = 3;
    provider.save(&file).unwrap();

    let stored = Provider::read(&file).unwrap();

    assert_eq!(stored.local.num_ctx, 16_384);
    assert_eq!(stored.local.temperature_tenths, 3);
}

#[test]
fn конфиг_без_контекста_и_температуры_читается_как_прежде() {
    let file = path("v2-no-tuning");
    std::fs::write(
        &file,
        concat!(
            "schema: tolearn/provider/v2\n",
            "enabled: true\n",
            "active: local\n",
            "local:\n  endpoint: http://127.0.0.1:11434\n  api: ollama\n  model: qwen3:8b\n",
            "remote:\n  endpoint: https://api.example.test/v1\n  api: openai\n  model: gpt-4o-mini\n",
            "harness:\n  id: claude\n  command: claude\n  args:\n    - -p\n  timeout_secs: 180\n",
        ),
    )
    .unwrap();

    let stored = Provider::read(&file).unwrap();

    assert_eq!(stored.local.num_ctx, 0);
    assert_eq!(stored.local.temperature_tenths, DEFAULT_TEMPERATURE_TENTHS);
}

#[test]
fn конфиг_без_поля_api_читается_как_прежде() {
    let file = path("v2-no-api");
    std::fs::write(
        &file,
        concat!(
            "schema: tolearn/provider/v2\n",
            "enabled: true\n",
            "active: local\n",
            "local:\n  endpoint: http://127.0.0.1:11434\n  model: qwen3:8b\n",
            "remote:\n  endpoint: https://api.example.test/v1\n  model: gpt-4o-mini\n",
            "harness:\n  id: claude\n  command: claude\n  args:\n    - -p\n  timeout_secs: 180\n",
        ),
    )
    .unwrap();

    let stored = Provider::read(&file).unwrap();

    assert_eq!(stored.local.api, Api::Ollama);
    assert_eq!(stored.remote.api, Api::OpenAi);
}

#[test]
fn старый_конфиг_ollama_переезжает_в_local() {
    let file = path("v1-ollama");
    std::fs::write(
        &file,
        concat!(
            "schema: tolearn/provider/v1\n",
            "enabled: true\n",
            "flavor: ollama\n",
            "endpoint: http://127.0.0.1:11434\n",
            "model: qwen3:8b\n",
        ),
    )
    .unwrap();

    let stored = Provider::read(&file).unwrap();

    assert!(stored.enabled);
    assert_eq!(stored.active, Kind::Local);
    assert_eq!(stored.local.model, "qwen3:8b");
    assert_eq!(stored.local.endpoint, DEFAULT_ENDPOINT);
    assert_eq!(stored.local.api, Api::Ollama);
    assert_eq!(stored.harness, Provider::default().harness);
}

#[test]
fn старый_конфиг_openai_переезжает_в_remote() {
    let file = path("v1-openai");
    std::fs::write(
        &file,
        concat!(
            "schema: tolearn/provider/v1\n",
            "enabled: true\n",
            "flavor: openai\n",
            "endpoint: https://api.example.test/v1\n",
            "model: gpt-4o-mini\n",
        ),
    )
    .unwrap();

    let stored = Provider::read(&file).unwrap();

    assert_eq!(stored.active, Kind::Remote);
    assert_eq!(stored.remote.endpoint, "https://api.example.test/v1");
    assert_eq!(stored.remote.model, "gpt-4o-mini");
    assert_eq!(stored.local.endpoint, DEFAULT_ENDPOINT);
}

#[test]
fn переехавший_конфиг_записывается_уже_как_v2() {
    let file = path("v1-rewrite");
    std::fs::write(
        &file,
        concat!(
            "schema: tolearn/provider/v1\n",
            "enabled: false\n",
            "flavor: ollama\n",
            "endpoint: http://127.0.0.1:11434\n",
            "model: \"\"\n",
        ),
    )
    .unwrap();

    let stored = Provider::read(&file).unwrap();
    stored.save(&file).unwrap();
    let written = std::fs::read_to_string(&file).unwrap();

    assert!(written.contains("schema: tolearn/provider/v2"), "{written}");
    assert!(!written.contains("flavor"), "{written}");
    assert_eq!(Provider::read(&file).unwrap(), stored);
}

#[test]
fn ключ_не_попадает_в_файл_конфига() {
    let file = path("secret");
    let vault = Remembered::default();
    vault.store("sk-очень-секретный-ключ").unwrap();

    filled(Kind::Remote).save(&file).unwrap();
    let written = std::fs::read_to_string(&file).unwrap();

    assert!(!written.contains("sk-очень-секретный-ключ"), "{written}");
    assert!(!written.contains("key"), "{written}");
    assert_eq!(
        vault.key().unwrap(),
        Some("sk-очень-секретный-ключ".to_owned())
    );
}

#[test]
fn ключ_кладётся_читается_и_забывается() {
    let vault = Remembered::default();

    assert_eq!(vault.key().unwrap(), None);
    vault.store("sk-first").unwrap();
    assert_eq!(vault.key().unwrap(), Some("sk-first".to_owned()));
    vault.store("sk-second").unwrap();
    assert_eq!(vault.key().unwrap(), Some("sk-second".to_owned()));
    vault.forget().unwrap();
    assert_eq!(vault.key().unwrap(), None);
    vault.forget().unwrap();
}

#[test]
fn системное_хранилище_помнит_ключ() {
    if std::env::var("TOLEARN_KEYCHAIN").is_err() {
        return;
    }
    let vault = Keychain::new("tolearn-test", "provider");

    vault.store("sk-keychain").unwrap();
    assert_eq!(vault.key().unwrap(), Some("sk-keychain".to_owned()));
    vault.forget().unwrap();
    assert_eq!(vault.key().unwrap(), None);
}
