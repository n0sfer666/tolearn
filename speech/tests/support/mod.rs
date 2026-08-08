use std::path::{Path, PathBuf};

use tolearn_speech::listen::Listener;
use tolearn_speech::{Pcm, model, wav};

#[allow(
    dead_code,
    reason = "каждый тестовый бинарь берёт из записи свою часть"
)]
pub struct Clip {
    pub name: String,
    pub language: String,
    pub voice: Pcm,
    pub said: String,
}

pub fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or(Path::new("."))
        .to_path_buf()
}

pub fn skipped(why: &str) {
    eprintln!("пропущено: {why}");
}

pub fn model() -> Option<PathBuf> {
    match model::located(&root()) {
        Ok(path) => Some(path),
        Err(reason) => {
            skipped(&format!("{reason}; путь задаётся {}", model::ENV));
            None
        }
    }
}

pub fn listener() -> Option<Listener> {
    let path = model()?;
    match Listener::open(&path) {
        Ok(listener) => Some(listener),
        Err(reason) => {
            skipped(&reason.to_string());
            None
        }
    }
}

pub fn corpus() -> Vec<Clip> {
    let mut clips = Vec::new();
    let audio = root().join("fixtures/audio");
    let Ok(languages) = std::fs::read_dir(&audio) else {
        return clips;
    };
    for language in languages.flatten().filter(|entry| entry.path().is_dir()) {
        let tongue = language.file_name().to_string_lossy().into_owned();
        let Ok(files) = std::fs::read_dir(language.path()) else {
            continue;
        };
        for file in files.flatten() {
            let path = file.path();
            if path.extension().and_then(|kind| kind.to_str()) != Some("wav") {
                continue;
            }
            let beside = path.with_extension("txt");
            let voice = match wav::read(&path) {
                Ok(voice) => voice,
                Err(reason) => panic!("{}: {reason}", path.display()),
            };
            let said = match std::fs::read_to_string(&beside) {
                Ok(said) => said,
                Err(reason) => panic!("{}: {reason}", beside.display()),
            };
            clips.push(Clip {
                name: path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
                language: tongue.clone(),
                voice,
                said,
            });
        }
    }
    clips.sort_by(|left, right| (&left.language, &left.name).cmp(&(&right.language, &right.name)));
    clips
}
