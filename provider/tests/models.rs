#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "provider gate: a panic here is the report"
)]

use tolearn_provider::{Api, advised, memory};

#[test]
fn ollama_ставит_модель_своей_командой() {
    let advised = advised(Api::Ollama, 64);

    let first = advised.first().expect("таблица не пуста");
    assert!(first.command.starts_with("ollama pull "), "{first:?}");
    assert!(first.command.ends_with(&first.model), "{first:?}");
}

#[test]
fn openai_совместимому_серверу_модель_называют_репозиторием() {
    let advised = advised(Api::OpenAi, 64);

    let first = advised.first().expect("таблица не пуста");
    assert!(first.command.starts_with("llama-server -hf "), "{first:?}");
    assert!(first.model.contains('/'), "{first:?}");
}

#[test]
fn на_маленькой_машине_тяжёлые_модели_помечены() {
    let tight = advised(Api::Ollama, 8);
    let roomy = advised(Api::Ollama, 128);

    assert!(tight.iter().any(|model| model.heavy), "{tight:?}");
    assert!(tight.iter().any(|model| !model.heavy), "{tight:?}");
    assert!(roomy.iter().all(|model| !model.heavy), "{roomy:?}");
}

#[test]
fn неизвестный_объём_памяти_никого_не_чернит() {
    let advised = advised(Api::Ollama, 0);

    assert!(advised.iter().all(|model| !model.heavy), "{advised:?}");
}

#[test]
fn память_машины_измеряется_без_сети_и_без_зависимостей() {
    let gigabytes = memory();

    assert!(gigabytes > 0, "{gigabytes}");
    assert!(gigabytes < 4_096, "{gigabytes}");
}
