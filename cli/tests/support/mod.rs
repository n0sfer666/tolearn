#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "cli gate: a panic here is the report"
)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

pub fn reference() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("рядом с cli лежит корень репозитория")
        .join("examples/llm-agents-base")
}

pub fn scratch(name: &str) -> PathBuf {
    let directory = std::env::temp_dir().join(format!("tolearn-cli-{name}-{}", std::process::id()));
    if directory.exists() {
        std::fs::remove_dir_all(&directory).unwrap();
    }
    std::fs::create_dir_all(&directory).unwrap();
    directory
}

pub fn copied(name: &str) -> PathBuf {
    let directory = scratch(name);
    copy(&reference(), &directory);
    strip(&directory, "json");
    directory
}

pub fn dual(name: &str) -> PathBuf {
    let directory = scratch(name);
    copy(&reference(), &directory);
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

pub fn tolearn(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tolearn"))
        .args(args)
        .output()
        .unwrap()
}

pub fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

pub fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

pub fn json(output: &Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).expect("вывод --json должен быть JSON")
}

pub fn code(output: &Output) -> i32 {
    output.status.code().expect("команда не убита сигналом")
}
