#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "cli gate: a panic here is the report"
)]

#[path = "../../../tests-support/scratch.rs"]
mod rooms;

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

pub fn scratch(name: &str) -> PathBuf {
    rooms::made(&format!("cli-{name}"))
}

pub fn copy(from: &Path, to: &Path) {
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
