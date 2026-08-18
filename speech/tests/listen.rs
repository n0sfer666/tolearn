#![cfg(feature = "speech")]
#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "speech gate: a panic here is the report"
)]

use tolearn_speech::listen::Listener;
use tolearn_speech::{Pcm, SpeechError};

mod support;

#[test]
fn распознавание_собрано_в_этом_варианте() {
    assert!(tolearn_speech::available());
}

#[test]
fn чужой_файл_вместо_весов_отвергается_с_путём() {
    match Listener::open(&support::root().join("speech/Cargo.toml")) {
        Err(SpeechError::Rejected { path, reason }) => {
            assert!(path.ends_with("Cargo.toml"), "{path:?}");
            assert!(!reason.is_empty());
        }
        other => panic!("не тот исход: {other:?}"),
    }
}

#[test]
fn тишина_не_уходит_в_модель() {
    let Some(listener) = support::listener() else {
        return;
    };

    assert!(matches!(
        listener.hear(&Pcm::default(), "ru"),
        Err(SpeechError::Silent)
    ));
}

#[test]
fn речь_из_фикстуры_расшифровывается() {
    let Some(listener) = support::listener() else {
        return;
    };
    let Some(clip) = support::corpus().into_iter().next() else {
        support::skipped("в fixtures/audio/ нет ни одной записи");
        return;
    };

    let heard = listener.hear(&clip.voice, &clip.language).unwrap();

    assert!(!heard.is_empty(), "{}: модель промолчала", clip.name);
}
