#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "provider gate: a panic here is the report"
)]

use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use tolearn_provider::{DEFAULT_ENDPOINT, Flavor, Keychain, Provider, Remembered, Vault};

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

#[test]
fn по_умолчанию_провайдер_выключен() {
    let provider = Provider::default();

    assert!(!provider.enabled);
    assert_eq!(provider.flavor, Flavor::Ollama);
    assert_eq!(provider.endpoint, DEFAULT_ENDPOINT);
    assert_eq!(provider.model, "");
}

#[test]
fn отсутствующий_конфиг_читается_как_умолчание() {
    let stored = Provider::read(&path("missing")).unwrap();

    assert_eq!(stored, Provider::default());
}

#[test]
fn настройки_переживают_запись_и_чтение() {
    let file = path("roundtrip");
    let provider = Provider {
        enabled: true,
        flavor: Flavor::OpenAi,
        endpoint: "https://api.example.test/v1".to_owned(),
        model: "gpt-4o-mini".to_owned(),
    };

    provider.save(&file).unwrap();

    assert_eq!(Provider::read(&file).unwrap(), provider);
}

#[test]
fn ключ_не_попадает_в_файл_конфига() {
    let file = path("secret");
    let vault = Remembered::default();
    vault.store("sk-очень-секретный-ключ").unwrap();
    let provider = Provider {
        enabled: true,
        flavor: Flavor::OpenAi,
        endpoint: "https://api.example.test/v1".to_owned(),
        model: "gpt-4o-mini".to_owned(),
    };

    provider.save(&file).unwrap();
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
