#![cfg(feature = "speech")]
#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "speech gate: a panic here is the report"
)]

use std::time::Instant;

use tolearn_speech::wer::{Score, score};

mod support;

const WORST: f64 = 0.05;
const SLOWEST: f64 = 1.0;
const HEAVIEST: u64 = 192 * 1024 * 1024;

#[test]
fn корпус_расшифровывается_в_бюджет_ошибок_и_времени() {
    let Some(listener) = support::listener() else {
        return;
    };
    let clips = support::corpus();
    if clips.is_empty() {
        support::skipped("в fixtures/audio/ нет ни одной записи");
        return;
    }

    let mut total = Score::default();
    let mut speech = 0.0;
    let mut spent = 0.0;
    for clip in &clips {
        let started = Instant::now();
        let heard = listener.hear(&clip.voice, &clip.language).unwrap();
        spent += started.elapsed().as_secs_f64();
        speech += clip.voice.seconds();

        let clip_score = score(&clip.said, &heard);
        eprintln!(
            "{}: ошибок {} из {} слов, услышано «{heard}»",
            clip.name, clip_score.errors, clip_score.words
        );
        total += clip_score;
    }

    let rate = total.rate();
    let slowdown = spent / speech;
    eprintln!(
        "корпус: {:.1} с речи, {:.1} с работы, WER {:.3}, замедление {:.2}",
        speech, spent, rate, slowdown
    );

    assert!(rate <= WORST, "WER {rate:.3} выше бюджета {WORST}");
    assert!(
        slowdown <= SLOWEST,
        "на секунду речи ушло {slowdown:.2} с при бюджете {SLOWEST}"
    );
}

#[test]
fn веса_модели_укладываются_в_бюджет() {
    let Some(path) = support::model() else {
        return;
    };

    let weight = std::fs::metadata(&path).unwrap().len();

    eprintln!("веса: {weight} байт");
    assert!(
        weight <= HEAVIEST,
        "веса {weight} байт тяжелее бюджета {HEAVIEST}"
    );
}
