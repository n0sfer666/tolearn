#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "app gate: a panic here is the report"
)]

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

static COPIES: AtomicUsize = AtomicUsize::new(0);

pub fn repository() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("рядом с app лежит корень репозитория")
        .to_path_buf()
}

pub fn copied(name: &str) -> PathBuf {
    let directory = std::env::temp_dir().join(format!(
        "tolearn-app-{name}-{}-{}",
        std::process::id(),
        COPIES.fetch_add(1, Ordering::Relaxed)
    ));
    if directory.exists() {
        std::fs::remove_dir_all(&directory).unwrap();
    }
    copy(&repository().join("examples/llm-agents-base"), &directory);
    strip(&directory, "json");
    directory
}

fn strip(directory: &Path, extension: &str) {
    for entry in std::fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            strip(&path, extension);
            continue;
        }
        if path.extension().is_some_and(|found| found == extension) {
            std::fs::remove_file(path).unwrap();
        }
    }
}

fn copy(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}

#[allow(dead_code, reason = "нужна не каждому тест-бинарнику")]
pub fn sources(directory: &Path, found: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            sources(&path, found);
            continue;
        }
        if path.extension().is_some_and(|extension| extension == "rs") {
            found.push(path);
        }
    }
}
