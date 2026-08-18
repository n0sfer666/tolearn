#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "speech gate: a panic here is the report"
)]

use std::path::{Path, PathBuf};

const NAMED: [&str; 3] = ["whisper.cpp", "ggml-small-q5_1.bin", "fixtures/audio/"];

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or(Path::new("."))
        .to_path_buf()
}

fn attribution() -> String {
    let path = root().join("THIRD-PARTY.md");
    match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(reason) => panic!("{}: {reason}", path.display()),
    }
}

#[test]
fn чужое_названо_поимённо() {
    let text = attribution();

    for name in NAMED {
        assert!(text.contains(name), "в THIRD-PARTY.md нет {name}");
    }
}

#[test]
fn каждый_источник_записей_назван() {
    let text = attribution();
    let audio = root().join("fixtures/audio");
    let Ok(languages) = std::fs::read_dir(&audio) else {
        panic!("{}: корпуса нет", audio.display());
    };

    let mut clips = 0;
    for language in languages.flatten().filter(|entry| entry.path().is_dir()) {
        for file in std::fs::read_dir(language.path()).unwrap().flatten() {
            let path = file.path();
            if path.extension().and_then(|kind| kind.to_str()) != Some("wav") {
                continue;
            }
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            let Some((source, _)) = name.split_once('-') else {
                panic!("{name}: в имени записи нет префикса источника");
            };
            assert!(
                text.contains(&format!("`{source}-`")),
                "{name}: источник {source} не назван в THIRD-PARTY.md"
            );
            clips += 1;
        }
    }

    assert!(clips > 0, "в {} нет ни одной записи", audio.display());
}
